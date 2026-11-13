//! The open sessions of the app, whatever their kind (SPEC-UI §4.5).

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError, RwLock};
use std::time::{Duration, Instant};

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

/// How long [`SessionManager::close_all`] waits for the programs to end:
/// a terminal ends at once, FreeRDP within the grace period of its kill.
const CLOSE_ALL_WAIT: Duration = Duration::from_secs(2);

/// Opens, drives and closes sessions. A session stays known until it is
/// closed, also after its program is over, so that closing its tab is
/// always valid.
#[derive(Default)]
pub struct SessionManager {
    sessions: Mutex<HashMap<SessionId, Arc<dyn Session>>>,
    running: Arc<Running>,
    on_ended: Arc<RwLock<Option<Arc<EndedListener>>>>,
    /// Set by [`SessionManager::close_all`]: the app is quitting.
    closing: AtomicBool,
}

impl SessionManager {
    /// Makes `listener` hear of each session whose program is over, from
    /// then on. The app sets it once it can emit events, after the state
    /// that holds the manager exists.
    pub fn on_ended(&self, listener: impl Fn(SessionEnded) + Send + Sync + 'static) {
        *self
            .on_ended
            .write()
            .unwrap_or_else(PoisonError::into_inner) = Some(Arc::new(listener));
    }

    /// Opens a session of `kind` (see [`super::session_kind`]) that runs
    /// `launch` and reports to `sink`: first [`SessionState::Connecting`],
    /// then its output and states. `size` is the size of the terminal of a
    /// terminal session; the other kinds ignore it.
    ///
    /// # Errors
    /// [`SessionError::Closing`] once the app quits; as the start of each
    /// kind: [`SessionError::StdinSecretInTerminal`],
    /// [`SessionError::Terminal`], [`SessionError::Spawn`],
    /// [`SessionError::Io`].
    pub fn open(
        &self,
        kind: SessionKind,
        launch: PreparedLaunch,
        size: TerminalSize,
        sink: Arc<dyn SessionSink>,
    ) -> Result<SessionOpened, SessionError> {
        if self.closing.load(Ordering::SeqCst) {
            return Err(SessionError::Closing);
        }
        let id = SessionId::new();
        sink.state(&SessionState::Connecting);
        let finish = self.finish(id, Arc::clone(&sink));
        // Windows App leaves no program to wait for.
        if kind != SessionKind::External {
            self.running.add(id);
        }
        match start(kind, launch, size, sink, finish) {
            Ok(session) => {
                let mut sessions = self.lock();
                // `close_all` may have run while the session started: it set
                // the flag before taking this lock, so either it sees this
                // session or this sees the flag.
                if self.closing.load(Ordering::SeqCst) {
                    drop(sessions);
                    session.close();
                    return Err(SessionError::Closing);
                }
                sessions.insert(id, session);
                Ok(SessionOpened { id, kind })
            }
            Err(error) => {
                self.running.remove(id);
                Err(error)
            }
        }
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

    /// Ends every session, when the app quits, and waits a little for their
    /// programs to end: FreeRDP lives in its own window and would outlive
    /// the app. Returns whether they all ended in time.
    pub fn close_all(&self) -> bool {
        // No session may start after the others were asked to end.
        self.closing.store(true, Ordering::SeqCst);
        for (_, session) in self.lock().drain() {
            session.close();
        }
        self.running.wait_until_none(CLOSE_ALL_WAIT)
    }

    fn get(&self, id: SessionId) -> Result<Arc<dyn Session>, SessionError> {
        self.lock()
            .get(&id)
            .cloned()
            .ok_or(SessionError::NotFound { id })
    }

    /// What a session does when its program is over: its last state to its
    /// sink, the news to the listener, and it no longer runs.
    fn finish(&self, id: SessionId, sink: Arc<dyn SessionSink>) -> Finish {
        let on_ended = Arc::clone(&self.on_ended);
        let running = Arc::clone(&self.running);
        Box::new(move |state| {
            sink.state(&state);
            let listener = on_ended
                .read()
                .unwrap_or_else(PoisonError::into_inner)
                .clone();
            if let Some(listener) = listener {
                listener(SessionEnded { id, state });
            }
            running.remove(id);
        })
    }

    /// The sessions. A panic while the lock was held leaves the map
    /// consistent: each change is a single insert or removal.
    fn lock(&self) -> MutexGuard<'_, HashMap<SessionId, Arc<dyn Session>>> {
        self.sessions.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Starts a session of `kind`.
fn start(
    kind: SessionKind,
    launch: PreparedLaunch,
    size: TerminalSize,
    sink: Arc<dyn SessionSink>,
    finish: Finish,
) -> Result<Arc<dyn Session>, SessionError> {
    Ok(match kind {
        SessionKind::Terminal => Arc::new(TerminalSession::start(launch, size, sink, finish)?),
        SessionKind::RdpExternal => {
            let log = launch.log_path().map(Path::to_path_buf);
            Arc::new(RdpSession::start(launch, log, sink, finish)?)
        }
        SessionKind::External => Arc::new(ExternalSession::start(launch, sink, finish)?),
    })
}

/// The sessions whose program still runs, for [`SessionManager::close_all`]
/// to wait on.
#[derive(Default)]
struct Running {
    ids: Mutex<HashSet<SessionId>>,
    changed: Condvar,
}

impl Running {
    fn add(&self, id: SessionId) {
        self.lock().insert(id);
    }

    fn remove(&self, id: SessionId) {
        self.lock().remove(&id);
        self.changed.notify_all();
    }

    /// Waits at most `timeout` for no program to run; whether none does.
    fn wait_until_none(&self, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        let mut ids = self.lock();
        while !ids.is_empty() {
            let left = deadline.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return false;
            }
            ids = self
                .changed
                .wait_timeout(ids, left)
                .unwrap_or_else(PoisonError::into_inner)
                .0;
        }
        true
    }

    /// A panic while the lock was held leaves the set consistent: each
    /// change is a single insert or removal.
    fn lock(&self) -> MutexGuard<'_, HashSet<SessionId>> {
        self.ids.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl fmt::Debug for SessionManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionManager")
            .field("sessions", &self.lock().len())
            .finish_non_exhaustive()
    }
}
