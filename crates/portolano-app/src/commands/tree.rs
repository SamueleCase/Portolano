//! The tree and the detail of the nodes (SPEC-UI §4.3 "Albero e
//! dettaglio").

use tauri::{AppHandle, Runtime};
use uuid::Uuid;

use super::blocking;
use crate::dto::{NodeDetailsView, TreeView, node_details as details_of, tree_view};
use crate::error::IpcError;
use crate::search::search_nodes;
use crate::state::AppState;

/// The whole document as a tree, with the state of the secrets.
#[tauri::command]
#[specta::specta]
pub async fn tree<R: Runtime>(app: AppHandle<R>) -> Result<TreeView, IpcError> {
    blocking(app, |_, state| tree_of(state)).await
}

/// Everything the detail and the editor show about node `id`.
#[tauri::command]
#[specta::specta]
pub async fn node_details<R: Runtime>(
    app: AppHandle<R>,
    id: Uuid,
) -> Result<NodeDetailsView, IpcError> {
    blocking(app, move |_, state| details(state, id)).await
}

/// The nodes whose name, host or tag contains `query`, ignoring case.
#[tauri::command]
#[specta::specta]
pub async fn search<R: Runtime>(app: AppHandle<R>, query: String) -> Result<Vec<Uuid>, IpcError> {
    blocking(app, move |_, state| {
        state.with_document(|open| Ok(search_nodes(open.document(), &query)))
    })
    .await
}

// The views read the vault once per secret (the keychain may take a while
// for each): they work on a copy of the document, so that the lock of the
// document is not held meanwhile and other commands are not blocked.

pub(crate) fn tree_of(state: &AppState) -> Result<TreeView, IpcError> {
    let vault = state.vault()?;
    let doc = state.with_document(|open| Ok(open.document().clone()))?;
    tree_view(&doc, state.protocols(), vault.as_ref())
}

pub(crate) fn details(state: &AppState, id: Uuid) -> Result<NodeDetailsView, IpcError> {
    let vault = state.vault()?;
    let doc = state.with_document(|open| Ok(open.document().clone()))?;
    details_of(&doc, state.protocols(), vault.as_ref(), id)
}
