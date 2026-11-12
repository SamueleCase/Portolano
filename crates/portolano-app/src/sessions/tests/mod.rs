//! Sessions run with harmless probe scripts in place of `ssh`, FreeRDP and
//! the launchers (SPEC-UI §7): never a real client. Hosts and users are
//! fictitious (SPEC §1.3).

mod external;
mod manager;
mod rdp;
mod secrets;
mod terminal;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use portolano_connectors::{LaunchPlan, LaunchStep, PreparedLaunch, prepare};
use portolano_core::vault::MemoryVault;
use portolano_core::{AppConfig, Secret, SecretRef, Vault};
use tempfile::TempDir;

use super::*;

/// How long to wait for something a probe should do.
const TIMEOUT: Duration = Duration::from_secs(20);

/// The size of the terminals of the tests.
pub(crate) const SIZE: TerminalSize = TerminalSize { cols: 80, rows: 24 };

/// What a session sent to its sink.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Sent {
    Output(Vec<u8>),
    State(SessionState),
}

/// A sink that records everything, for the tests to wait on.
#[derive(Default)]
pub(crate) struct Recorder {
    pub(crate) sent: Mutex<Vec<Sent>>,
    changed: Condvar,
}

impl SessionSink for Recorder {
    fn output(&self, frame: Vec<u8>) {
        self.push(Sent::Output(frame));
    }

    fn state(&self, state: &SessionState) {
        self.push(Sent::State(state.clone()));
    }
}

impl Recorder {
    fn push(&self, sent: Sent) {
        self.sent.lock().unwrap().push(sent);
        self.changed.notify_all();
    }

    /// Waits until `done` holds for what was sent; returns it.
    pub(crate) fn wait_until(&self, what: &str, done: impl Fn(&[Sent]) -> bool) -> Vec<Sent> {
        let deadline = Instant::now() + TIMEOUT;
        let mut sent = self.sent.lock().unwrap();
        while !done(&sent) {
            let left = deadline.saturating_duration_since(Instant::now());
            assert!(!left.is_zero(), "{what} non avvenuto; inviato: {sent:?}");
            sent = self.changed.wait_timeout(sent, left).unwrap().0;
        }
        sent.clone()
    }

    /// Waits for the last state of the session.
    pub(crate) fn wait_for_end(&self) -> SessionState {
        let sent = self.wait_until("fine della sessione", |sent| {
            sent.iter()
                .any(|sent| matches!(sent, Sent::State(state) if is_final(state)))
        });
        states(&sent).into_iter().find(is_final).unwrap()
    }

    /// Waits until the terminal output contains `text`.
    pub(crate) fn wait_for_output(&self, text: &str) -> String {
        let sent = self.wait_until(&format!("output {text:?}"), |sent| {
            terminal_text(sent).contains(text)
        });
        terminal_text(&sent)
    }

    fn states(&self) -> Vec<SessionState> {
        states(&self.sent.lock().unwrap())
    }
}

fn is_final(state: &SessionState) -> bool {
    matches!(
        state,
        SessionState::Ended { .. } | SessionState::Failed { .. }
    )
}

fn states(sent: &[Sent]) -> Vec<SessionState> {
    sent.iter()
        .filter_map(|sent| match sent {
            Sent::State(state) => Some(state.clone()),
            Sent::Output(_) => None,
        })
        .collect()
}

/// The payloads of the terminal frames, as text.
pub(crate) fn terminal_text(sent: &[Sent]) -> String {
    sent.iter()
        .filter_map(|sent| match sent {
            Sent::Output(frame) => {
                assert_eq!(frame[..2], [FRAME_VERSION, FrameKind::TerminalOutput as u8]);
                Some(String::from_utf8_lossy(&frame[2..]).into_owned())
            }
            Sent::State(_) => None,
        })
        .collect()
}

/// Writes the probe `script` (a `/bin/sh` script) in `dir`.
fn probe(dir: &TempDir, name: &str, script: &str) -> PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, format!("#!/bin/sh\n{script}")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    path
}

/// A plan that runs `program` and waits for it.
fn run(program: &Path) -> LaunchStep {
    LaunchStep::Run {
        program: program.to_path_buf(),
        args: Vec::new(),
        env: Vec::new(),
        stdin_secret: None,
        askpass: None,
        wait: true,
    }
}

fn plan(steps: Vec<LaunchStep>) -> LaunchPlan {
    LaunchPlan {
        steps,
        warnings: Vec::new(),
    }
}

/// `plan` prepared with `vault`.
fn prepared(plan: &LaunchPlan, vault: &MemoryVault) -> PreparedLaunch {
    prepare(plan, vault, &AppConfig::default()).unwrap()
}

/// A vault holding `value` under a new reference.
fn vault_with(value: &str) -> (MemoryVault, SecretRef) {
    let vault = MemoryVault::default();
    let reference = SecretRef::new();
    vault
        .set(&reference, &Secret::new(value.to_owned()))
        .unwrap();
    (vault, reference)
}

/// A manager whose ended sessions are recorded.
fn manager() -> (SessionManager, Arc<Mutex<Vec<SessionEnded>>>) {
    let ended = Arc::new(Mutex::new(Vec::new()));
    let recorded = Arc::clone(&ended);
    let manager = SessionManager::default();
    manager.on_ended(move |session| recorded.lock().unwrap().push(session));
    (manager, ended)
}

/// Opens `kind` on the probe `script` with a recorder.
fn open(
    manager: &SessionManager,
    kind: SessionKind,
    launch: PreparedLaunch,
) -> (SessionOpened, Arc<Recorder>) {
    let recorder = Arc::new(Recorder::default());
    let opened = manager
        .open(
            kind,
            launch,
            SIZE,
            Arc::clone(&recorder) as Arc<dyn SessionSink>,
        )
        .unwrap();
    (opened, recorder)
}

/// The CLI of the workspace, `target/<profile>/portolano`, linked where the
/// app looks for its sidecar: next to the test binaries, in
/// `target/<profile>/deps/`. The sessions need it for askpass and new
/// windows; `cargo test --workspace` builds it first.
pub(crate) fn cli_next_to_the_tests() -> PathBuf {
    let deps = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned();
    let cli = deps.parent().unwrap().join("portolano");
    assert!(
        cli.is_file(),
        "CLI mancante in {}: `cargo build -p portolano-cli`",
        cli.display()
    );
    match std::os::unix::fs::symlink(&cli, deps.join("portolano")) {
        Err(error) if error.kind() != std::io::ErrorKind::AlreadyExists => panic!("{error}"),
        _ => cli,
    }
}

#[test]
fn askpass_and_new_windows_use_the_cli_next_to_the_app() {
    let cli = cli_next_to_the_tests();

    let environment = environment(&AppConfig::default()).unwrap();

    assert_eq!(
        environment.self_exe.canonicalize().unwrap(),
        cli.canonicalize().unwrap()
    );
}
