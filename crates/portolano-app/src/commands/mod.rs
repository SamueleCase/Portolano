//! The commands of the app (SPEC-UI §4.3). Each one is also listed in
//! `build.rs`, which generates its permission.

mod document;

use tauri::Runtime;
use tauri_specta::{Builder, ErrorHandlingMode, collect_commands};

/// The commands, with the shape of their TypeScript bindings.
///
/// Errors reach the frontend as values, `{ status: "error", error }`, so
/// that every caller has to handle the [`IpcError`](crate::error::IpcError).
pub fn builder<R: Runtime>() -> Builder<R> {
    Builder::new()
        .error_handling(ErrorHandlingMode::Result)
        .commands(collect_commands![document::app_status])
}
