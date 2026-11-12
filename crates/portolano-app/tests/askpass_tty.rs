//! The askpass fallback inside a PTY of the app (SPEC-UI §4.5): when the
//! askpass server has no answer (host key confirmation, unknown prompt), the
//! `portolano` client asks on `/dev/tty`, which must be the terminal of the
//! app, so that the question appears in xterm.js and the answer typed there
//! reaches `ssh`.
//!
//! No `ssh` runs: a probe script calls the askpass program exactly as `ssh`
//! does (`$SSH_ASKPASS "<prompt>"`) and prints what it answered. The client
//! is the `portolano` CLI of the workspace, next to the test binaries.
#![cfg(unix)]
// The helpers of this test are not `#[test]` functions, which clippy.toml
// already exempts: a failure there is a failed test too (DEC-019.8).
#![allow(clippy::unwrap_used, clippy::panic)]

use std::fs;
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use portolano_connectors::{
    AskpassEntry, AskpassSpec, CHILD_ENV_REMOVE, LaunchPlan, LaunchStep, PreparedLaunch,
    PromptMatcher, prepare,
};
use portolano_core::vault::MemoryVault;
use portolano_core::{AppConfig, Secret, SecretRef, Vault};
use tempfile::TempDir;

/// How long to wait for the probe to print something expected.
const TIMEOUT: Duration = Duration::from_secs(20);

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
/// does, and prints the answer between markers.
fn probe(dir: &TempDir) -> PathBuf {
    let path = dir.path().join("ssh-probe.sh");
    fs::write(
        &path,
        "#!/bin/sh\nanswer=$(\"$SSH_ASKPASS\" \"$1\")\nprintf 'RISPOSTA=[%s]\\n' \"$answer\"\n",
    )
    .unwrap();
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

    fn type_line(&mut self, line: &str) {
        self.writer.write_all(line.as_bytes()).unwrap();
        self.writer.write_all(b"\r").unwrap();
        self.writer.flush().unwrap();
    }
}

/// Starts `prompt` through the probe in a new PTY, with the environment the
/// terminal of the app gives to `ssh`.
fn run_in_pty(dir: &TempDir, prompt: &str) -> Session {
    let password = SecretRef::new();
    let vault = MemoryVault::default();
    vault
        .set(&password, &Secret::new("sentinel-pty-password".to_owned()))
        .unwrap();
    let prepared = prepare(
        &plan(probe(dir), prompt, &password),
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
            output,
            seen: String::new(),
            writer: pair.master.take_writer().unwrap(),
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

#[test]
fn host_key_confirmation_is_asked_in_the_pty_and_answered_from_it() {
    let dir = TempDir::new().unwrap();
    let mut session = run_in_pty(&dir, HOST_KEY_PROMPT);

    session.terminal.wait_for(HOST_KEY_PROMPT);
    session.terminal.type_line("yes");

    // Echoed while typed, then printed by the probe.
    session.terminal.wait_for("RISPOSTA=[yes]");
    assert!(session.child.wait().unwrap().success());
}

#[test]
fn unknown_prompt_is_asked_in_the_pty_without_echo() {
    let dir = TempDir::new().unwrap();
    let mut session = run_in_pty(&dir, UNKNOWN_PROMPT);

    session.terminal.wait_for(UNKNOWN_PROMPT);
    // The client prints the prompt before turning the echo off: a person
    // reads the prompt first, the test waits for as long.
    std::thread::sleep(Duration::from_millis(500));
    session.terminal.type_line("424242");
    let shown = session.terminal.wait_for("RISPOSTA=[424242]").to_owned();

    // The code typed in the terminal is not echoed: it appears once, in the
    // line printed by the probe.
    assert_eq!(shown.matches("424242").count(), 1, "{shown:?}");
    assert!(session.child.wait().unwrap().success());
}
