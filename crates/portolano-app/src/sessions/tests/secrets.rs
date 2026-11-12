//! No secret reaches the channels (SPEC-UI §7): the programs receive the
//! sentinel through askpass or stdin, and only its length comes back.

use portolano_connectors::{AskpassEntry, AskpassSpec, PromptMatcher};

use super::*;

const SENTINEL: &str = "sentinel-session-secret-5c1e";

/// Everything sent, events included, as one text to search.
fn everything(recorder: &Recorder, ended: &Mutex<Vec<SessionEnded>>) -> String {
    let sent = recorder.sent.lock().unwrap();
    format!(
        "{}{sent:?}{:?}",
        terminal_text(&sent),
        ended.lock().unwrap()
    )
}

#[test]
fn askpass_answers_ssh_without_the_password_reaching_the_terminal() {
    let dir = TempDir::new().unwrap();
    let (vault, password) = vault_with(SENTINEL);
    let program = probe(
        &dir,
        "ssh",
        "answer=$(\"$SSH_ASKPASS\" \"admin@web01.example.com's password: \" </dev/null 2>/dev/null)\n\
         printf 'LUNGHEZZA=%s\\n' \"${#answer}\"\n",
    );
    let mut step = run(&program);
    if let LaunchStep::Run { askpass, .. } = &mut step {
        *askpass = Some(AskpassSpec {
            program: cli_next_to_the_tests(),
            entries: vec![AskpassEntry {
                matcher: PromptMatcher::Password {
                    user: "admin".to_owned(),
                    host: "web01.example.com".to_owned(),
                },
                secret: password,
            }],
        });
    }
    let (manager, ended) = manager();

    let (_, recorder) = open(
        &manager,
        SessionKind::Terminal,
        prepared(&plan(vec![step]), &vault),
    );
    recorder.wait_for_output(&format!("LUNGHEZZA={}", SENTINEL.len()));
    recorder.wait_for_end();

    let sent = everything(&recorder, &ended);
    assert!(!sent.contains(SENTINEL), "{sent}");
}

#[test]
fn the_rdp_password_goes_to_stdin_and_never_to_the_state() {
    let dir = TempDir::new().unwrap();
    let (vault, password) = vault_with(SENTINEL);
    let log = dir.path().join("freerdp.log");
    fs::write(&log, "").unwrap();
    let program = probe(
        &dir,
        "freerdp",
        "read -r password\nprintf 'letti %s caratteri\\n' \"${#password}\" >&2\nexit 2\n",
    );
    let mut step = run(&program);
    if let LaunchStep::Run { stdin_secret, .. } = &mut step {
        *stdin_secret = Some(password);
    }
    let recorder = Arc::new(Recorder::default());
    let ended = Mutex::new(Vec::new());
    let (finished, last) = std::sync::mpsc::channel();

    let _session = crate::sessions::rdp::RdpSession::start(
        prepared(&plan(vec![step]), &vault),
        Some(log),
        Arc::clone(&recorder) as Arc<dyn SessionSink>,
        Box::new(move |state| finished.send(state).unwrap()),
    )
    .unwrap();
    let state = last.recv_timeout(TIMEOUT).unwrap();

    let expected = format!("letti {} caratteri", SENTINEL.len());
    assert!(
        matches!(&state, SessionState::Failed { reason, .. } if *reason == expected),
        "{state:?}"
    );
    let sent = format!("{}{state:?}", everything(&recorder, &ended));
    assert!(!sent.contains(SENTINEL), "{sent}");
}
