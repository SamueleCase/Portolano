//! The open sessions of the app, whatever their kind (SPEC-UI §4.5).

use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use portolano_connectors::PreparedLaunch;

use super::channel::SessionSink;
use super::error::SessionError;
use super::external::ExternalSession;
use super::model::{
    SessionEnded, SessionId, SessionKind, SessionOpened, SessionState, TerminalSize,
};
use super::rdp::RdpSession;
use super::session::{Finish, Session};
use super::terminal::TerminalSession;

/// Told about every session whose program is over (the `session-ended`
/// event of the app).
type EndedListener = dyn Fn(SessionEnded) + Send + Sync;

/// Opens, drives and closes sessions. A session stays known until it is
/// closed, also after its program is over, so that closing its tab is
/// always valid.
pub struct SessionManager {
    sessions: Mutex<HashMap<SessionId, Arc<dyn Session>>>,
    on_ended: Arc<EndedListener>,
}

impl SessionManager {
    /// A manager without sessions; `on_ended` hears of each session whose
    /// program is over.
    pub fn new(on_ended: impl Fn(SessionEnded) + Send + Sync + 'static) -> SessionManager {
        SessionManager {
            sessions: Mutex::new(HashMap::new()),
            on_ended: Arc::new(on_ended),
        }
    }

    /// Opens a session of `kind` (see [`super::session_kind`]) that runs
    /// `launch` and reports to `sink`: first [`SessionState::Connecting`],
    /// then its output and states. `size` is the size of the terminal of a
    /// terminal session; the other kinds ignore it.
    ///
    /// # Errors
    /// As the start of each kind: [`SessionError::StdinSecretInTerminal`],
    /// [`SessionError::Terminal`], [`SessionError::Spawn`],
    /// [`SessionError::Io`].
    pub fn open(
        &self,
        kind: SessionKind,
        launch: PreparedLaunch,
        size: TerminalSize,
        sink: Arc<dyn SessionSink>,
    ) -> Result<SessionOpened, SessionError> {
        let id = SessionId::new();
        sink.state(&SessionState::Connecting);
        let finish = self.finish(id, Arc::clone(&sink));
        let session: Arc<dyn Session> = match kind {
            SessionKind::Terminal => Arc::new(TerminalSession::start(launch, size, sink, finish)?),
            SessionKind::RdpExternal => {
                let log = launch.log_path().map(Path::to_path_buf);
                Arc::new(RdpSession::start(launch, log, sink, finish)?)
            }
            SessionKind::External => Arc::new(ExternalSession::start(launch, sink, finish)?),
        };
        self.lock().insert(id, session);
        Ok(SessionOpened { id, kind })
    }

    /// Sends what the user typed to session `id`.
    ///
    /// # Errors
    /// [`SessionError::NotFound`], [`SessionError::Unsupported`] without a
    /// terminal, [`SessionError::Io`].
    pub fn input(&self, id: SessionId, bytes: &[u8]) -> Result<(), SessionError> {
        self.get(id)?.input(bytes)
    }

    /// Resizes the terminal of session `id`.
    ///
    /// # Errors
    /// [`SessionError::NotFound`], [`SessionError::Unsupported`] without a
    /// terminal, [`SessionError::Terminal`].
    pub fn resize(&self, id: SessionId, size: TerminalSize) -> Result<(), SessionError> {
        self.get(id)?.resize(size)
    }

    /// Closes session `id`: its program, if still running, is ended and its
    /// last state follows on its sink.
    ///
    /// # Errors
    /// [`SessionError::NotFound`].
    pub fn close(&self, id: SessionId) -> Result<(), SessionError> {
        let session = self
            .lock()
            .remove(&id)
            .ok_or(SessionError::NotFound { id })?;
        session.close();
        Ok(())
    }

    /// Ends every session, when the app quits.
    pub fn close_all(&self) {
        for (_, session) in self.lock().drain() {
            session.close();
        }
    }

    fn get(&self, id: SessionId) -> Result<Arc<dyn Session>, SessionError> {
        self.lock()
            .get(&id)
            .cloned()
            .ok_or(SessionError::NotFound { id })
    }

    /// What a session does when its program is over: its last state to its
    /// sink, then the news to the listener.
    fn finish(&self, id: SessionId, sink: Arc<dyn SessionSink>) -> Finish {
        let on_ended = Arc::clone(&self.on_ended);
        Box::new(move |state| {
            sink.state(&state);
            on_ended(SessionEnded { id, state });
        })
    }

    /// The sessions. A panic while the lock was held leaves the map
    /// consistent: each change is a single insert or removal.
    fn lock(&self) -> MutexGuard<'_, HashMap<SessionId, Arc<dyn Session>>> {
        self.sessions.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl fmt::Debug for SessionManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionManager")
            .field("sessions", &self.lock().len())
            .finish_non_exhaustive()
    }
}
