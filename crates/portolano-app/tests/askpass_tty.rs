//! The askpass fallback inside a PTY of the app (SPEC-UI §4.5): when the
//! askpass server has no answer (host key confirmation, unknown prompt), the
//! `portolano` client asks on `/dev/tty`, which must be the terminal of the
//! app, so that the question appears in xterm.js and the answer typed there
//! reaches `ssh`.
//!
//! No `ssh` runs: a probe script calls the askpass program as `ssh` does
//! (`$SSH_ASKPASS "<prompt>"`), with stdin and stderr sent to `/dev/null`
//! so that only `/dev/tty` can reach the terminal, and prints what it
//! answered. The client is the `portolano` CLI of the workspace, next to the
//! test binaries.
//!
//! For hidden input the client turns the echo off before it writes the
//! prompt, and back on even when interrupted (DEC-028.2).
#![cfg(unix)]
// The helpers of this test are not `#[test]` functions, which clippy.toml
// already exempts: a failure there is a failed test too (DEC-019.8).
#![allow(clippy::unwrap_used, clippy::panic)]

use std::fs;
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

use nix::sys::termios::LocalFlags;
use portable_pty::{CommandBuilder, MasterPty, PtySize, native_pty_system};
use portolano_connectors::{
    AskpassEntry, AskpassSpec, CHILD_ENV_REMOVE, LaunchPlan, LaunchStep, PreparedLaunch,
    PromptMatcher, prepare,
};
use portolano_core::vault::MemoryVault;
use portolano_core::{AppConfig, Secret, SecretRef, Vault};
use tempfile::TempDir;

/// How long to wait for the probe to print something expected.
const TIMEOUT: Duration = Duration::from_secs(20);

/// The password the askpass server knows: it must never reach the terminal.
const SENTINEL_PASSWORD: &str = "sentinel-pty-password";

const HOST_KEY_PROMPT: &str =
    "Are you sure you want to continue connecting (yes/no/[fingerprint])? ";
const UNKNOWN_PROMPT: &str = "Verification code: ";

/// `target/<profile>/portolano`: integration tests live in
/// `target/<profile>/deps/`. `cargo test --workspace` builds the CLI first.
fn cli() -> PathBuf {
    let deps = std::env::current_exe().unwrap();
    let cli = deps.parent().unwrap().parent().unwrap().join("portolano");
    assert!(
        cli.is_file(),
        "CLI mancante in {}: `cargo build -p portolano-cli`",
        cli.display()
    );
    cli
}

/// A script that runs the askpass program on its first argument, as `ssh`
/// does, and prints the answer between markers. Without stdin and stderr the
/// askpass client can reach the person only through `/dev/tty`.
const ANSWERING_PROBE: &str = "#!/bin/sh\n\
    answer=$(\"$SSH_ASKPASS\" \"$1\" </dev/null 2>/dev/null)\n\
    printf 'RISPOSTA=[%s]\\n' \"$answer\"\n";

/// As [`ANSWERING_PROBE`], but it survives Ctrl-C (its trap, unlike an
/// ignored signal, is not inherited by the askpass client), prints the exit
/// code of the client and then waits for a line, so that the PTY stays open
/// while the test reads its modes.
const INTERRUPTED_PROBE: &str = "#!/bin/sh\n\
    trap 'printf \"INTERROTTO\\n\"' INT\n\
    \"$SSH_ASKPASS\" \"$1\" </dev/null >/dev/null 2>&1\n\
    printf 'USCITA=%s\\n' \"$?\"\n\
    read line\n";

/// Writes `script` as an executable probe in `dir`.
fn probe(dir: &TempDir, script: &str) -> PathBuf {
    let path = dir.path().join("ssh-probe.sh");
    fs::write(&path, script).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    path
}

/// A plan that runs `program` with `prompt`; its askpass server knows only a
/// password for another prompt, so it answers `prompt` with a fallback.
fn plan(program: PathBuf, prompt: &str, password: &SecretRef) -> LaunchPlan {
    LaunchPlan {
        steps: vec![LaunchStep::Run {
            program,
            args: vec![prompt.to_owned()],
            env: vec![],
            stdin_secret: None,
            askpass: Some(AskpassSpec {
                program: cli(),
                entries: vec![AskpassEntry {
                    matcher: PromptMatcher::Password {
                        user: "admin".to_owned(),
                        host: "web01.example.com".to_owned(),
                    },
                    secret: password.clone(),
                }],
            }),
            wait: true,
        }],
        warnings: vec![],
    }
}

/// The terminal of the probe, as the app will drive it.
struct Terminal {
    /// Kept to read the line discipline of the PTY (the echo flag).
    master: Box<dyn MasterPty + Send>,
    output: Receiver<Vec<u8>>,
    seen: String,
    writer: Box<dyn Write + Send>,
}

impl Terminal {
    /// Waits until the terminal has shown `text`; returns everything shown.
    fn wait_for(&mut self, text: &str) -> &str {
        let deadline = Instant::now() + TIMEOUT;
        while !self.seen.contains(text) {
            let left = deadline.saturating_duration_since(Instant::now());
            match self.output.recv_timeout(left) {
                Ok(bytes) => self.seen.push_str(&String::from_utf8_lossy(&bytes)),
                Err(_) => panic!("{text:?} non comparso; terminale: {:?}", self.seen),
            }
        }
        &self.seen
    }

    /// Whether the PTY echoes what is typed, read now.
    fn echoes(&self) -> bool {
        self.master
            .get_termios()
            .unwrap_or_else(|| panic!("termios della PTY non leggibili"))
            .local_flags
            .contains(LocalFlags::ECHO)
    }

    /// Reads the output until the PTY closes; returns everything shown.
    fn read_to_end(&mut self) -> &str {
        let deadline = Instant::now() + TIMEOUT;
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            match self.output.recv_timeout(left) {
                Ok(bytes) => self.seen.push_str(&String::from_utf8_lossy(&bytes)),
                Err(RecvTimeoutError::Disconnected) => return &self.seen,
                Err(RecvTimeoutError::Timeout) => {
                    panic!("la PTY non si è chiusa; terminale: {:?}", self.seen)
                }
            }
        }
    }

    fn type_line(&mut self, line: &str) {
        self.writer.write_all(line.as_bytes()).unwrap();
        self.writer.write_all(b"\r").unwrap();
        self.writer.flush().unwrap();
    }
}

/// Starts `prompt` through the probe `script` in a new PTY, with the
/// environment the terminal of the app gives to `ssh`.
fn run_in_pty(dir: &TempDir, script: &str, prompt: &str) -> Session {
    let password = SecretRef::new();
    let vault = MemoryVault::default();
    vault
        .set(&password, &Secret::new(SENTINEL_PASSWORD.to_owned()))
        .unwrap();
    let prepared = prepare(
        &plan(probe(dir, script), prompt, &password),
        &vault,
        &AppConfig::default(),
    )
    .unwrap();

    let pair = native_pty_system()
        .openpty(PtySize {
            rows: 24,
            cols: 120,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();
    let mut command = CommandBuilder::new(prepared.program());
    command.args(prepared.args());
    for name in CHILD_ENV_REMOVE {
        command.env_remove(name);
    }
    for (key, value) in prepared.env() {
        command.env(key, value);
    }
    command.env("TERM", "xterm-256color");
    let child = pair.slave.spawn_command(command).unwrap();
    // Otherwise the reader never sees the end of the output.
    drop(pair.slave);

    let mut reader = pair.master.try_clone_reader().unwrap();
    let (sender, output) = mpsc::channel();
    std::thread::spawn(move || {
        let mut buffer = [0_u8; 4096];
        while let Ok(read @ 1..) = reader.read(&mut buffer) {
            if sender.send(buffer[..read].to_vec()).is_err() {
                break;
            }
        }
    });
    Session {
        terminal: Terminal {
            writer: pair.master.take_writer().unwrap(),
            master: pair.master,
            output,
            seen: String::new(),
        },
        child,
        _launch: prepared,
    }
}

/// A probe running in a PTY.
struct Session {
    terminal: Terminal,
    child: Box<dyn portable_pty::Child + Send + Sync>,
    /// Keeps the askpass server alive until the session ends, as the
    /// sessions of the app will do.
    _launch: PreparedLaunch,
}

impl Session {
    /// Waits for the probe to succeed; returns everything the terminal showed.
    fn finish(mut self) -> String {
        assert!(self.child.wait().unwrap().success());
        self.terminal.read_to_end().to_owned()
    }
}

#[test]
fn host_key_confirmation_is_asked_in_the_pty_and_answered_from_it() {
    let dir = TempDir::new().unwrap();
    let mut session = run_in_pty(&dir, ANSWERING_PROBE, HOST_KEY_PROMPT);

    session.terminal.wait_for(HOST_KEY_PROMPT);
    session.terminal.type_line("yes");

    // Echoed while typed, then printed by the probe.
    session.terminal.wait_for("RISPOSTA=[yes]");
    let shown = session.finish();

    assert!(!shown.contains(SENTINEL_PASSWORD), "{shown:?}");
}

#[test]
fn unknown_prompt_is_asked_in_the_pty_without_echo() {
    let dir = TempDir::new().unwrap();
    let mut session = run_in_pty(&dir, ANSWERING_PROBE, UNKNOWN_PROMPT);

    session.terminal.wait_for(UNKNOWN_PROMPT);
    session.terminal.type_line("424242");
    session.terminal.wait_for("RISPOSTA=[424242]");
    let shown = session.finish();

    // The code typed in the terminal is not echoed: it appears once, in the
    // line printed by the probe.
    assert_eq!(shown.matches("424242").count(), 1, "{shown:?}");
    assert!(!shown.contains(SENTINEL_PASSWORD), "{shown:?}");
}

#[test]
fn hidden_prompt_shows_only_once_the_echo_is_off() {
    let dir = TempDir::new().unwrap();
    let mut session = run_in_pty(&dir, ANSWERING_PROBE, UNKNOWN_PROMPT);

    session.terminal.wait_for(UNKNOWN_PROMPT);

    // Read as soon as the prompt is visible, without waiting: whatever is
    // typed from now on is not echoed.
    assert!(!session.terminal.echoes(), "{:?}", session.terminal.seen);
    session.terminal.type_line("424242");
    session.finish();
}

#[test]
fn interrupted_hidden_prompt_turns_the_echo_back_on() {
    let dir = TempDir::new().unwrap();
    let mut session = run_in_pty(&dir, INTERRUPTED_PROBE, UNKNOWN_PROMPT);
    session.terminal.wait_for(UNKNOWN_PROMPT);
    assert!(!session.terminal.echoes());

    // Ctrl-C, as typed in the terminal: SIGINT to the foreground group.
    session.terminal.writer.write_all(b"\x03").unwrap();
    session.terminal.writer.flush().unwrap();

    // 130 = 128 + SIGINT: the client still ends because of the signal.
    session.terminal.wait_for("USCITA=130");
    assert!(session.terminal.echoes(), "{:?}", session.terminal.seen);
    session.terminal.type_line("");
    session.finish();
}
