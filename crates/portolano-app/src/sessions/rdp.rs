//! RDP sessions in the window of FreeRDP (SPEC-UI §4.5, "RDP"): the app
//! starts FreeRDP without detaching it and follows its process, so the tab
//! can show that it runs, when it ended and, on failure, its log.

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::mpsc::{self, Sender};

use portolano_connectors::{CHILD_ENV_REMOVE, PreparedLaunch};
use portolano_core::Secret;

use super::channel::SessionSink;
use super::error::SessionError;
use super::model::{SessionKind, SessionState};
use super::process::{Exit, WAIT_FAILED, spawn_thread, wait_for_exit};
use super::session::{Finish, Session};

/// The end of the log shown by "Mostra log": FreeRDP logs warnings and
/// errors only, and the last ones explain the failure.
const LOG_TAIL_BYTES: u64 = 4 * 1024;

/// FreeRDP running in its own window.
pub(super) struct RdpSession {
    close: Sender<()>,
}

impl RdpSession {
    /// Starts the program of `launch` (FreeRDP), writes its stdin secret and
    /// follows it; its stderr goes to `log`, the log that the connectors
    /// create for a detached FreeRDP (`PreparedLaunch::log_path`). Its last
    /// state goes to `finish`.
    ///
    /// # Errors
    /// [`SessionError::Spawn`] if FreeRDP cannot start, [`SessionError::Io`]
    /// if its log cannot be opened or its stdin written.
    pub(super) fn start(
        launch: PreparedLaunch,
        log: Option<PathBuf>,
        sink: Arc<dyn SessionSink>,
        finish: Finish,
    ) -> Result<RdpSession, SessionError> {
        let mut child = command(&launch, log.as_deref())?
            .spawn()
            .map_err(|source| SessionError::Spawn {
                program: launch.program().to_path_buf(),
                source,
            })?;
        if let (Some(secret), Some(mut stdin)) = (launch.stdin_secret(), child.stdin.take())
            && let Err(error) = write_secret(&mut stdin, secret)
        {
            let _ = child.kill();
            let _ = child.wait();
            return Err(SessionError::Io {
                context: "scrittura della password sullo stdin di FreeRDP",
                source: error,
            });
        }
        sink.state(&SessionState::Connected);

        let (close, close_requests) = mpsc::channel();
        let mut child: Box<dyn portable_pty::Child + Send + Sync> = Box::new(child);
        spawn_thread("wait", move || {
            let exit = wait_for_exit(child.as_mut(), &close_requests);
            drop(launch);
            finish(final_state(exit, log.as_deref()));
        })?;
        Ok(RdpSession { close })
    }
}

impl Session for RdpSession {
    fn kind(&self) -> SessionKind {
        SessionKind::RdpExternal
    }

    fn close(&self) {
        let _ = self.close.send(());
    }
}

/// The command that starts FreeRDP: no terminal, the password (if any) on a
/// pipe, its messages in the log of the launch.
fn command(launch: &PreparedLaunch, log: Option<&Path>) -> Result<Command, SessionError> {
    let mut command = Command::new(launch.program());
    command.args(launch.args());
    for name in CHILD_ENV_REMOVE {
        command.env_remove(name);
    }
    command.envs(launch.env().iter().map(|(name, value)| (name, value)));
    let stdin = if launch.stdin_secret().is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    };
    let stderr = match log {
        Some(path) => Stdio::from(
            OpenOptions::new()
                .append(true)
                .open(path)
                .map_err(SessionError::io("apertura del log di FreeRDP"))?,
        ),
        None => Stdio::null(),
    };
    command.stdin(stdin).stdout(Stdio::null()).stderr(stderr);
    Ok(command)
}

/// Writes `secret` and a newline, as `PreparedLaunch::launch` does for the
/// CLI; dropping `stdin` afterwards closes it. A program that exits without
/// reading is not an error: its exit code tells the story.
fn write_secret(stdin: &mut impl Write, secret: &Secret) -> io::Result<()> {
    match stdin
        .write_all(secret.expose().as_bytes())
        .and_then(|()| stdin.write_all(b"\n"))
    {
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        other => other,
    }
}

/// The last state of FreeRDP: a failure carries the end of its log.
fn final_state(exit: Exit, log: Option<&Path>) -> SessionState {
    match exit {
        Exit { code: None, .. } => SessionState::Failed {
            code: None,
            reason: WAIT_FAILED.to_owned(),
        },
        Exit {
            code: Some(code),
            closed,
        } if closed || code == 0 => SessionState::Ended { code },
        Exit {
            code: Some(code), ..
        } => SessionState::Failed {
            code: Some(code),
            reason: log
                .and_then(|path| log_tail(path).ok())
                .filter(|tail| !tail.is_empty())
                .unwrap_or_else(|| format!("FreeRDP è uscito con codice {code}")),
        },
    }
}

/// The last [`LOG_TAIL_BYTES`] of the log at `path`, from the first whole
/// line, as text.
fn log_tail(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let length = file.metadata()?.len();
    let start = length.saturating_sub(LOG_TAIL_BYTES);
    file.seek(SeekFrom::Start(start))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    let text = String::from_utf8_lossy(&bytes);
    let text = match text.split_once('\n') {
        Some((_, whole_lines)) if start > 0 => whole_lines,
        _ => &text,
    };
    Ok(text.trim().to_owned())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use tempfile::TempDir;

    use super::*;

    fn log_in(dir: &TempDir, contents: &[u8]) -> PathBuf {
        let path = dir.path().join("freerdp.log");
        std::fs::write(&path, contents).unwrap();
        path
    }

    fn failed(code: i32) -> Exit {
        Exit {
            code: Some(code),
            closed: false,
        }
    }

    #[test]
    fn a_clean_exit_ends_the_session() {
        assert_eq!(
            final_state(failed(0), None),
            SessionState::Ended { code: 0 }
        );
    }

    #[test]
    fn a_failure_shows_the_end_of_the_log() {
        let dir = TempDir::new().unwrap();
        let log = log_in(&dir, b"[ERROR] connessione rifiutata da 192.0.2.10\n");

        let state = final_state(failed(131), Some(&log));

        assert_eq!(
            state,
            SessionState::Failed {
                code: Some(131),
                reason: "[ERROR] connessione rifiutata da 192.0.2.10".to_owned()
            }
        );
    }

    #[test]
    fn a_failure_without_log_names_the_exit_code() {
        let state = final_state(failed(1), None);

        assert!(
            matches!(&state, SessionState::Failed { reason, .. } if reason.contains("codice 1")),
            "{state:?}"
        );
    }

    #[test]
    fn a_long_log_is_cut_to_its_last_whole_lines() {
        let dir = TempDir::new().unwrap();
        let mut contents = "riga vecchia\n".repeat(1000).into_bytes();
        contents.extend_from_slice(b"ultima riga\n");
        let log = log_in(&dir, &contents);

        let tail = log_tail(&log).unwrap();

        assert!(tail.len() <= LOG_TAIL_BYTES as usize);
        assert!(tail.starts_with("riga vecchia"), "{tail:?}");
        assert!(tail.ends_with("ultima riga"), "{tail:?}");
    }

    #[test]
    fn a_session_closed_by_the_user_has_ended() {
        let exit = Exit {
            code: Some(1),
            closed: true,
        };

        assert_eq!(final_state(exit, None), SessionState::Ended { code: 1 });
    }
}
