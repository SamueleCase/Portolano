//! The secrets of one node (SPEC-UI §4.3 "Segreti"): states only, never
//! values; a password arrives in `set_secret` and becomes a secret at once.

use portolano_ops::nodes::EditOutcome;
use portolano_ops::secrets::{clear_secret as clear, secret_status as status, set_secret as set};
use tauri::{AppHandle, Runtime};
use uuid::Uuid;

use super::{blocking, changed};
use crate::dto::{SecretSlotInput, SecretText, SecretView};
use crate::error::IpcError;
use crate::state::AppState;

/// The secrets of node `id` itself and whether the vault has them.
#[tauri::command]
#[specta::specta]
pub async fn secret_status<R: Runtime>(
    app: AppHandle<R>,
    id: Uuid,
) -> Result<Vec<SecretView>, IpcError> {
    blocking(app, move |_, state| states(state, id)).await
}

/// Stores `value` in `slot` of node `id`: an existing entry of the vault is
/// overwritten in place, otherwise the document gets a new reference
/// (DEC-022.1). Returns the secrets of the node after the change.
#[tauri::command]
#[specta::specta]
pub async fn set_secret<R: Runtime>(
    app: AppHandle<R>,
    id: Uuid,
    slot: SecretSlotInput,
    value: SecretText,
) -> Result<Vec<SecretView>, IpcError> {
    blocking(app, move |app, state| {
        let stored = store(state, id, slot, value);
        changed(app, state, stored).and_then(|()| states(state, id))
    })
    .await
}

/// Removes the secret in `slot` of node `id` from the document, then from
/// the vault.
#[tauri::command]
#[specta::specta]
pub async fn clear_secret<R: Runtime>(
    app: AppHandle<R>,
    id: Uuid,
    slot: SecretSlotInput,
) -> Result<EditOutcome, IpcError> {
    blocking(app, move |app, state| {
        let vault = state.vault()?;
        let cleared = state.with_document(|open| {
            let (store, doc) = open.parts();
            Ok(clear(store, doc, vault.as_ref(), id, slot.into())?)
        });
        changed(app, state, cleared)
    })
    .await
}

pub(crate) fn states(state: &AppState, id: Uuid) -> Result<Vec<SecretView>, IpcError> {
    let vault = state.vault()?;
    state.with_document(|open| {
        Ok(status(open.document(), vault.as_ref(), id)?
            .into_iter()
            .map(SecretView::from)
            .collect())
    })
}

pub(crate) fn store(
    state: &AppState,
    id: Uuid,
    slot: SecretSlotInput,
    value: SecretText,
) -> Result<(), IpcError> {
    let vault = state.vault()?;
    state.with_document(|open| {
        let (store, doc) = open.parts();
        set(
            store,
            doc,
            vault.as_ref(),
            id,
            slot.into(),
            value.into_secret(),
        )?;
        Ok(())
    })
}
