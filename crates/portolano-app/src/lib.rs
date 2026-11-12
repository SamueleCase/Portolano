//! Portolano desktop app (SPEC-UI): the Tauri backend of the React frontend.
//!
//! Commands are thin adapters over `portolano-ops`; their TypeScript
//! bindings are generated (`bindings.ts`). The frontend never receives a
//! secret (SPEC-UI §1.1).

mod bindings;
mod builtin_icons;
mod commands;
mod dialogs;
mod dto;
mod error;
mod events;
mod icons;
mod menu;
mod prefs;
mod search;
pub mod sessions;
mod sidecar;
mod state;
mod watch;
mod window;

use tauri::{App, AppHandle, Manager, RunEvent, Runtime, UriSchemeContext, WindowEvent, http};

pub use bindings::export_bindings;

use crate::prefs::UI_PREFERENCES_FILE_NAME;
use crate::state::{AppPaths, AppState};

/// Starts the app; returns when the user quits it.
pub fn run() {
    init_logging();
    let commands = commands::builder();
    let invoke_handler = commands.invoke_handler();
    let app = tauri::Builder::default()
        // For the native dialogs that Rust opens (SPEC-UI §4.6): the
        // capability grants none of its commands to the frontend.
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(invoke_handler)
        .register_asynchronous_uri_scheme_protocol(icons::SCHEME, serve_icon)
        .on_menu_event(|app, event| menu::on_event(app, &event))
        .on_window_event(|window, event| {
            if matches!(event, WindowEvent::CloseRequested { .. })
                && let Some(main) = window.app_handle().get_webview_window(window::MAIN)
            {
                window::remember(&main);
            }
        })
        .setup(move |app| {
            commands.mount_events(app);
            setup(app)
        })
        .build(tauri::generate_context!());
    match app {
        Ok(app) => app.run(on_run_event),
        Err(error) => {
            tracing::error!("avvio dell'app non riuscito: {error}");
            std::process::exit(1);
        }
    }
}

/// The state, the menu and the window, before the frontend starts.
fn setup<R: Runtime>(app: &mut App<R>) -> Result<(), Box<dyn std::error::Error>> {
    let config = portolano_ops::locations::config_path(None)?;
    let ui_preferences = portolano_core::config::data_dir()?.join(UI_PREFERENCES_FILE_NAME);
    let state = AppState::new(AppPaths {
        config,
        ui_preferences,
    });
    let preferences = state.ui_preferences();
    app.manage(state);
    menu::install(app.handle(), &preferences.recent_documents)?;
    if let (Some(main), Some(geometry)) = (app.get_webview_window(window::MAIN), preferences.window)
    {
        window::restore(&main, geometry);
    }
    if let Err(error) = sidecar::cli_path() {
        // The app still opens: only connections need the CLI.
        tracing::error!("{error}");
    }
    Ok(())
}

/// Finishes the deletions that could still be undone when the app quits,
/// so that their secrets do not stay in the vault.
fn on_run_event<R: Runtime>(app: &AppHandle<R>, event: RunEvent) {
    if let RunEvent::Exit = event {
        app.state::<AppState>().purge_pending_deletes();
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
