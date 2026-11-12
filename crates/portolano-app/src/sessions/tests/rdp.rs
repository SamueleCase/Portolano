//! RDP sessions: a probe in place of FreeRDP, never FreeRDP itself.

use pretty_assertions::assert_eq;

use super::*;

/// Opens an RDP session on the probe `script`.
fn rdp(dir: &TempDir, script: &str) -> (SessionManager, SessionOpened, Arc<Recorder>) {
    let (manager, _) = manager();
    let launch = prepared(
        &plan(vec![run(&probe(dir, "freerdp", script))]),
        &MemoryVault::default(),
    );
    let (opened, recorder) = open(&manager, SessionKind::RdpExternal, launch);
    (manager, opened, recorder)
}

#[test]
fn running_client_is_connected_and_a_clean_exit_ends_the_session() {
    let dir = TempDir::new().unwrap();
    let (_manager, _, recorder) = rdp(&dir, "exit 0\n");

    let last = recorder.wait_for_end();

    assert_eq!(
        recorder.states(),
        vec![
            SessionState::Connecting,
            SessionState::Connected,
            SessionState::Ended { code: 0 }
        ]
    );
    assert_eq!(last, SessionState::Ended { code: 0 });
}

#[test]
fn a_failure_carries_the_end_of_the_log() {
    let dir = TempDir::new().unwrap();
    let log = dir.path().join("freerdp.log");
    fs::write(&log, "").unwrap();
    let launch = prepared(
        &plan(vec![run(&probe(
            &dir,
            "freerdp",
            "echo '[ERROR] connessione a 192.0.2.10 rifiutata' >&2\nexit 131\n",
        ))]),
        &MemoryVault::default(),
    );
    let recorder = Arc::new(Recorder::default());
    let (finished, last) = std::sync::mpsc::channel();

    let _session = crate::sessions::rdp::RdpSession::start(
        launch,
        Some(log),
        Arc::clone(&recorder) as Arc<dyn SessionSink>,
        Box::new(move |state| finished.send(state).unwrap()),
    )
    .unwrap();

    assert_eq!(
        last.recv_timeout(TIMEOUT).unwrap(),
        SessionState::Failed {
            code: Some(131),
            reason: "[ERROR] connessione a 192.0.2.10 rifiutata".to_owned()
        }
    );
}

#[test]
fn closing_ends_the_client() {
    let dir = TempDir::new().unwrap();
    let (manager, opened, recorder) = rdp(&dir, "sleep 30\n");
    recorder.wait_until("avvio", |sent| sent.len() >= 2);

    manager.close(opened.id).unwrap();

    assert!(matches!(
        recorder.wait_for_end(),
        SessionState::Ended { .. }
    ));
}

#[test]
fn an_rdp_window_takes_no_keyboard_input() {
    let dir = TempDir::new().unwrap();
    let (manager, opened, _recorder) = rdp(&dir, "sleep 30\n");

    let result = manager.input(opened.id, b"x");

    assert!(matches!(result, Err(SessionError::Unsupported { .. })));
    manager.close(opened.id).unwrap();
}
