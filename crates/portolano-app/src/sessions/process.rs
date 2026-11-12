//! Following the program of a session until it exits.

use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;

use portable_pty::{Child, ExitStatus};

use super::error::SessionError;

/// How often the waiting thread looks at the program between close
/// requests: the delay of a tab in noticing that its program is over.
const POLL: Duration = Duration::from_millis(50);

/// How a program ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Exit {
    /// Its exit code; `None` if it could not be read.
    pub(super) code: Option<i32>,
    /// Whether the user closed the session.
    pub(super) closed: bool,
}

/// Waits for `child` to exit. A request on `close`, or the end of the
/// session that holds its sender, kills it.
///
/// Waiting and killing happen on this one thread, so the process is never
/// signalled after it has been reaped, when its pid may belong to another.
pub(super) fn wait_for_exit(child: &mut (dyn Child + Send + Sync), close: &Receiver<()>) -> Exit {
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return Exit {
                    code: Some(exit_code(&status)),
                    closed: false,
                };
            }
            Ok(None) => {}
            Err(_) => {
                return Exit {
                    code: None,
                    closed: false,
                };
            }
        }
        match close.recv_timeout(POLL) {
            Err(RecvTimeoutError::Timeout) => {}
            Ok(()) | Err(RecvTimeoutError::Disconnected) => {
                // SIGHUP first, as a terminal that closes; then SIGKILL if
                // the program is still there (`portable_pty::ChildKiller`).
                let _ = child.kill();
                return Exit {
                    code: child.wait().ok().map(|status| exit_code(&status)),
                    closed: true,
                };
            }
        }
    }
}

/// The exit code; a program killed by a signal counts as 1, as
/// `portable_pty` reports it.
fn exit_code(status: &ExitStatus) -> i32 {
    i32::try_from(status.exit_code()).unwrap_or(i32::MAX)
}

/// Starts a thread of the session called `name`; it is never joined; it ends
/// with the program it follows.
///
/// # Errors
/// [`SessionError::Io`] if the system refuses a new thread.
pub(super) fn spawn_thread(
    name: &str,
    body: impl FnOnce() + Send + 'static,
) -> Result<(), SessionError> {
    std::thread::Builder::new()
        .name(format!("portolano-session-{name}"))
        .spawn(body)
        .map(drop)
        .map_err(SessionError::io("avvio di un thread della sessione"))
}

/// The reason of [`super::SessionState::Failed`] when the exit code of the
/// program could not be read.
pub(super) const WAIT_FAILED: &str = "impossibile seguire il programma della sessione";
