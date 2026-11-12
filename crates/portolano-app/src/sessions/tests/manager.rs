//! The bookkeeping of the manager, whatever the kind.

use pretty_assertions::assert_eq;

use super::*;

fn short_session(manager: &SessionManager, dir: &TempDir) -> (SessionOpened, Arc<Recorder>) {
    let launch = prepared(
        &plan(vec![run(&probe(dir, "ssh", "echo fatto\n"))]),
        &MemoryVault::default(),
    );
    open(manager, SessionKind::Terminal, launch)
}

#[test]
fn the_end_of_a_program_is_announced_with_its_session() {
    let dir = TempDir::new().unwrap();
    let (manager, ended) = manager();
    let (opened, recorder) = short_session(&manager, &dir);

    let last = recorder.wait_for_end();

    assert_eq!(
        *ended.lock().unwrap(),
        vec![SessionEnded {
            id: opened.id,
            state: last
        }]
    );
}

#[test]
fn an_ended_session_can_still_be_closed_once() {
    let dir = TempDir::new().unwrap();
    let (manager, _) = manager();
    let (opened, recorder) = short_session(&manager, &dir);
    recorder.wait_for_end();

    manager.close(opened.id).unwrap();

    assert!(matches!(
        manager.close(opened.id),
        Err(SessionError::NotFound { .. })
    ));
}

#[test]
fn an_unknown_session_is_not_found() {
    let (manager, _) = manager();

    let result = manager.resize(SessionId::new(), SIZE);

    assert!(matches!(result, Err(SessionError::NotFound { .. })));
}

#[test]
fn closing_all_ends_every_running_program() {
    let dir = TempDir::new().unwrap();
    let (manager, ended) = manager();
    let launch = || {
        prepared(
            &plan(vec![run(&probe(&dir, "freerdp", "sleep 30\n"))]),
            &MemoryVault::default(),
        )
    };
    open(&manager, SessionKind::RdpExternal, launch());
    open(&manager, SessionKind::RdpExternal, launch());

    let all_ended = manager.close_all();

    // Already over when `close_all` returns, without waiting here.
    assert!(all_ended);
    assert_eq!(ended.lock().unwrap().len(), 2);
}

#[test]
fn closing_all_does_not_wait_for_an_external_application() {
    let dir = TempDir::new().unwrap();
    let (manager, _) = manager();
    let launch = prepared(
        &plan(vec![run(&probe(&dir, "open", "exit 0\n"))]),
        &MemoryVault::default(),
    );
    let (_, recorder) = open(&manager, SessionKind::External, launch);
    recorder.wait_until("apertura", |sent| {
        sent.contains(&Sent::State(SessionState::Connected))
    });

    assert!(manager.close_all());
}

#[test]
fn states_serialize_as_the_frontend_reads_them() {
    let failed = SessionState::Failed {
        code: Some(255),
        reason: "connessione non riuscita".to_owned(),
    };

    assert_eq!(
        serde_json::to_value(&failed).unwrap(),
        serde_json::json!({ "state": "failed", "code": 255, "reason": "connessione non riuscita" })
    );
    assert_eq!(
        serde_json::to_value(SessionState::Ended { code: 0 }).unwrap(),
        serde_json::json!({ "state": "ended", "code": 0 })
    );
    assert_eq!(
        serde_json::to_value(SessionKind::RdpExternal).unwrap(),
        serde_json::json!("rdp-external")
    );
}
