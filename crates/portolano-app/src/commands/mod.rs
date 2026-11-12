//! The commands of the app (SPEC-UI §4.3). Each one is also listed in
//! `build.rs`, which generates its permission, and in the capability of the
//! main window.
//!
//! Commands are thin adapters: the logic is in `portolano-ops`, and what is
//! left here lives in functions that receive the state, tested without
//! Tauri.

mod connect;
mod document;
mod icons;
mod import;
mod nodes;
mod secrets;
mod settings;
mod tree;
mod validate;
mod vault;

use tauri::{AppHandle, Manager, Runtime, Wry};
use tauri_specta::{Builder, ErrorHandlingMode, collect_commands, collect_events};

use crate::builtin_icons::BUILTIN_ICONS;
use crate::dto::file_name;
use crate::error::IpcError;
use crate::events::{
    DocumentChanged, DocumentMissing, DocumentModifiedExternally, VaultLocked, emit,
};
use crate::menu::MenuAction;
use crate::state::AppState;

/// The commands and the events, with the shape of their TypeScript
/// bindings, and the names of the builtin icons as a constant.
///
/// Errors reach the frontend as values, `{ status: "error", error }`, so
/// that every caller has to handle the [`IpcError`].
pub fn builder<R: Runtime>() -> Builder<R> {
    Builder::new()
        .error_handling(ErrorHandlingMode::Result)
        .commands(collect_commands![
            document::app_status::<Wry>,
            document::choose_document::<Wry>,
            document::choose_new_document::<Wry>,
            document::open_document::<Wry>,
            document::create_document::<Wry>,
            document::recent_documents::<Wry>,
            document::reload_document::<Wry>,
            document::close_document::<Wry>,
            tree::tree::<Wry>,
            tree::node_details::<Wry>,
            tree::search::<Wry>,
            nodes::add_node::<Wry>,
            nodes::edit_node::<Wry>,
            nodes::move_node::<Wry>,
            nodes::choose_key_file::<Wry>,
            nodes::duplicate_node::<Wry>,
            nodes::preview_delete::<Wry>,
            nodes::delete_node::<Wry>,
            nodes::undo_delete::<Wry>,
            secrets::secret_status::<Wry>,
            secrets::set_secret::<Wry>,
            secrets::clear_secret::<Wry>,
            vault::vault_status::<Wry>,
            vault::unlock_vault::<Wry>,
            vault::lock_vault::<Wry>,
            vault::create_file_vault::<Wry>,
            validate::validate::<Wry>,
            import::choose_import_file::<Wry>,
            import::import_preview::<Wry>,
            import::import_apply::<Wry>,
            import::save_import_report::<Wry>,
            connect::connect_preview::<Wry>,
            connect::login_request::<Wry>,
            settings::config_get::<Wry>,
            settings::config_set::<Wry>,
            settings::config_reset::<Wry>,
            settings::choose_config_file::<Wry>,
            settings::ui_prefs_get::<Wry>,
            settings::ui_prefs_set::<Wry>,
            icons::builtin_icons,
            icons::custom_icons::<Wry>,
            icons::choose_icon_file::<Wry>,
            icons::add_icon::<Wry>,
        ])
        .events(collect_events![
            DocumentChanged,
            DocumentModifiedExternally,
            DocumentMissing,
            VaultLocked,
            MenuAction,
        ])
        .constant("BUILTIN_ICONS", BUILTIN_ICONS)
}

/// Runs `work` on a thread that may block (file I/O, the key derivation of
/// the vault, native dialogs), with the state of the app.
async fn blocking<R: Runtime, T: Send + 'static>(
    app: AppHandle<R>,
    work: impl FnOnce(&AppHandle<R>, &AppState) -> Result<T, IpcError> + Send + 'static,
) -> Result<T, IpcError> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        work(&app, &state)
    })
    .await
    .map_err(|error| IpcError::internal(&format!("comando interrotto: {error}")))?
}

/// `result`, after telling the frontend that the document was saved when it
/// is a success.
fn changed<R: Runtime, T>(
    app: &AppHandle<R>,
    state: &AppState,
    result: Result<T, IpcError>,
) -> Result<T, IpcError> {
    if result.is_ok()
        && let Some(path) = state.document_path()
    {
        emit(
            app,
            DocumentChanged {
                file_name: file_name(&path),
            },
        );
    }
    result
}

#[cfg(test)]
pub(crate) mod test_support;
