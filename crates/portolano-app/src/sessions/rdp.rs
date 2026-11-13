//! RDP sessions in the window of FreeRDP (SPEC-UI §4.5, "RDP"): the app
//! starts FreeRDP without detaching it and follows its process, so the tab
//! can show that it runs, when it ended and, on failure, its log.

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::mpsc::{self, Sender};

use portolano_connectors::PreparedLaunch;

use super::channel::SessionSink;
use super::error::SessionError;
use super::model::{SessionKind, SessionState};
use super::process::{Exit, Program, WAIT_FAILED, hand_over, start_waiter, stop};
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
        // Opened once and read back through the same descriptor: the
        // connectors remove logs older than 10 minutes by name whenever
        // another connection is prepared, and a long session keeps its own.
        let mut log = log
            .map(|path| OpenOptions::new().read(true).append(true).open(path))
            .transpose()
            .map_err(SessionError::io("apertura del log di FreeRDP"))?;
        let stderr = match &log {
            Some(file) => Stdio::from(
                file.try_clone()
                    .map_err(SessionError::io("apertura del log di FreeRDP"))?,
            ),
            None => Stdio::null(),
        };
        let mut command = launch.command();
        if launch.stdin_secret().is_none() {
            command.stdin(Stdio::null());
        }
        command.stdout(Stdio::null()).stderr(stderr);

        let (close, close_requests) = mpsc::channel();
        let waiter = start_waiter(close_requests, move |exit, launch: PreparedLaunch| {
            drop(launch);
            finish(final_state(exit, log.as_mut()));
        })?;
        let mut child = command.spawn().map_err(|source| SessionError::Spawn {
            program: launch.program().to_path_buf(),
            source,
        })?;
        let written = child
            .stdin
            .take()
            .map_or(Ok(()), |mut stdin| launch.write_stdin_secret(&mut stdin));
        let mut program: Program = child;
        if let Err(source) = written {
            stop(&mut program);
            return Err(SessionError::Io {
                context: "scrittura della password sullo stdin di FreeRDP",
                source,
            });
        }
        // Before the waiting thread can report the end: a FreeRDP that exits
        // at once still reads as started, then failed.
        sink.state(&SessionState::Connected);
        hand_over(&waiter, program, launch)?;
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

/// The last state of FreeRDP: a failure carries the end of its log.
fn final_state(exit: Exit, log: Option<&mut File>) -> SessionState {
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
                .and_then(|file| log_tail(file).ok())
                .filter(|tail| !tail.is_empty())
                .unwrap_or_else(|| format!("FreeRDP è uscito con codice {code}")),
        },
    }
}

/// The last [`LOG_TAIL_BYTES`] of `log`, from the first whole line, as
/// text.
fn log_tail(log: &mut File) -> io::Result<String> {
    let length = log.seek(SeekFrom::End(0))?;
    let start = length.saturating_sub(LOG_TAIL_BYTES);
    log.seek(SeekFrom::Start(start))?;
    let mut bytes = Vec::new();
    log.read_to_end(&mut bytes)?;
    let text = String::from_utf8_lossy(&bytes);
    let text = match text.split_once('\n') {
        Some((_, whole_lines)) if start > 0 => whole_lines,
        _ => &text,
    };
    Ok(text.trim().to_owned())
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use pretty_assertions::assert_eq;

    use super::*;

    /// A log holding `contents`, as FreeRDP leaves it.
    fn log_with(contents: &[u8]) -> File {
        let mut log = tempfile::tempfile().unwrap();
        log.write_all(contents).unwrap();
        log
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
        let mut log = log_with(b"[ERROR] connessione rifiutata da 192.0.2.10\n");

        let state = final_state(failed(131), Some(&mut log));

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
        let mut contents = "riga vecchia\n".repeat(1000).into_bytes();
        contents.extend_from_slice(b"ultima riga\n");
        let mut log = log_with(&contents);

        let tail = log_tail(&mut log).unwrap();

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
