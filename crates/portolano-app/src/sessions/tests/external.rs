//! Windows App and new windows: the launcher of the plan is a probe, never
//! `open` or `osascript`.

use portolano_core::config::{TerminalApp, TerminalConfig};
use pretty_assertions::assert_eq;

use super::*;

fn external(dir: &TempDir, script: &str) -> Arc<Recorder> {
    let (manager, _) = manager();
    // Without the `.rdp` step of the real plan, which would write in the
    // cache directory of the user: the session only runs the launcher.
    let launch = prepared(
        &plan(vec![run(&probe(dir, "open", script))]),
        &MemoryVault::default(),
    );
    open(&manager, SessionKind::External, launch).1
}

#[test]
fn an_application_that_opens_is_connected() {
    let dir = TempDir::new().unwrap();

    let recorder = external(&dir, "exit 0\n");

    recorder.wait_until("apertura", |sent| {
        sent.contains(&Sent::State(SessionState::Connected))
    });
}

#[test]
fn an_application_that_does_not_open_fails() {
    let dir = TempDir::new().unwrap();

    let recorder = external(&dir, "exit 1\n");

    assert!(matches!(
        recorder.wait_for_end(),
        SessionState::Failed { code: Some(1), .. }
    ));
}

/// The plan of a new window, whose launcher is the probe `script` as a
/// custom terminal: the command reaches it as its arguments.
fn new_window(dir: &TempDir, script: &str) -> PreparedLaunch {
    let launcher = probe(dir, "terminal", script);
    let config = AppConfig {
        terminal: TerminalConfig {
            app: TerminalApp::Custom,
            command: Some(format!("{} {{cmd}}", launcher.display())),
        },
        ..AppConfig::default()
    };
    let plan = plan(vec![LaunchStep::OpenInTerminal {
        argv: vec![
            "/Applications/Portolano.app/Contents/MacOS/portolano".to_owned(),
            "connect".to_owned(),
            "@0f4c2a".to_owned(),
        ],
    }]);
    prepare(&plan, &MemoryVault::default(), &config).unwrap()
}

#[test]
fn a_new_window_runs_the_command_of_the_plan() {
    let dir = TempDir::new().unwrap();
    let received = dir.path().join("argv");
    let launch = new_window(&dir, &format!("echo \"$@\" > '{}'\n", received.display()));

    open_in_new_window(launch).unwrap();

    assert_eq!(
        fs::read_to_string(&received).unwrap(),
        "/Applications/Portolano.app/Contents/MacOS/portolano connect @0f4c2a\n"
    );
}

#[test]
fn a_failed_launcher_is_reported() {
    let dir = TempDir::new().unwrap();
    let launch = new_window(&dir, "exit 4\n");

    let result = open_in_new_window(launch);

    assert!(matches!(result, Err(SessionError::NewWindow { code: 4 })));
}
