//! The configuration shared with the CLI and the preferences of the
//! interface (SPEC-UI §4.3 "Config e preferenze", §5.4 "Preferenze").

use std::path::Path;

use portolano_core::AppConfig;
use portolano_ops::config::{set_config_value, unset_config_value};
use tauri::{AppHandle, Runtime};

use super::blocking;
use crate::dialogs;
use crate::dto::{ConfigEntry, config_entries};
use crate::error::IpcError;
use crate::prefs::InterfacePreferences;
use crate::state::AppState;

/// Keys whose value is a file. They are chosen with a native dialog
/// (`choose_config_file`), never typed: the frontend gives no path of its
/// own (SPEC-UI §1.5, §4.6).
const FILE_KEYS: [&str; 4] = [
    "default_document",
    "vault.file",
    "ssh.program",
    "rdp.freerdp_program",
];

/// The keys of `config.toml` and their values.
#[tauri::command]
#[specta::specta]
pub async fn config_get<R: Runtime>(app: AppHandle<R>) -> Result<Vec<ConfigEntry>, IpcError> {
    blocking(app, |_, state| Ok(config_entries(&state.config()?))).await
}

/// Sets `key` to `value`, validated by the core or by the protocol of the
/// section (DEC-008, DEC-011), and saves `config.toml`.
#[tauri::command]
#[specta::specta]
pub async fn config_set<R: Runtime>(
    app: AppHandle<R>,
    key: String,
    value: String,
) -> Result<Vec<ConfigEntry>, IpcError> {
    blocking(app, move |_, state| set(state, &key, &value)).await
}

/// Removes `key` from `config.toml`: it takes its default again.
#[tauri::command]
#[specta::specta]
pub async fn config_reset<R: Runtime>(
    app: AppHandle<R>,
    key: String,
) -> Result<Vec<ConfigEntry>, IpcError> {
    blocking(app, move |_, state| {
        update(state, |config| {
            Ok(unset_config_value(config, &key, state.protocols())?)
        })
    })
    .await
}

/// Asks for the file of `key` (a document, the file vault or a program)
/// with the native dialog and sets it; `None` if the user cancels.
#[tauri::command]
#[specta::specta]
pub async fn choose_config_file<R: Runtime>(
    app: AppHandle<R>,
    key: String,
) -> Result<Option<Vec<ConfigEntry>>, IpcError> {
    blocking(app, move |app, state| {
        if !FILE_KEYS.contains(&key.as_str()) {
            return Err(IpcError::invalid(
                &key,
                format!("'{key}' non è un percorso di file"),
            ));
        }
        let dialog = match key.as_str() {
            "default_document" => dialogs::document(app),
            _ => dialogs::any_file(app),
        };
        // The file vault may not exist yet: it is created at the first use.
        let chosen = if key == "vault.file" {
            dialog
                .set_file_name(portolano_core::config::VAULT_FILE_NAME)
                .blocking_save_file()
        } else {
            dialog.blocking_pick_file()
        };
        dialogs::local_path(chosen)
            .map(|path| set_file(state, &key, &path))
            .transpose()
    })
    .await
}

/// The preferences of the interface.
#[tauri::command]
#[specta::specta]
pub async fn ui_prefs_get<R: Runtime>(app: AppHandle<R>) -> Result<InterfacePreferences, IpcError> {
    blocking(app, |_, state| Ok(state.ui_preferences().interface)).await
}

/// Changes the preferences of the interface and saves them in `ui.toml`.
#[tauri::command]
#[specta::specta]
pub async fn ui_prefs_set<R: Runtime>(
    app: AppHandle<R>,
    preferences: InterfacePreferences,
) -> Result<InterfacePreferences, IpcError> {
    blocking(app, move |_, state| {
        Ok(state
            .update_ui_preferences(|current| current.interface = preferences)?
            .interface)
    })
    .await
}

pub(crate) fn set(state: &AppState, key: &str, value: &str) -> Result<Vec<ConfigEntry>, IpcError> {
    if FILE_KEYS.contains(&key) {
        return Err(IpcError::invalid(
            key,
            "questo percorso si sceglie con «Scegli…»".to_owned(),
        ));
    }
    update(state, |config| {
        Ok(set_config_value(config, key, value, state.protocols())?)
    })
}

pub(crate) fn set_file(
    state: &AppState,
    key: &str,
    path: &Path,
) -> Result<Vec<ConfigEntry>, IpcError> {
    // As a TOML string, so that no path is read as another literal.
    let value = toml::Value::String(path.display().to_string()).to_string();
    update(state, |config| {
        Ok(set_config_value(config, key, &value, state.protocols())?)
    })
}

/// Applies `change` to the configuration on disk and saves it.
fn update(
    state: &AppState,
    change: impl FnOnce(&AppConfig) -> Result<AppConfig, IpcError>,
) -> Result<Vec<ConfigEntry>, IpcError> {
    let config = change(&state.config()?)?;
    config
        .save(&state.paths().config)
        .map_err(portolano_ops::OpsError::from)?;
    Ok(config_entries(&config))
}

#[cfg(test)]
mod tests;
