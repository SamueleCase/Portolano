//! Creation of a node with its secrets (`add`).

use portolano_connectors::Registry;
use portolano_core::{Document, Node, PrivateKey, Store, Vault};

use super::credential::with_password;
use super::details::{apply_details, set_secret_properties};
use super::{
    AddOutcome, CredentialInput, NewNode, NewNodeKind, NodeSecrets, check_name, non_empty,
};
use crate::commit::Change;
use crate::secrets::SecretSlot;
use crate::tree_util::node_path;
use crate::{OpsError, Result};

/// Creates a node at `node.path`, writes its secrets to the vault, stores its
/// custom icon and saves the document.
///
/// The parent must exist (or is created with `create_parents`) and must not
/// already contain a node with the same name. A connection's protocol must
/// be registered and its settings valid for it; a password given for a
/// connection goes into its inline credential (an inherited credential
/// becomes inline).
///
/// # Errors
/// Path errors of the core, [`OpsError::NameTaken`], [`OpsError::EmptyName`],
/// [`OpsError::UnknownProtocol`], [`OpsError::InvalidSettings`],
/// [`OpsError::EmptyHost`], [`OpsError::InvalidPort`],
/// [`OpsError::NotACredential`], [`OpsError::SecretNotApplicable`],
/// [`OpsError::KeyFileAndVaultKey`], [`OpsError::InvalidColor`], errors of
/// the vault, the icon store and the store. On error the document and the vault
/// are unchanged; an image already stored is removed by a later cleanup of
/// the unused icons.
pub fn add_node(
    store: &Store,
    doc: &mut Document,
    vault: &dyn Vault,
    protocols: &Registry,
    node: NewNode,
) -> Result<AddOutcome> {
    let NewNode {
        path,
        create_parents,
        kind,
        details,
        secrets,
    } = node;
    let mut change = Change::from(doc);
    let (parent, name) = if create_parents {
        change.doc.ensure_parent_path(&path)?
    } else {
        change.doc.resolve_parent_path(&path)?
    };
    check_name(&change.doc, parent, &name, None)?;
    let mut new = build_node(&mut change, protocols, &path, name, kind, secrets)?;
    let image = apply_details(&mut new, details)?;
    let id = new.id;
    change.doc.insert(parent, None, new)?;
    let committed = change.commit(store, vault, doc, |doc| match image {
        Some(image) => image.store(store, doc, id),
        None => Ok(()),
    })?;
    Ok(AddOutcome {
        id,
        path: node_path(doc, id),
        secrets_written: committed.secrets_written,
    })
}

/// The node of `kind` named `name`, with its secrets queued in `change`.
fn build_node(
    change: &mut Change,
    protocols: &Registry,
    path: &str,
    name: String,
    kind: NewNodeKind,
    secrets: NodeSecrets,
) -> Result<Node> {
    let NodeSecrets {
        password,
        passphrase,
        private_key,
        properties,
    } = secrets;
    let mut new = match kind {
        NewNodeKind::Folder => {
            refuse(path, password, SecretSlot::Password)?;
            refuse(path, passphrase, SecretSlot::Passphrase)?;
            refuse(path, private_key, SecretSlot::PrivateKey)?;
            Node::folder(name)
        }
        NewNodeKind::Connection {
            protocol,
            host,
            port,
            credential,
            settings,
        } => {
            refuse(path, passphrase, SecretSlot::Passphrase)?;
            refuse(path, private_key, SecretSlot::PrivateKey)?;
            let target = ConnectionTarget {
                protocol,
                host,
                port,
                settings,
            };
            build_connection(change, protocols, path, name, target, credential, password)?
        }
        NewNodeKind::Credential {
            username,
            domain,
            key_file,
        } => {
            let secrets = CredentialSecrets {
                password,
                passphrase,
                private_key,
            };
            build_credential(change, name, username, domain, key_file, secrets)?
        }
    };
    let property_refs = properties
        .into_iter()
        .map(|(name, secret)| (name, change.add_secret(secret)))
        .collect();
    set_secret_properties(&mut new, property_refs);
    Ok(new)
}

/// Where a new connection points.
struct ConnectionTarget {
    protocol: String,
    host: String,
    port: Option<u16>,
    settings: serde_json::Map<String, serde_json::Value>,
}

fn build_connection(
    change: &mut Change,
    protocols: &Registry,
    path: &str,
    name: String,
    target: ConnectionTarget,
    credential: CredentialInput,
    password: Option<portolano_core::Secret>,
) -> Result<Node> {
    let protocol = protocols
        .get(&target.protocol)
        .ok_or_else(|| unknown_protocol(&target.protocol, protocols))?;
    check_host(&target.host, path)?;
    check_port(target.port)?;
    protocol
        .validate_settings(&target.settings)
        .map_err(|reason| OpsError::InvalidSettings {
            path: path.to_owned(),
            reason,
        })?;
    let spec = credential.resolve(&change.doc)?;
    let password = password.map(|secret| change.add_secret(secret));
    let spec = with_password(spec, password, path)?;
    let mut node = Node::connection(name, target.protocol, target.host);
    if let Some(connection) = node.as_connection_mut() {
        connection.port = target.port;
        connection.credential = spec;
        connection.settings = target.settings;
    }
    Ok(node)
}

/// The secrets of a new credential.
struct CredentialSecrets {
    password: Option<portolano_core::Secret>,
    passphrase: Option<portolano_core::Secret>,
    private_key: Option<portolano_core::Secret>,
}

fn build_credential(
    change: &mut Change,
    name: String,
    username: Option<String>,
    domain: Option<String>,
    key_file: Option<String>,
    secrets: CredentialSecrets,
) -> Result<Node> {
    let private_key = match (key_file.and_then(non_empty), secrets.private_key) {
        (Some(_), Some(_)) => return Err(OpsError::KeyFileAndVaultKey),
        (Some(path), None) => Some(PrivateKey::File { path }),
        (None, Some(key)) => Some(PrivateKey::Vault {
            secret: change.add_secret(key),
        }),
        (None, None) => None,
    };
    let password = secrets.password.map(|secret| change.add_secret(secret));
    let passphrase = secrets.passphrase.map(|secret| change.add_secret(secret));
    let mut node = Node::credential(name);
    if let Some(credential) = node.as_credential_mut() {
        credential.username = username.and_then(non_empty);
        credential.domain = domain.and_then(non_empty);
        credential.password = password;
        credential.private_key = private_key;
        credential.passphrase = passphrase;
    }
    Ok(node)
}

fn refuse(path: &str, secret: Option<portolano_core::Secret>, slot: SecretSlot) -> Result<()> {
    match secret {
        Some(_) => Err(OpsError::SecretNotApplicable {
            path: path.to_owned(),
            slot,
        }),
        None => Ok(()),
    }
}

/// [`OpsError::EmptyHost`] for a blank host.
pub(super) fn check_host(host: &str, path: &str) -> Result<()> {
    if host.trim().is_empty() {
        return Err(OpsError::EmptyHost {
            path: path.to_owned(),
        });
    }
    Ok(())
}

/// [`OpsError::InvalidPort`] for port 0.
pub(super) fn check_port(port: Option<u16>) -> Result<()> {
    match port {
        Some(0) => Err(OpsError::InvalidPort { port: 0 }),
        _ => Ok(()),
    }
}

/// [`OpsError::UnknownProtocol`] listing the registered protocols.
pub(crate) fn unknown_protocol(protocol: &str, protocols: &Registry) -> OpsError {
    OpsError::UnknownProtocol {
        protocol: protocol.to_owned(),
        known: protocols.ids().into_iter().map(str::to_owned).collect(),
    }
}
