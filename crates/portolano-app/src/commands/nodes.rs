//! Changes to the nodes (SPEC-UI §4.3 "Modifiche"). Each command saves the
//! document through `portolano-ops` and then sends `document-changed`, the
//! invalidation the frontend reacts to.

use portolano_ops::delete::{
    DeleteOutcome, DeletePreview, RestoreOutcome, delete_undoable, preview_delete as preview,
    restore_deleted,
};
use portolano_ops::duplicate::{DuplicateOutcome, duplicate_node as duplicate};
use portolano_ops::moves::move_node as move_to;
use portolano_ops::nodes::{AddOutcome, EditOutcome, add_node as add, edit_node_staged};
use serde::Serialize;
use tauri::{AppHandle, Manager, Runtime};
use uuid::Uuid;

use super::{blocking, changed};
use crate::dialogs;
use crate::dto::{DeleteOptionsInput, NewNodeInput, NodeEditInput, PlacementInput};
use crate::error::{ErrorCode, IpcError};
use crate::state::{AppState, PendingDelete, UNDO_WINDOW, UndoId};

/// Creates a node with its secrets.
#[tauri::command]
#[specta::specta]
pub async fn add_node<R: Runtime>(
    app: AppHandle<R>,
    node: NewNodeInput,
) -> Result<AddOutcome, IpcError> {
    blocking(app, move |app, state| {
        changed(app, state, create(state, node))
    })
    .await
}

/// Applies the changes of the editor, staged secrets included.
#[tauri::command]
#[specta::specta]
pub async fn edit_node<R: Runtime>(
    app: AppHandle<R>,
    id: Uuid,
    edit: NodeEditInput,
) -> Result<EditOutcome, IpcError> {
    blocking(app, move |app, state| {
        changed(app, state, change(state, id, edit))
    })
    .await
}

/// The new path of a moved node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
pub struct MoveOutcome {
    /// The path after the move.
    pub path: String,
}

/// Moves node `id` into `destination` (none is the root), or reorders it.
#[tauri::command]
#[specta::specta]
pub async fn move_node<R: Runtime>(
    app: AppHandle<R>,
    id: Uuid,
    destination: Option<Uuid>,
    placement: PlacementInput,
) -> Result<MoveOutcome, IpcError> {
    blocking(app, move |app, state| {
        let moved = state.with_document(|open| {
            let (store, doc) = open.parts();
            Ok(move_to(store, doc, id, destination, placement.into())?)
        });
        changed(app, state, moved.map(|path| MoveOutcome { path }))
    })
    .await
}

/// Asks for a private key file with the native dialog and returns its
/// path, for the editor: the key file is a datum of the document that `ssh`
/// reads and the app never opens (DEC-031.4). `None` if the user cancels.
#[tauri::command]
#[specta::specta]
pub async fn choose_key_file<R: Runtime>(app: AppHandle<R>) -> Result<Option<String>, IpcError> {
    blocking(app, |app, _| {
        let chosen = dialogs::local_path(dialogs::key_file(app).blocking_pick_file());
        Ok(chosen.map(|path| path.display().to_string()))
    })
    .await
}

/// Copies node `id` with new ids and new secrets right after it.
#[tauri::command]
#[specta::specta]
pub async fn duplicate_node<R: Runtime>(
    app: AppHandle<R>,
    id: Uuid,
) -> Result<DuplicateOutcome, IpcError> {
    blocking(app, move |app, state| {
        let vault = state.vault()?;
        let copied = state.with_document(|open| {
            let (store, doc) = open.parts();
            Ok(duplicate(store, doc, vault.as_ref(), id)?)
        });
        changed(app, state, copied)
    })
    .await
}

/// What deleting node `id` would remove, for the confirmation dialog.
#[tauri::command]
#[specta::specta]
pub async fn preview_delete<R: Runtime>(
    app: AppHandle<R>,
    id: Uuid,
) -> Result<DeletePreview, IpcError> {
    blocking(app, move |_, state| {
        state.with_document(|open| Ok(preview(open.document(), id)?))
    })
    .await
}

/// A deletion that can be undone for a while.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct DeleteResult {
    /// What was done; the secrets are deleted when the deletion can no
    /// longer be undone.
    pub outcome: DeleteOutcome,
    /// For `undo_delete`, during the next 10 seconds.
    pub undo_id: UndoId,
}

/// Deletes node `id` and its subtree; `undo_delete` puts it back during the
/// next 10 seconds, after which its secrets leave the vault.
#[tauri::command]
#[specta::specta]
pub async fn delete_node<R: Runtime>(
    app: AppHandle<R>,
    id: Uuid,
    options: DeleteOptionsInput,
) -> Result<DeleteResult, IpcError> {
    blocking(app, move |app, state| {
        let result = remove(state, id, options);
        if let Ok(deleted) = &result {
            expire_later(app, deleted.undo_id);
        }
        changed(app, state, result)
    })
    .await
}

/// Undoes deletion `undo_id`, if it is still undoable.
#[tauri::command]
#[specta::specta]
pub async fn undo_delete<R: Runtime>(
    app: AppHandle<R>,
    undo_id: UndoId,
) -> Result<RestoreOutcome, IpcError> {
    blocking(app, move |app, state| {
        changed(app, state, undo(state, undo_id))
    })
    .await
}

pub(crate) fn create(state: &AppState, node: NewNodeInput) -> Result<AddOutcome, IpcError> {
    let vault = state.vault()?;
    state.with_document(|open| {
        let node = node.into_new_node(open.document())?;
        let (store, doc) = open.parts();
        Ok(add(store, doc, vault.as_ref(), state.protocols(), node)?)
    })
}

pub(crate) fn change(
    state: &AppState,
    id: Uuid,
    edit: NodeEditInput,
) -> Result<EditOutcome, IpcError> {
    let vault = state.vault()?;
    state.with_document(|open| {
        let current = open
            .document()
            .find(id)
            .ok_or(portolano_ops::OpsError::Core(
                portolano_core::Error::NodeNotFound { id },
            ))?;
        let (edit, changes) = edit.into_edit(current)?;
        let (store, doc) = open.parts();
        Ok(edit_node_staged(
            store,
            doc,
            vault.as_ref(),
            state.protocols(),
            id,
            edit,
            changes,
        )?)
    })
}

pub(crate) fn remove(
    state: &AppState,
    id: Uuid,
    options: DeleteOptionsInput,
) -> Result<DeleteResult, IpcError> {
    let vault = state.vault()?;
    state.with_document(|open| {
        let path = open.path().to_owned();
        let (store, doc) = open.parts();
        let undoable = delete_undoable(store, doc, vault.as_ref(), id, options.into())?;
        let undo_id = state.pending_deletes().insert(&path, undoable.deleted);
        Ok(DeleteResult {
            outcome: undoable.outcome,
            undo_id,
        })
    })
}

pub(crate) fn undo(state: &AppState, undo_id: UndoId) -> Result<RestoreOutcome, IpcError> {
    let PendingDelete {
        document,
        deleted,
        expires_at,
    } = state
        .pending_deletes()
        .take(undo_id)
        .ok_or_else(|| IpcError::not_found("l'eliminazione non si può più annullare".to_owned()))?;
    let fallback = PendingDelete {
        document: document.clone(),
        deleted: deleted.clone(),
        expires_at,
    };
    // The document is compared under its lock: another document opened in
    // the meantime must not receive the subtree.
    let mut other_document = false;
    let restored = state.vault().and_then(|vault| {
        state.with_document(|open| {
            if open.path() != document {
                other_document = true;
                return Err(IpcError::no_document());
            }
            let (store, doc) = open.parts();
            Ok(restore_deleted(store, doc, vault.as_ref(), deleted)?)
        })
    });
    match &restored {
        Ok(_) => {}
        // Definitive: the document of the deletion is no longer open.
        Err(error) if other_document || error.code == ErrorCode::NoDocument => {
            state.purge(fallback);
        }
        // The file changed on disk, a sibling took the name, a disk error:
        // it can be fixed, and the deletion stays undoable until it expires.
        Err(_) => state.put_back(undo_id, fallback),
    }
    restored
}

/// Finishes deletion `undo_id` when it can no longer be undone.
fn expire_later<R: Runtime>(app: &AppHandle<R>, undo_id: UndoId) {
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(UNDO_WINDOW);
        let state = app.state::<AppState>();
        if let Some(pending) = state.pending_deletes().take(undo_id) {
            state.purge(pending);
        }
    });
}

#[cfg(test)]
mod tests;
