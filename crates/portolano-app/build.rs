//! Build script: Tauri context, and the permissions of the app commands.

/// The commands of the app. Listing them makes Tauri generate one
/// `allow-<command>` permission each, so that a window gets only the
/// commands its capability names (SPEC-UI §4.8). A command registered in
/// `lib.rs` but missing here is refused at runtime.
const COMMANDS: &[&str] = &[
    "app_status",
    "choose_document",
    "choose_new_document",
    "open_document",
    "create_document",
    "recent_documents",
    "reload_document",
    "close_document",
    "tree",
    "node_details",
    "search",
    "add_node",
    "edit_node",
    "move_node",
    "choose_key_file",
    "duplicate_node",
    "preview_delete",
    "delete_node",
    "undo_delete",
    "secret_status",
    "set_secret",
    "clear_secret",
    "vault_status",
    "unlock_vault",
    "lock_vault",
    "create_file_vault",
    "validate",
    "choose_import_file",
    "import_preview",
    "import_apply",
    "save_import_report",
    "connect_preview",
    "login_request",
    "config_get",
    "config_set",
    "config_reset",
    "choose_config_file",
    "ui_prefs_get",
    "ui_prefs_set",
    "builtin_icons",
    "custom_icons",
    "choose_icon_file",
    "add_icon",
    "open_session",
    "session_input",
    "session_resize",
    "session_close",
    "open_in_terminal",
];

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let manifest = tauri_build::AppManifest::new().commands(COMMANDS);
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(manifest))?;
    Ok(())
}
