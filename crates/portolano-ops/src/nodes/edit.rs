//! Editing of an existing node (`edit`).

use portolano_connectors::Registry;
use portolano_core::{CredentialSpec, Document, Node, NodeKind, SecretRef, Store, Vault};
use uuid::Uuid;

use super::add::{check_host, check_port, unknown_protocol};
use super::credential::credential_spec_mut;
use super::details::{NodeDetails, PendingImage, apply_details, set_secret_properties};
use super::staged::{EditorChanges, StagedChanges};
use super::{EditOutcome, NodeEdit, check_name, collect_unused_icons, non_empty};
use crate::commit::Change;
use crate::tree_util::node_path;
use crate::{OpsError, Result};

/// Applies `edit` to node `id` and saves the document.
///
/// Settings are changed by top-level key and validated again by the
/// protocol. Secrets that the node no longer references (a replaced secret
/// property, the password of an inline credential replaced by another
/// credential) are deleted from the vault after saving.
///
/// # Errors
/// [`portolano_core::Error::NodeNotFound`], [`OpsError::NameTaken`],
/// [`OpsError::EmptyName`], [`OpsError::CredentialWithLogin`],
/// [`OpsError::NotEditable`] for a field the node does not have,
/// [`OpsError::UnknownProtocol`], [`OpsError::InvalidSettings`],
/// [`OpsError::EmptyHost`], [`OpsError::InvalidPort`],
/// [`OpsError::NotACredential`], [`OpsError::InvalidColor`], errors of the
/// vault, the icon store and the store. On error the document and the vault
/// are unchanged; an image already stored is removed by a later cleanup of
/// the unused icons.
pub fn edit_node(
    store: &Store,
    doc: &mut Document,
    vault: &dyn Vault,
    protocols: &Registry,
    id: Uuid,
    edit: NodeEdit,
) -> Result<EditOutcome> {
    edit_node_staged(
        store,
        doc,
        vault,
        protocols,
        id,
        edit,
        EditorChanges::default(),
    )
}

/// [`edit_node`] with the changes of the editor of the app (SPEC-UI §3.6):
/// the secrets staged in the editor are written to the vault with new
/// references in the same save as the other changes, and the secrets they
/// replace or clear are deleted from the vault after the save.
///
/// # Errors
/// As [`edit_node`], plus [`OpsError::SecretNotApplicable`],
/// [`OpsError::SecretNotSet`] and [`OpsError::KeyFileAndVaultKey`] for the
/// staged changes. On error the document and the vault are unchanged.
pub fn edit_node_staged(
    store: &Store,
    doc: &mut Document,
    vault: &dyn Vault,
    protocols: &Registry,
    id: Uuid,
    edit: NodeEdit,
    changes: EditorChanges,
) -> Result<EditOutcome> {
    doc.find(id)
        .ok_or(portolano_core::Error::NodeNotFound { id })?;
    let path = node_path(doc, id);
    check_edit(doc, id, &path, &edit)?;
    let mut change = Change::from(doc);
    let edit = ResolvedEdit::new(edit, &mut change)?;
    let staged = StagedChanges::new(changes, &mut change);
    let icon_changed = edit.details.icon.is_some();
    let node = change
        .doc
        .find_mut(id)
        .ok_or(portolano_core::Error::NodeNotFound { id })?;
    let image = edit.apply(node, protocols, &path)?;
    staged.apply(node, &path)?;
    let committed = change.commit(store, vault, doc, |doc| match image {
        Some(image) => image.store(store, doc, id),
        None => Ok(()),
    })?;
    if icon_changed {
        collect_unused_icons(store, doc);
    }
    Ok(EditOutcome {
        path: node_path(doc, id),
        secrets_written: committed.secrets_written,
        secrets_deleted: committed.secrets_deleted,
        cleanup_failures: committed.cleanup_failures,
    })
}

/// The checks that need the whole document: the new name among the
/// siblings, and a credential given together with a user or a domain,
/// which would leave one of the two ignored.
fn check_edit(doc: &Document, id: Uuid, path: &str, edit: &NodeEdit) -> Result<()> {
    if let Some(name) = &edit.name {
        check_name(doc, doc.parent_of(id).flatten(), name, Some(id))?;
    }
    if edit.credential.is_some() && (edit.username.is_some() || edit.domain.is_some()) {
        return Err(OpsError::CredentialWithLogin {
            path: path.to_owned(),
        });
    }
    Ok(())
}

/// An edit whose credential is resolved against the document and whose new
/// secrets are queued in the change.
struct ResolvedEdit {
    name: Option<String>,
    details: NodeDetails,
    target: TargetEdit,
    spec: Option<CredentialSpec>,
    username: Option<String>,
    domain: Option<String>,
    property_refs: Vec<(String, SecretRef)>,
}

impl ResolvedEdit {
    fn new(edit: NodeEdit, change: &mut Change) -> Result<ResolvedEdit> {
        let spec = edit
            .credential
            .map(|input| input.resolve(&change.doc))
            .transpose()?;
        let property_refs = edit
            .secret_properties
            .into_iter()
            .map(|(name, secret)| (name, change.add_secret(secret)))
            .collect();
        Ok(ResolvedEdit {
            name: edit.name,
            details: edit.details,
            target: TargetEdit {
                host: edit.host,
                port: edit.port,
                settings_set: edit.settings_set,
                settings_unset: edit.settings_unset,
            },
            spec,
            username: edit.username,
            domain: edit.domain,
            property_refs,
        })
    }

    /// Changes `node`; a custom image is returned to be stored in the order
    /// of the commit.
    fn apply(
        self,
        node: &mut Node,
        protocols: &Registry,
        path: &str,
    ) -> Result<Option<PendingImage>> {
        if let Some(name) = self.name {
            node.name = name;
        }
        let image = apply_details(node, self.details)?;
        edit_target(node, protocols, path, self.target)?;
        edit_login(node, path, self.spec, self.username, self.domain)?;
        set_secret_properties(node, self.property_refs);
        node.touch();
        Ok(image)
    }
}

/// Changes to where a connection points.
struct TargetEdit {
    host: Option<String>,
    port: Option<u16>,
    settings_set: Vec<(String, serde_json::Value)>,
    settings_unset: Vec<String>,
}

fn edit_target(node: &mut Node, protocols: &Registry, path: &str, edit: TargetEdit) -> Result<()> {
    let settings_changed = !edit.settings_set.is_empty() || !edit.settings_unset.is_empty();
    let field = if edit.host.is_some() {
        "host"
    } else if edit.port.is_some() {
        "port"
    } else if settings_changed {
        "settings"
    } else {
        return Ok(());
    };
    let Some(connection) = node.as_connection_mut() else {
        return Err(OpsError::NotEditable {
            path: path.to_owned(),
            field,
        });
    };
    if let Some(host) = edit.host {
        check_host(&host, path)?;
        connection.host = host;
    }
    if edit.port.is_some() {
        check_port(edit.port)?;
        connection.port = edit.port;
    }
    if settings_changed {
        for key in &edit.settings_unset {
            connection.settings.remove(key);
        }
        connection.settings.extend(edit.settings_set);
        let protocol = protocols
            .get(&connection.protocol)
            .ok_or_else(|| unknown_protocol(&connection.protocol, protocols))?;
        protocol
            .validate_settings(&connection.settings)
            .map_err(|reason| OpsError::InvalidSettings {
                path: path.to_owned(),
                reason,
            })?;
    }
    Ok(())
}

/// Applies a new credential spec, or a new user name or domain (never both,
/// see [`check_edit`]). On a credential node the user and domain are its
/// own fields; on a folder or a connection they belong to the inline
/// credential, which replaces any other spec.
fn edit_login(
    node: &mut Node,
    path: &str,
    spec: Option<CredentialSpec>,
    username: Option<String>,
    domain: Option<String>,
) -> Result<()> {
    let not_editable = |field| OpsError::NotEditable {
        path: path.to_owned(),
        field,
    };
    if let NodeKind::Credential(credential) = &mut node.kind {
        if spec.is_some() {
            return Err(not_editable("credential"));
        }
        if let Some(username) = username {
            credential.username = non_empty(username);
        }
        if let Some(domain) = domain {
            credential.domain = non_empty(domain);
        }
        return Ok(());
    }
    if spec.is_none() && username.is_none() && domain.is_none() {
        return Ok(());
    }
    let field = if spec.is_some() { "credential" } else { "user" };
    let slot = credential_spec_mut(node).ok_or_else(|| not_editable(field))?;
    *slot = match spec {
        Some(spec) => spec,
        None => inline_with(std::mem::take(slot), username, domain),
    };
    Ok(())
}

/// `spec` as an inline credential with the new user and domain, keeping its
/// inline fields (and password) when it already was inline.
fn inline_with(
    spec: CredentialSpec,
    new_username: Option<String>,
    new_domain: Option<String>,
) -> CredentialSpec {
    let (username, domain, password) = match spec {
        CredentialSpec::Inline {
            username,
            domain,
            password,
        } => (username, domain, password),
        _ => (None, None, None),
    };
    CredentialSpec::Inline {
        username: new_username.map_or(username, non_empty),
        domain: new_domain.map_or(domain, non_empty),
        password,
    }
}
