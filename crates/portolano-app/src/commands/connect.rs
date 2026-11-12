//! The connection commands that do not open a session (SPEC-UI §4.3
//! "Connessione"): the plan shown by "Mostra comando", and whether the login
//! must be asked first. Sessions are another module.

use portolano_connectors::{ConnectOptions, Environment, render};
use portolano_ops::OpsError;
use portolano_ops::connect::{LoginRequest, login_request as request_of, plan_connection};
use serde::Serialize;
use tauri::{AppHandle, Runtime};
use uuid::Uuid;

use super::blocking;
use crate::error::IpcError;
use crate::sidecar;
use crate::state::AppState;

/// The launch plan of a connection, as `connect --print` shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
pub struct ConnectPreview {
    /// The command lines, with secrets as `<vault:…>` references.
    pub command: String,
    /// Warnings of the plan, in Italian.
    pub warnings: Vec<String>,
}

/// The plan of connection `id` ("Mostra comando"). With a credential "ask
/// at launch" the plan has no login, as before the login is typed.
#[tauri::command]
#[specta::specta]
pub async fn connect_preview<R: Runtime>(
    app: AppHandle<R>,
    id: Uuid,
) -> Result<ConnectPreview, IpcError> {
    blocking(app, move |_, state| preview(state, id)).await
}

/// The login to ask before connecting to `id`, if its credential is "ask at
/// launch" (DEC-016.3).
#[tauri::command]
#[specta::specta]
pub async fn login_request<R: Runtime>(
    app: AppHandle<R>,
    id: Uuid,
) -> Result<Option<LoginRequest>, IpcError> {
    blocking(app, move |_, state| {
        state.with_document(|open| Ok(request_of(open.document(), id)?))
    })
    .await
}

pub(crate) fn preview(state: &AppState, id: Uuid) -> Result<ConnectPreview, IpcError> {
    let config = state.config()?;
    let mut environment = Environment::discover(&config).map_err(OpsError::from)?;
    // The askpass program and the new window are the CLI next to the app
    // (SPEC-UI §4.10); without it the plan still shows, with the app itself.
    if let Ok(cli) = sidecar::cli_path() {
        environment.self_exe = cli;
    }
    state.with_document(|open| {
        let plan = plan_connection(
            open.document(),
            &config,
            &environment,
            state.protocols(),
            id,
            &ConnectOptions::default(),
        )?;
        Ok(ConnectPreview {
            command: render(&plan),
            warnings: plan.warnings,
        })
    })
}

#[cfg(test)]
mod tests;
