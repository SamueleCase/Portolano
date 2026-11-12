//! Undoable deletion (SPEC-UI §3.6, §5.4 "Eliminato · Annulla"): the
//! subtree leaves the document at once, but its secrets stay in the vault
//! until the deletion can no longer be undone, so that putting the subtree
//! back finds them again.

use std::collections::HashSet;

mod detached;

use portolano_core::{Document, Node, SecretRef, Store, Vault};
use serde::Serialize;
use uuid::Uuid;

use self::detached::{Detached, Restored};
use super::{
    DeleteOptions, DeleteOutcome, detach_credentials, external_references, preview_delete,
    subtree_of,
};
use crate::Result;
use crate::commit::{Change, CleanupFailure, delete_secrets};
use crate::inspect::NodeRef;
use crate::nodes::{check_name, collect_unused_icons};
use crate::tree_util::{document_secret_refs, node_path};

/// What [`delete_undoable`] removed, to put it back with
/// [`restore_deleted`] or to finish with [`purge_secrets`].
#[derive(Debug, Clone)]
pub struct DeletedSubtree {
    node: Node,
    parent: Option<Uuid>,
    position: usize,
    detached: Vec<Detached>,
    secrets: Vec<SecretRef>,
}

impl DeletedSubtree {
    /// The id of the deleted node.
    pub fn id(&self) -> Uuid {
        self.node.id
    }

    /// The secrets left in the vault, to delete when the deletion can no
    /// longer be undone.
    pub fn secrets(&self) -> &[SecretRef] {
        &self.secrets
    }
}

/// The result of [`delete_undoable`].
#[derive(Debug, Clone)]
pub struct UndoableDelete {
    /// What was done; no secret is deleted yet.
    pub outcome: DeleteOutcome,
    /// What is needed to undo it.
    pub deleted: DeletedSubtree,
}

/// The result of [`restore_deleted`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct RestoreOutcome {
    /// The node put back.
    pub id: Uuid,
    /// Its path.
    pub path: String,
    /// Nodes that the deletion made "ask at launch" and that changed again
    /// before the undo: what changed since stays, so their credential may
    /// not be the one they had before the deletion.
    pub kept: Vec<NodeRef>,
}

/// The result of [`purge_secrets`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PurgeOutcome {
    /// Secrets deleted from the vault.
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub secrets_deleted: usize,
    /// Secrets that could not be deleted.
    pub cleanup_failures: Vec<CleanupFailure>,
}

/// Deletes node `id` like [`super::delete`] and saves the document, but
/// leaves the secrets of the subtree in the vault (and the images it used in
/// the icon store): [`restore_deleted`] puts everything back,
/// [`purge_secrets`] finishes the deletion.
///
/// The vault is still checked before the document changes, so that a
/// locked vault stops the deletion instead of the later cleanup.
///
/// # Errors
/// As [`super::delete`].
pub fn delete_undoable(
    store: &Store,
    doc: &mut Document,
    vault: &dyn Vault,
    id: Uuid,
    options: DeleteOptions,
) -> Result<UndoableDelete> {
    let preview = preview_delete(doc, id)?;
    preview.check(options)?;
    let parent = doc
        .parent_of(id)
        .ok_or(portolano_core::Error::NodeNotFound { id })?;
    let position = doc
        .children(parent)?
        .iter()
        .position(|node| node.id == id)
        .unwrap_or_default();
    let mut change = Change::from(doc);
    let detached = if options.force {
        detach_credentials(&mut change.doc, id)
    } else {
        Vec::new()
    };
    let previous = detached
        .iter()
        .map(|&node| Detached::record(doc, &change.doc, node))
        .collect();
    let remaining_references = external_references(&change.doc, &subtree_of(&change.doc, id));
    let node = change.doc.remove(id)?;
    let secrets = change.commit_keeping_obsolete(store, vault, doc)?;
    Ok(UndoableDelete {
        outcome: DeleteOutcome {
            removed_nodes: preview.descendants + 1,
            detached,
            remaining_references,
            secrets_deleted: 0,
            cleanup_failures: Vec::new(),
        },
        deleted: DeletedSubtree {
            node,
            parent,
            position,
            detached: previous,
            secrets,
        },
    })
}

/// Puts a deleted subtree back where it was (same parent, same position)
/// and saves the document; the nodes that the deletion made "ask at launch"
/// get their credential back where nobody changed it since (DEC-031.2), the
/// others are listed in [`RestoreOutcome::kept`]. The vault is not touched:
/// the secrets of the subtree are still in it.
///
/// # Errors
/// [`portolano_core::Error::NodeNotFound`] or
/// [`portolano_core::Error::NotAFolder`] if the parent is gone,
/// [`crate::OpsError::NameTaken`] if a sibling took the name meanwhile,
/// [`portolano_core::Error::DuplicateId`], errors of the store. On error
/// the document is unchanged.
pub fn restore_deleted(
    store: &Store,
    doc: &mut Document,
    vault: &dyn Vault,
    deleted: DeletedSubtree,
) -> Result<RestoreOutcome> {
    let DeletedSubtree {
        node,
        parent,
        position,
        detached,
        ..
    } = deleted;
    let id = node.id;
    check_name(doc, parent, &node.name, None)?;
    let mut change = Change::from(doc);
    change.doc.insert(parent, Some(position), node)?;
    let mut kept = Vec::new();
    for changes in detached {
        let node = changes.node();
        if changes.restore(&mut change.doc) == Restored::Kept {
            kept.push(node);
        }
    }
    change.commit(store, vault, doc, |_| Ok(()))?;
    Ok(RestoreOutcome {
        id,
        path: node_path(doc, id),
        kept: kept
            .into_iter()
            .map(|node| NodeRef::of(doc, node))
            .collect(),
    })
}

/// Finishes undoable deletions: deletes `secrets` from the vault, except
/// those that `doc` references again, and removes the images no node uses
/// any more.
///
/// A secret that cannot be deleted is reported, not an error: the deleted
/// nodes are already gone from the saved document.
pub fn purge_secrets(
    store: &Store,
    doc: &Document,
    vault: &dyn Vault,
    secrets: &[SecretRef],
) -> PurgeOutcome {
    let referenced: HashSet<SecretRef> = document_secret_refs(doc);
    let obsolete = secrets
        .iter()
        .filter(|reference| !referenced.contains(*reference))
        .cloned()
        .collect();
    let purged = delete_secrets(vault, obsolete);
    collect_unused_icons(store, doc);
    PurgeOutcome {
        secrets_deleted: purged.secrets_deleted,
        cleanup_failures: purged.cleanup_failures,
    }
}

#[cfg(test)]
mod tests;
