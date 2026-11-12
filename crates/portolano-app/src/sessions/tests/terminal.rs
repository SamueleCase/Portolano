//! Terminal sessions: a probe in a PTY of the app in place of `ssh`.

use pretty_assertions::assert_eq;

use super::*;

/// Opens a terminal session on the probe `script`.
fn terminal(dir: &TempDir, script: &str) -> (SessionManager, SessionOpened, Arc<Recorder>) {
    let (manager, _) = manager();
    let launch = prepared(
        &plan(vec![run(&probe(dir, "ssh", script))]),
        &MemoryVault::default(),
    );
    let (opened, recorder) = open(&manager, SessionKind::Terminal, launch);
    (manager, opened, recorder)
}

#[test]
fn session_is_connected_at_its_first_output_and_ends_with_its_code() {
    let dir = TempDir::new().unwrap();
    let (_manager, opened, recorder) = terminal(&dir, "echo 'Last login: oggi'\nexit 3\n");

    let last = recorder.wait_for_end();

    assert_eq!(opened.kind, SessionKind::Terminal);
    assert_eq!(
        recorder.states(),
        vec![
            SessionState::Connecting,
            SessionState::Connected,
            last.clone()
        ]
    );
    assert_eq!(last, SessionState::Ended { code: 3 });
    assert!(
        recorder
            .wait_for_output("Last login: oggi")
            .contains("oggi")
    );
}

#[test]
fn exit_255_is_a_failed_connection() {
    let dir = TempDir::new().unwrap();
    let (_manager, _, recorder) = terminal(
        &dir,
        "echo 'ssh: connect to host srv01.example.com port 22: Connection refused'\nexit 255\n",
    );

    let last = recorder.wait_for_end();

    assert!(
        matches!(
            last,
            SessionState::Failed {
                code: Some(255),
                ..
            }
        ),
        "{last:?}"
    );
}

#[test]
fn typed_input_reaches_the_program() {
    let dir = TempDir::new().unwrap();
    let (manager, opened, recorder) = terminal(&dir, "read line\necho \"letto:$line\"\n");

    manager.input(opened.id, b"ciao\r").unwrap();

    recorder.wait_for_output("letto:ciao");
}

#[test]
fn the_terminal_has_the_type_and_size_of_the_tab() {
    let dir = TempDir::new().unwrap();
    let (manager, opened, recorder) = terminal(&dir, "read line\necho \"$TERM $(stty size)\"\n");

    manager
        .resize(
            opened.id,
            TerminalSize {
                cols: 100,
                rows: 30,
            },
        )
        .unwrap();
    manager.input(opened.id, b"\r").unwrap();

    recorder.wait_for_output("xterm-256color 30 100");
}

#[test]
fn closing_a_running_session_ends_its_program() {
    let dir = TempDir::new().unwrap();
    let (manager, opened, recorder) = terminal(&dir, "echo pronto\nsleep 30\n");
    recorder.wait_for_output("pronto");

    manager.close(opened.id).unwrap();

    assert!(matches!(
        recorder.wait_for_end(),
        SessionState::Ended { .. }
    ));
}

#[test]
fn a_secret_for_stdin_is_never_written_to_the_terminal() {
    let dir = TempDir::new().unwrap();
    let (vault, password) = vault_with("sentinel-terminal-stdin");
    let mut step = run(&probe(&dir, "freerdp", "cat\n"));
    if let LaunchStep::Run { stdin_secret, .. } = &mut step {
        *stdin_secret = Some(password);
    }
    let (manager, _) = manager();

    let result = manager.open(
        SessionKind::Terminal,
        prepared(&plan(vec![step]), &vault),
        SIZE,
        Arc::new(Recorder::default()),
    );

    let error = result.unwrap_err();
    assert!(matches!(error, SessionError::StdinSecretInTerminal));
    assert!(error.to_string().contains("Apri in terminale esterno"));
}
