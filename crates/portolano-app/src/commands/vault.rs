//! The vault (SPEC-UI §4.3 "Vault", §5.4 "Sblocco del vault"): the file
//! vault stays locked until its master password is typed; the keychain has
//! no state of its own in the app (ambiguity 20).

use portolano_core::{Secret, SecretRef, Vault, VaultSelection};
use portolano_ops::OpsError;
use tauri::{AppHandle, Runtime};

use super::blocking;
use crate::dto::{SecretText, VaultKind, VaultStatus};
use crate::error::IpcError;
use crate::events::{VaultLocked, emit};
use crate::state::AppState;

/// The vault of the configuration and whether it is locked.
#[tauri::command]
#[specta::specta]
pub async fn vault_status<R: Runtime>(app: AppHandle<R>) -> Result<VaultStatus, IpcError> {
    blocking(app, |_, state| status(state)).await
}

/// Opens the file vault with `master_password`.
#[tauri::command]
#[specta::specta]
pub async fn unlock_vault<R: Runtime>(
    app: AppHandle<R>,
    master_password: SecretText,
) -> Result<VaultStatus, IpcError> {
    blocking(app, move |_, state| {
        unlock(state, master_password.into_secret())
    })
    .await
}

/// Closes the file vault: the master password is needed again.
#[tauri::command]
#[specta::specta]
pub async fn lock_vault<R: Runtime>(app: AppHandle<R>) -> Result<VaultStatus, IpcError> {
    blocking(app, |app, state| {
        let status = lock(state)?;
        emit(app, VaultLocked { kind: status.kind });
        Ok(status)
    })
    .await
}

/// Creates the file vault of the configuration with `master_password`, of
/// at least 12 characters (DEC-024.7), and opens it.
#[tauri::command]
#[specta::specta]
pub async fn create_file_vault<R: Runtime>(
    app: AppHandle<R>,
    master_password: SecretText,
) -> Result<VaultStatus, IpcError> {
    blocking(app, move |_, state| {
        create(state, master_password.into_secret())
    })
    .await
}

pub(crate) fn status(state: &AppState) -> Result<VaultStatus, IpcError> {
    let selection = state.locations()?.vault;
    let vault = state.vault()?;
    Ok(match &selection {
        VaultSelection::File { path } => VaultStatus {
            kind: VaultKind::File,
            path: Some(path.display().to_string()),
            exists: path.is_file(),
            locked: !vault.is_open(),
        },
        other => VaultStatus {
            kind: VaultKind::from(other),
            path: None,
            exists: true,
            locked: false,
        },
    })
}

pub(crate) fn unlock(state: &AppState, master_password: Secret) -> Result<VaultStatus, IpcError> {
    let path = file_vault(state)?;
    if !path.is_file() {
        return Err(IpcError::not_found(format!(
            "il vault su file '{}' non esiste ancora: crealo",
            path.display()
        )));
    }
    open_with(state, master_password)
}

pub(crate) fn create(state: &AppState, master_password: Secret) -> Result<VaultStatus, IpcError> {
    let path = file_vault(state)?;
    if path.exists() {
        return Err(OpsError::Core(portolano_core::Error::AlreadyExists { path }).into());
    }
    open_with(state, master_password)
}

pub(crate) fn lock(state: &AppState) -> Result<VaultStatus, IpcError> {
    state.vault()?.lock();
    status(state)
}

/// The file of the file vault of the configuration.
fn file_vault(state: &AppState) -> Result<std::path::PathBuf, IpcError> {
    match state.locations()?.vault {
        VaultSelection::File { path } => Ok(path),
        _ => Err(IpcError::invalid(
            "vault.backend",
            "il vault configurato non è un file cifrato".to_owned(),
        )),
    }
}

/// Opens the vault (creating the file if it does not exist) with
/// `master_password`, which leaves the slot once the vault holds the key
/// derived from it.
fn open_with(state: &AppState, master_password: Secret) -> Result<VaultStatus, IpcError> {
    let vault = state.vault()?;
    state.unlock_slot().fill(master_password);
    let opened = vault.contains(&SecretRef::new());
    state.unlock_slot().clear();
    opened.map_err(OpsError::from)?;
    status(state)
}

#[cfg(test)]
mod tests;
