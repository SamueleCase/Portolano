//! Deletion of a node and its subtree (SPEC §10.2 `delete`, DEC-018.1).
//!
//! Two phases: [`preview_delete`] tells what would go and who refers to it,
//! so that the caller can ask for confirmation; [`delete`] does it.

use std::collections::HashSet;

use portolano_core::{Document, NodeKind, Store, Vault};
use serde::Serialize;
use uuid::Uuid;

use crate::commit::{Change, CleanupFailure};
use crate::inspect::NodeType;
use crate::nodes::collect_unused_icons;
use crate::tree_util::{node_path, subtree_ids};
use crate::{OpsError, Result};

/// A reference from a node outside the deleted subtree to a node inside it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ExternalReference {
    /// The node that holds the reference.
    pub from: Uuid,
    /// Its path.
    pub from_path: String,
    /// The referenced node, in the subtree.
    pub target: Uuid,
    /// Its path.
    pub target_path: String,
}

/// What a deletion would remove.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct DeletePreview {
    /// The node to delete.
    pub id: Uuid,
    /// Its path.
    pub path: String,
    /// Its type.
    pub node_type: NodeType,
    /// Its direct children (a folder needs a recursive deletion if any).
    pub children: usize,
    /// All its descendants.
    pub descendants: usize,
    /// Secrets of the subtree that would be deleted from the vault.
    pub secrets: usize,
    /// References from outside the subtree to nodes in it.
    pub references: Vec<ExternalReference>,
}

impl DeletePreview {
    /// Whether [`delete`] with `options` would be refused, so that a caller
    /// can tell before asking for confirmation.
    ///
    /// # Errors
    /// [`OpsError::FolderNotEmpty`] for a folder with children without
    /// `recursive`, [`OpsError::Referenced`] for references from outside
    /// without `force`.
    pub fn check(&self, options: DeleteOptions) -> Result<()> {
        if self.children > 0 && !options.recursive {
            return Err(OpsError::FolderNotEmpty {
                path: self.path.clone(),
                children: self.children,
            });
        }
        if !self.references.is_empty() && !options.force {
            return Err(OpsError::Referenced {
                path: self.path.clone(),
                references: self.references.clone(),
            });
        }
        Ok(())
    }
}

/// Options of [`delete`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DeleteOptions {
    /// Delete a folder with its content (`-r`).
    pub recursive: bool,
    /// Delete even if other nodes refer to the subtree (`--force`).
    pub force: bool,
}

/// The result of [`delete`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeleteOutcome {
    /// Nodes removed, the deleted node included.
    pub removed_nodes: usize,
    /// Nodes outside the subtree whose credential became "ask at launch"
    /// because it referred to a deleted credential.
    pub detached: Vec<Uuid>,
    /// References from outside the subtree that are left dangling: those to
    /// nodes that are not credentials, such as jump hosts; `validate`
    /// reports them.
    pub remaining_references: Vec<ExternalReference>,
    /// Secrets deleted from the vault.
    pub secrets_deleted: usize,
    /// Secrets that could not be deleted from the vault.
    pub cleanup_failures: Vec<CleanupFailure>,
}

/// What deleting node `id` would remove, and the references to it from the
/// rest of the document.
///
/// # Errors
/// [`portolano_core::Error::NodeNotFound`].
pub fn preview_delete(doc: &Document, id: Uuid) -> Result<DeletePreview> {
    let node = doc
        .find(id)
        .ok_or(portolano_core::Error::NodeNotFound { id })?;
    let subtree = subtree_ids(node);
    let references = external_references(doc, &subtree);
    let secrets: HashSet<_> = doc.secret_refs_in(id).into_iter().collect();
    Ok(DeletePreview {
        id,
        path: node_path(doc, id),
        node_type: NodeType::of(node),
        children: node.as_folder().map_or(0, |folder| folder.children.len()),
        descendants: subtree.len() - 1,
        secrets: secrets.len(),
        references,
    })
}

/// Deletes node `id` with its subtree, saves the document, then deletes the
/// secrets of the subtree from the vault.
///
/// A folder with children needs `recursive`. References from outside the
/// subtree to any node in it make the deletion fail unless `force`; with
/// `force`, specs that refer to a deleted credential become "ask at launch"
/// (DEC-016.5), and other references (such as jump hosts) are left dangling
/// for `validate` to report.
///
/// # Errors
/// [`portolano_core::Error::NodeNotFound`], [`OpsError::FolderNotEmpty`],
/// [`OpsError::Referenced`], errors of the vault and the store. On error
/// nothing is changed.
pub fn delete(
    store: &Store,
    doc: &mut Document,
    vault: &dyn Vault,
    id: Uuid,
    options: DeleteOptions,
) -> Result<DeleteOutcome> {
    let preview = preview_delete(doc, id)?;
    preview.check(options)?;
    let mut change = Change::from(doc);
    let detached = if options.force {
        detach_credentials(&mut change.doc, id)
    } else {
        Vec::new()
    };
    let remaining_references = external_references(&change.doc, &subtree_of(&change.doc, id));
    change.doc.remove(id)?;
    let committed = change.commit(store, vault, doc, |_| Ok(()))?;
    collect_unused_icons(store, doc);
    Ok(DeleteOutcome {
        removed_nodes: preview.descendants + 1,
        detached,
        remaining_references,
        secrets_deleted: committed.secrets_deleted,
        cleanup_failures: committed.cleanup_failures,
    })
}

/// The references from nodes outside `subtree` to nodes inside it, in tree
/// order.
fn external_references(doc: &Document, subtree: &HashSet<Uuid>) -> Vec<ExternalReference> {
    let mut references = Vec::new();
    for (_, inner) in doc.walk().filter(|(_, n)| subtree.contains(&n.id)) {
        for from in doc.references_to(inner.id) {
            if !subtree.contains(&from) {
                references.push(ExternalReference {
                    from,
                    from_path: node_path(doc, from),
                    target: inner.id,
                    target_path: node_path(doc, inner.id),
                });
            }
        }
    }
    references
}

fn subtree_of(doc: &Document, id: Uuid) -> HashSet<Uuid> {
    doc.find(id).map(subtree_ids).unwrap_or_default()
}

/// Turns the references to every credential of the subtree of `id` into
/// "ask at launch"; returns the changed nodes outside the subtree.
fn detach_credentials(doc: &mut Document, id: Uuid) -> Vec<Uuid> {
    let subtree = subtree_of(doc, id);
    let credentials: Vec<Uuid> = doc
        .walk()
        .filter(|(_, node)| {
            subtree.contains(&node.id) && matches!(node.kind, NodeKind::Credential(_))
        })
        .map(|(_, node)| node.id)
        .collect();
    let mut detached = Vec::new();
    for credential in credentials {
        for changed in doc.detach_credential_references(credential) {
            if !subtree.contains(&changed) && !detached.contains(&changed) {
                detached.push(changed);
            }
        }
    }
    detached
}

#[cfg(test)]
mod tests;
