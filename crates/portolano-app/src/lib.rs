//! Portolano desktop app (SPEC-UI): the Tauri backend of the React frontend.
//!
//! Commands are thin adapters over `portolano-ops`; their TypeScript
//! bindings are generated (`bindings.ts`). The frontend never receives a
//! secret (SPEC-UI §1.1).

mod bindings;
mod commands;
mod dto;
mod error;
mod icons;
pub mod sessions;
mod sidecar;
mod state;

use tauri::{Manager, UriSchemeContext, http};

pub use bindings::export_bindings;

use crate::state::AppState;

/// Starts the app; returns when the user quits it.
pub fn run() {
    init_logging();
    let commands = commands::builder();
    let result = tauri::Builder::default()
        // For the native dialogs that Rust opens (SPEC-UI §4.6): the
        // capability grants none of its commands to the frontend.
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(commands.invoke_handler())
        .register_asynchronous_uri_scheme_protocol(icons::SCHEME, serve_icon)
        .setup(|app| {
            let config_path = portolano_ops::locations::config_path(None)?;
            app.manage(AppState::new(config_path));
            if let Err(error) = sidecar::cli_path() {
                // The app still opens: only connections need the CLI.
                tracing::error!("{error}");
            }
            Ok(())
        })
        .run(tauri::generate_context!());
    if let Err(error) = result {
        tracing::error!("avvio dell'app non riuscito: {error}");
        std::process::exit(1);
    }
}

/// Answers a `portolano-icon://` request off the main thread.
fn serve_icon<R: tauri::Runtime>(
    context: UriSchemeContext<'_, R>,
    request: http::Request<Vec<u8>>,
    responder: tauri::UriSchemeResponder,
) {
    let icons = context
        .app_handle()
        .try_state::<AppState>()
        .and_then(|state| state.icons());
    let path = request.uri().path().to_owned();
    tauri::async_runtime::spawn_blocking(move || {
        responder.respond(icons::respond(icons.as_ref(), &path));
    });
}

/// Logs go to stderr: warnings and errors only, as the CLI without `-v`.
fn init_logging() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_max_level(tracing::Level::WARN)
        .with_target(false)
        .init();
}

#[cfg(test)]
mod tests;
