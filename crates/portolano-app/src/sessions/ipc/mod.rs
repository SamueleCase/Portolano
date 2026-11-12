//! The session commands (SPEC-UI §4.3 "Connessione", DEC-030.5): the plan
//! of `portolano-ops`, prepared by the connectors, run by the
//! [`SessionManager`](crate::sessions::SessionManager) of the state.

mod wire;

use std::sync::Arc;

use portolano_connectors::{ConnectOptions, Environment, LaunchPlan, prepare};
use portolano_core::AppConfig;
use portolano_ops::OpsError;
use portolano_ops::connect::plan_connection;
use portolano_ops::vault::SessionVault;
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, Runtime};
use uuid::Uuid;

use wire::{ChannelSink, LoginInput, OutputFrame, SessionOptionsInput};

use crate::error::IpcError;
use crate::sessions::{
    SessionId, SessionOpened, SessionSink, SessionState, TerminalSize, environment,
    open_in_new_window, session_kind,
};
use crate::state::AppState;

/// Opens a session on connection `node`. `login` answers a credential "ask
/// at launch" (see `login_request`); `size` is the size of the terminal of
/// a session that has one. The output goes to `output`, the states to
/// `states`, starting with "connecting".
#[tauri::command]
#[specta::specta]
pub async fn open_session<R: Runtime>(
    app: AppHandle<R>,
    node: Uuid,
    options: SessionOptionsInput,
    login: Option<LoginInput>,
    size: TerminalSize,
    output: Channel<OutputFrame>,
    states: Channel<SessionState>,
) -> Result<SessionOpened, IpcError> {
    let sink = Arc::new(ChannelSink { output, states });
    blocking(app, move |state| {
        open(state, node, &options, login, size, sink)
    })
    .await
}

/// Sends what the user typed to session `id`.
#[tauri::command]
#[specta::specta]
pub async fn session_input<R: Runtime>(
    app: AppHandle<R>,
    id: SessionId,
    data: String,
) -> Result<(), IpcError> {
    blocking(app, move |state| {
        Ok(state.sessions().input(id, data.as_bytes())?)
    })
    .await
}

/// Resizes the terminal of session `id`.
#[tauri::command]
#[specta::specta]
pub async fn session_resize<R: Runtime>(
    app: AppHandle<R>,
    id: SessionId,
    size: TerminalSize,
) -> Result<(), IpcError> {
    blocking(app, move |state| Ok(state.sessions().resize(id, size)?)).await
}

/// Closes session `id`: its program ends, its last state follows on its
/// channel.
#[tauri::command]
#[specta::specta]
pub async fn session_close<R: Runtime>(app: AppHandle<R>, id: SessionId) -> Result<(), IpcError> {
    blocking(app, move |state| Ok(state.sessions().close(id)?)).await
}

/// Opens connection `node` in a new window of the configured terminal
/// ("Apri in terminale esterno"): the CLI shipped with the app reopens the
/// same document with the same configuration, and asks there what it needs.
#[tauri::command]
#[specta::specta]
pub async fn open_in_terminal<R: Runtime>(app: AppHandle<R>, node: Uuid) -> Result<(), IpcError> {
    blocking(app, move |state| open_new_window(state, node)).await
}

/// Opens a session on `node` reporting to `sink`.
///
/// # Errors
/// No open document, a plan that cannot be made or prepared (locked vault,
/// missing client, …), a session that cannot start.
pub(crate) fn open(
    state: &AppState,
    node: Uuid,
    options: &SessionOptionsInput,
    login: Option<LoginInput>,
    size: TerminalSize,
    sink: Arc<dyn SessionSink>,
) -> Result<SessionOpened, IpcError> {
    let config = state.config()?;
    let environment = environment(&config)?;
    let vault = state.vault()?;
    // The typed password lives in this vault of the session only, and
    // leaves memory once the launch is prepared.
    let session = SessionVault::new(vault.as_ref());
    let session_credential = login
        .map(|login| session.session_credential(login.into()))
        .transpose()
        .map_err(OpsError::from)?;
    let connect = ConnectOptions {
        rdp_client: options.rdp_client.map(Into::into),
        session_credential,
        ..ConnectOptions::default()
    };
    let plan = plan_of(state, &config, node, &environment, &connect)?;
    let kind = session_kind(&plan)?;
    let launch = prepare(&plan, &session, &config).map_err(OpsError::from)?;
    Ok(state.sessions().open(kind, launch, size, sink)?)
}

/// Opens `node` in a new terminal window.
///
/// # Errors
/// No open document, a plan that cannot be made, a launcher that fails.
pub(crate) fn open_new_window(state: &AppState, node: Uuid) -> Result<(), IpcError> {
    let config = state.config()?;
    let environment = environment(&config)?;
    let document = state.document_path().ok_or_else(IpcError::no_document)?;
    // The same document and configuration (DEC-004, DEC-022.8); the vault
    // comes from the configuration, and its master password is never
    // passed.
    let connect = ConnectOptions {
        new_window: true,
        relaunch_args: vec![
            "--file".to_owned(),
            document.display().to_string(),
            "--config".to_owned(),
            state.paths().config.display().to_string(),
        ],
        ..ConnectOptions::default()
    };
    let plan = plan_of(state, &config, node, &environment, &connect)?;
    let vault = state.vault()?;
    let launch = prepare(&plan, vault.as_ref(), &config).map_err(OpsError::from)?;
    Ok(open_in_new_window(launch)?)
}

/// The plan of `node` in the open document.
fn plan_of(
    state: &AppState,
    config: &AppConfig,
    node: Uuid,
    environment: &Environment,
    options: &ConnectOptions,
) -> Result<LaunchPlan, IpcError> {
    state.with_document(|open| {
        Ok(plan_connection(
            open.document(),
            config,
            environment,
            state.protocols(),
            node,
            options,
        )?)
    })
}

/// Runs `work` with the state on a thread that may block: preparing a plan
/// reads the vault, writing to a terminal may wait for it.
async fn blocking<R: Runtime, T: Send + 'static>(
    app: AppHandle<R>,
    work: impl FnOnce(&AppState) -> Result<T, IpcError> + Send + 'static,
) -> Result<T, IpcError> {
    tauri::async_runtime::spawn_blocking(move || work(&app.state::<AppState>()))
        .await
        .map_err(|error| IpcError::internal(&format!("comando interrotto: {error}")))?
}

#[cfg(test)]
mod tests;
