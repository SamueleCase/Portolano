//! Terminal sessions: the program of a plan in a PTY of the app, shown by
//! xterm.js (SPEC-UI §4.5, "SSH integrato").

use std::io::{self, Read, Write};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use portable_pty::{CommandBuilder, MasterPty, PtySize, native_pty_system};
use portolano_connectors::{CHILD_ENV_REMOVE, PreparedLaunch};

use super::channel::{FrameKind, SessionSink, frame};
use super::error::SessionError;
use super::model::{SessionKind, SessionState, TerminalSize};
use super::process::{Exit, WAIT_FAILED, spawn_thread, wait_for_exit};
use super::session::{Finish, Session};

/// The terminal xterm.js emulates.
const TERM: &str = "xterm-256color";

/// Size of the blocks of output sent to the tab.
const OUTPUT_CHUNK: usize = 32 * 1024;

/// How long the end of a program waits for the last output to be sent, so
/// that the final state comes after it. A process left in the background
/// that keeps the terminal open must not hold the state back for ever.
const OUTPUT_DRAIN: Duration = Duration::from_secs(1);

/// Exit code of `ssh` when the connection did not succeed.
const SSH_CONNECTION_FAILED: i32 = 255;

/// A program running in a terminal of the app.
pub(super) struct TerminalSession {
    master: Mutex<Box<dyn MasterPty + Send>>,
    writer: Mutex<Box<dyn Write + Send>>,
    close: Sender<()>,
}

impl TerminalSession {
    /// Starts the program of `launch` in a new terminal of `size`. Its
    /// output goes to `sink`, its last state to `finish`; the session keeps
    /// `launch` until the program exits, so that the askpass server answers
    /// for as long as `ssh` may ask.
    ///
    /// # Errors
    /// [`SessionError::StdinSecretInTerminal`] for a program that expects a
    /// secret on stdin, [`SessionError::Terminal`] if the terminal or the
    /// program cannot start, [`SessionError::Io`].
    pub(super) fn start(
        launch: PreparedLaunch,
        size: TerminalSize,
        sink: Arc<dyn SessionSink>,
        finish: Finish,
    ) -> Result<TerminalSession, SessionError> {
        // In a terminal, stdin is the keyboard of the tab: a secret written
        // there could be echoed to the frontend (DEC-030.3).
        if launch.stdin_secret().is_some() {
            return Err(SessionError::StdinSecretInTerminal);
        }
        let pair = native_pty_system()
            .openpty(pty_size(size))
            .map_err(terminal_error)?;
        let mut child = pair
            .slave
            .spawn_command(command(&launch))
            .map_err(terminal_error)?;
        // Only the program holds the terminal now: when it exits, reading
        // reaches the end of the output.
        drop(pair.slave);
        let reader = pair.master.try_clone_reader().map_err(terminal_error)?;
        let writer = pair.master.take_writer().map_err(terminal_error)?;

        let (output_done, output_drained) = mpsc::channel::<()>();
        let output_sink = Arc::clone(&sink);
        spawn_thread("output", move || {
            forward_output(reader, output_sink.as_ref());
            drop(output_done);
        })?;
        let (close, close_requests) = mpsc::channel();
        spawn_thread("wait", move || {
            let exit = wait_for_exit(child.as_mut(), &close_requests);
            let _ = output_drained.recv_timeout(OUTPUT_DRAIN);
            // Stops the askpass server and zeroes its secrets.
            drop(launch);
            finish(final_state(exit));
        })?;
        Ok(TerminalSession {
            master: Mutex::new(pair.master),
            writer: Mutex::new(writer),
            close,
        })
    }
}

impl Session for TerminalSession {
    fn kind(&self) -> SessionKind {
        SessionKind::Terminal
    }

    fn input(&self, bytes: &[u8]) -> Result<(), SessionError> {
        let mut writer = self.writer.lock().unwrap_or_else(PoisonError::into_inner);
        writer
            .write_all(bytes)
            .and_then(|()| writer.flush())
            .map_err(SessionError::io("invio dell'input al terminale"))
    }

    fn resize(&self, size: TerminalSize) -> Result<(), SessionError> {
        self.master
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .resize(pty_size(size))
            .map_err(terminal_error)
    }

    fn close(&self) {
        // The waiting thread may be gone already: nothing left to end.
        let _ = self.close.send(());
    }
}

/// The command that starts the program of `launch` in the terminal.
fn command(launch: &PreparedLaunch) -> CommandBuilder {
    let mut command = CommandBuilder::new(launch.program());
    command.args(launch.args());
    set_environment(&mut command, launch.env());
    command
}

/// The inherited environment without the variables no child may inherit
/// (DEC-023.1), plus those of the launch (askpass) and the terminal type.
fn set_environment(command: &mut CommandBuilder, launch_env: &[(String, String)]) {
    for name in CHILD_ENV_REMOVE {
        command.env_remove(name);
    }
    for (name, value) in launch_env {
        command.env(name, value);
    }
    command.env("TERM", TERM);
}

/// Sends the output of the terminal to `sink` until the terminal closes.
/// The first output marks the session as connected (ambiguity 11).
fn forward_output(mut reader: Box<dyn Read + Send>, sink: &dyn SessionSink) {
    let mut buffer = vec![0_u8; OUTPUT_CHUNK];
    let mut connected = false;
    loop {
        let read = match reader.read(&mut buffer) {
            Ok(0) => return,
            Ok(read) => read,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            // A closed terminal reads as an error (EIO) on some systems.
            Err(_) => return,
        };
        if !connected {
            connected = true;
            sink.state(&SessionState::Connected);
        }
        sink.output(frame(FrameKind::TerminalOutput, &buffer[..read]));
    }
}

/// The last state of a terminal session: `ssh` exits with 255 when the
/// connection does not succeed; a session closed by the user has ended,
/// whatever its program answered to the hang up.
fn final_state(exit: Exit) -> SessionState {
    match exit {
        Exit { code: None, .. } => SessionState::Failed {
            code: None,
            reason: WAIT_FAILED.to_owned(),
        },
        Exit {
            code: Some(code),
            closed: true,
        } => SessionState::Ended { code },
        Exit {
            code: Some(SSH_CONNECTION_FAILED),
            closed: false,
        } => SessionState::Failed {
            code: Some(SSH_CONNECTION_FAILED),
            reason: "connessione non riuscita".to_owned(),
        },
        Exit {
            code: Some(code),
            closed: false,
        } => SessionState::Ended { code },
    }
}

fn pty_size(size: TerminalSize) -> PtySize {
    PtySize {
        rows: size.rows,
        cols: size.cols,
        pixel_width: 0,
        pixel_height: 0,
    }
}

/// `portable_pty` reports its failures as `anyhow` errors.
fn terminal_error(error: impl std::fmt::Display) -> SessionError {
    SessionError::Terminal {
        reason: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use portolano_core::vault::MASTER_PASSWORD_ENV;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn variables_no_child_may_inherit_are_removed() {
        let mut command = CommandBuilder::new("/bin/sh");
        command.env(MASTER_PASSWORD_ENV, "inherited");

        set_environment(&mut command, &[]);

        for name in CHILD_ENV_REMOVE {
            assert_eq!(command.get_env(name), None, "{name}");
        }
    }

    #[test]
    fn launch_variables_and_terminal_type_are_set() {
        let mut command = CommandBuilder::new("/bin/sh");

        set_environment(
            &mut command,
            &[("SSH_ASKPASS_REQUIRE".to_owned(), "force".to_owned())],
        );

        assert_eq!(
            command.get_env("SSH_ASKPASS_REQUIRE"),
            Some("force".as_ref())
        );
        assert_eq!(command.get_env("TERM"), Some(TERM.as_ref()));
    }

    #[test]
    fn exit_255_is_a_failed_connection() {
        let state = final_state(Exit {
            code: Some(255),
            closed: false,
        });

        assert!(matches!(
            state,
            SessionState::Failed {
                code: Some(255),
                ..
            }
        ));
    }

    #[test]
    fn other_exit_codes_end_the_session() {
        let state = final_state(Exit {
            code: Some(130),
            closed: false,
        });

        assert_eq!(state, SessionState::Ended { code: 130 });
    }

    #[test]
    fn a_session_closed_by_the_user_has_ended_whatever_its_code() {
        let state = final_state(Exit {
            code: Some(255),
            closed: true,
        });

        assert_eq!(state, SessionState::Ended { code: 255 });
    }
}
