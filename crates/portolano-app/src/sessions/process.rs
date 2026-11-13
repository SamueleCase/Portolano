//! Following the program of a session until it exits.

use std::io;
use std::process::{Child, ExitStatus};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SendError, Sender};
use std::time::Duration;

use portable_pty::ChildKiller;

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
fn wait_for_exit(child: &mut Child, close: &Receiver<()>) -> Exit {
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
                stop_gently(child);
                return Exit {
                    code: child.wait().ok().map(|status| exit_code(&status)),
                    closed: true,
                };
            }
        }
    }
}

/// SIGHUP first, as a terminal that closes, then SIGKILL if the program is
/// still there after a short grace (`portable_pty::ChildKiller`).
fn stop_gently(child: &mut Child) {
    let _ = ChildKiller::kill(child);
}

/// The exit code, as the CLI reports it: `128 + n` for a program killed by
/// signal `n` (DEC-033.5).
fn exit_code(status: &ExitStatus) -> i32 {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return 128 + signal;
        }
    }
    status.code().unwrap_or(1)
}

/// A started program, followed by its waiting thread. The program of a
/// terminal is one too: `portable_pty` starts it as a standard child.
pub(super) type Program = Child;

/// Starts the thread that will follow a program, before the program itself:
/// once the program runs, handing it over cannot fail for want of a thread,
/// so no program is left running unattended. The thread waits for the
/// program and what goes with it (`T`, such as its `PreparedLaunch`) on the
/// returned sender, follows it until it exits or a close request arrives on
/// `close`, then calls `then`. If the sender is dropped instead, the program
/// never started and the thread ends.
///
/// # Errors
/// [`SessionError::Io`] if the system refuses a new thread.
pub(super) fn start_waiter<T: Send + 'static>(
    close: Receiver<()>,
    then: impl FnOnce(Exit, T) + Send + 'static,
) -> Result<Sender<(Program, T)>, SessionError> {
    let (sender, receiver) = mpsc::channel::<(Program, T)>();
    spawn_thread("wait", move || {
        let Ok((mut program, with)) = receiver.recv() else {
            return;
        };
        let exit = wait_for_exit(&mut program, &close);
        then(exit, with);
    })?;
    Ok(sender)
}

/// Hands `program` over to its waiting thread (see [`start_waiter`]); if the
/// thread is gone, ends the program instead.
///
/// # Errors
/// [`SessionError::Io`] if the waiting thread is gone.
pub(super) fn hand_over<T>(
    waiter: &Sender<(Program, T)>,
    program: Program,
    with: T,
) -> Result<(), SessionError> {
    waiter
        .send((program, with))
        .map_err(|SendError((mut program, _))| {
            stop(&mut program);
            SessionError::Io {
                context: "avvio del controllo della sessione",
                source: io::Error::other("il thread di attesa è terminato"),
            }
        })
}

/// Ends a program that cannot be followed, and reaps it.
pub(super) fn stop(program: &mut Child) {
    stop_gently(program);
    let _ = program.wait();
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
