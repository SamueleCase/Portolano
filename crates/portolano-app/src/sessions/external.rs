//! Connections handed to another application: Windows App (SPEC-UI §4.5),
//! and new terminal windows, which are not sessions.

use std::sync::Arc;

use portolano_connectors::PreparedLaunch;

use super::channel::SessionSink;
use super::error::SessionError;
use super::model::{SessionKind, SessionState};
use super::process::spawn_thread;
use super::session::{Finish, Session};

/// A connection opened by Windows App: the launcher (`open -a`) returns at
/// once and there is no process left to follow, so the tab only says that
/// the application has it.
pub(super) struct ExternalSession;

impl ExternalSession {
    /// Runs the launcher of `launch` off the calling thread: success marks
    /// the session as connected, a failure goes to `finish`.
    ///
    /// # Errors
    /// [`SessionError::Io`] if its thread cannot start.
    pub(super) fn start(
        launch: PreparedLaunch,
        sink: Arc<dyn SessionSink>,
        finish: Finish,
    ) -> Result<ExternalSession, SessionError> {
        spawn_thread("external", move || match launch.launch() {
            Ok(0) => sink.state(&SessionState::Connected),
            Ok(code) => finish(SessionState::Failed {
                code: Some(code),
                reason: format!("l'applicazione non si è aperta (codice {code})"),
            }),
            Err(error) => finish(SessionState::Failed {
                code: None,
                reason: error.to_string(),
            }),
        })?;
        Ok(ExternalSession)
    }
}

impl Session for ExternalSession {
    fn kind(&self) -> SessionKind {
        SessionKind::External
    }

    /// The application owns the connection: closing the tab leaves it open.
    fn close(&self) {}
}

/// Opens the connection in a new window of the configured terminal: the
/// plan of `--new-window`, whose command is the CLI shipped with the app
/// (`Environment::self_exe`). The window lives on its own, so this is not a
/// session.
///
/// # Errors
/// [`SessionError::Connect`] if the launcher cannot start,
/// [`SessionError::NewWindow`] if it fails.
pub fn open_in_new_window(launch: PreparedLaunch) -> Result<(), SessionError> {
    match launch.launch()? {
        0 => Ok(()),
        code => Err(SessionError::NewWindow { code }),
    }
}
