//! `--on-conflict update`: imported nodes replace the existing ones with the
//! same id where they are, and new nodes go under their imported parent.

use std::mem;

use portolano_core::{CredentialSpec, Document, Node, NodeKind, PrivateKey, PropertyValue};
use uuid::Uuid;

use super::insert;
use crate::nodes::{credential_spec, credential_spec_mut};
use crate::tree_util::node_path;
use crate::{OpsError, Result};

/// Merges the imported tree `root` into `doc`, visiting it in pre-order: a
/// node whose id exists replaces the existing node in place (parent,
/// position and current children kept), any other node is appended under
/// its imported parent (under `into` for the root). Existing nodes absent
/// from the import are left alone.
///
/// Returns the numbers of inserted and replaced nodes.
///
/// # Errors
/// [`OpsError::ImportKindChanged`] if a node would change kind (folder,
/// connection, credential), except from or to an unsupported node that is
/// not a folder; insertion errors.
pub(super) fn merge(
    doc: &mut Document,
    into: Option<Uuid>,
    root: Node,
    warnings: &mut Vec<String>,
) -> Result<(usize, usize)> {
    if doc.find(root.id).is_some() && doc.parent_of(root.id) != Some(into) {
        warnings.push(format!(
            "la cartella importata esiste già in '{}': aggiornata dove si trova",
            node_path(doc, root.id)
        ));
    }
    let mut counts = (0, 0);
    place(doc, into, root, &mut counts)?;
    Ok(counts)
}

fn place(
    doc: &mut Document,
    parent: Option<Uuid>,
    mut node: Node,
    counts: &mut (usize, usize),
) -> Result<()> {
    let children = node
        .as_folder_mut()
        .map(|folder| mem::take(&mut folder.children))
        .unwrap_or_default();
    let id = node.id;
    let path = node_path(doc, id);
    match doc.find_mut(id) {
        Some(existing) => {
            if !may_replace(existing, &node) {
                return Err(OpsError::ImportKindChanged { path });
            }
            keep_secrets(existing, &mut node);
            if let (Some(old), Some(new)) = (existing.as_folder_mut(), node.as_folder_mut()) {
                new.children = mem::take(&mut old.children);
            }
            *existing = node;
            counts.1 += 1;
        }
        None => counts.0 += insert(doc, parent, node)?,
    }
    for child in children {
        place(doc, Some(id), child, counts)?;
    }
    Ok(())
}

/// Whether an update may put `new` in place of `old`: same kind, or a
/// change from or to an unsupported node that is not a folder. The latter
/// happens when the same file is imported again after the importer learned
/// a new type, and loses nothing: an unsupported node holds no secrets, no
/// children and no valid reference to itself.
fn may_replace(old: &Node, new: &Node) -> bool {
    if mem::discriminant(&old.kind) == mem::discriminant(&new.kind) {
        return true;
    }
    let unsupported = |node: &Node| matches!(node.kind, NodeKind::Unsupported(_));
    (unsupported(old) || unsupported(new)) && !old.is_folder() && !new.is_folder()
}

/// Copies into `new` the secret references of `old` that `new` does not
/// replace (SPEC §8.4): an import without the passwords must not forget
/// those already typed.
fn keep_secrets(old: &Node, new: &mut Node) {
    if let (NodeKind::Credential(old), NodeKind::Credential(new)) = (&old.kind, &mut new.kind) {
        if new.password.is_none() {
            new.password = old.password.clone();
        }
        if new.passphrase.is_none() {
            new.passphrase = old.passphrase.clone();
        }
        if new.private_key.is_none() && matches!(old.private_key, Some(PrivateKey::Vault { .. })) {
            new.private_key = old.private_key.clone();
        }
    }
    if let (
        Some(CredentialSpec::Inline {
            password: Some(old_password),
            ..
        }),
        Some(CredentialSpec::Inline { password, .. }),
    ) = (credential_spec(old), credential_spec_mut(new))
        && password.is_none()
    {
        *password = Some(old_password.clone());
    }
    for property in &old.custom_properties {
        let is_secret = matches!(property.value, PropertyValue::Secret { .. });
        let replaced = new
            .custom_properties
            .iter()
            .any(|kept| kept.name == property.name);
        if is_secret && !replaced {
            new.custom_properties.push(property.clone());
        }
    }
}
