//! The session commands over the state of the app, with probe scripts as
//! `ssh` and as the terminal of new windows: never a real client.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde_json::json;
use tauri::ipc::IpcResponse;

use super::*;
use crate::commands::test_support::{SENTINEL, TestApp};
use crate::error::ErrorCode;
use crate::sessions::tests::{Recorder, SIZE, cli_next_to_the_tests, terminal_text};
use crate::sessions::{SessionEnded, SessionKind};

/// Writes the probe `script` (a `/bin/sh` script) as `name` in the
/// directory of the app.
fn probe(app: &TestApp, name: &str, script: &str) -> PathBuf {
    let path = app.dir.path().join(name);
    fs::write(&path, format!("#!/bin/sh\n{script}")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    path
}

/// Appends `toml` to the configuration of the app.
fn configure(app: &TestApp, toml: &str) {
    let mut config = OpenOptions::new()
        .append(true)
        .open(&app.state.paths().config)
        .unwrap();
    config.write_all(toml.as_bytes()).unwrap();
}

/// An app whose `ssh` is the probe `script`, with an SSH connection `web01`
/// whose credential is asked at launch.
fn app_with_ssh_probe(script: &str) -> (TestApp, Uuid) {
    cli_next_to_the_tests();
    let app = TestApp::with_document();
    let ssh = probe(&app, "ssh", script);
    configure(
        &app,
        &format!("[ssh]\nprogram = {:?}\n", ssh.display().to_string()),
    );
    let web01 = app.add(json!({
        "parent": null,
        "name": "web01",
        "kind": {
            "type": "connection", "protocol": "ssh", "host": "web01.example.com", "port": null,
            "credential": { "mode": "prompt" }, "settings": {},
        },
    }));
    (app, web01)
}

fn login(password: &str) -> LoginInput {
    serde_json::from_value(json!({
        "username": "operatore",
        "domain": null,
        "password": password,
    }))
    .unwrap()
}

fn open_with(
    app: &TestApp,
    node: Uuid,
    login: Option<LoginInput>,
) -> (Result<SessionOpened, IpcError>, Arc<Recorder>) {
    let recorder = Arc::new(Recorder::default());
    let opened = open(
        &app.state,
        node,
        &SessionOptionsInput::default(),
        login,
        SIZE,
        Arc::clone(&recorder) as Arc<dyn SessionSink>,
    );
    (opened, recorder)
}

#[test]
fn a_login_typed_for_the_session_reaches_ssh_through_askpass_only() {
    let (app, web01) = app_with_ssh_probe(
        "answer=$(\"$SSH_ASKPASS\" \"operatore@web01.example.com's password: \" </dev/null 2>/dev/null)\n\
         printf 'LUNGHEZZA=%s\\n' \"${#answer}\"\n",
    );

    let (opened, recorder) = open_with(&app, web01, Some(login(SENTINEL)));

    assert_eq!(opened.unwrap().kind, SessionKind::Terminal);
    recorder.wait_for_output(&format!("LUNGHEZZA={}", SENTINEL.len()));
    recorder.wait_for_end();
    let sent = recorder.sent.lock().unwrap();
    let everything = format!("{}{sent:?}", terminal_text(&sent));
    assert!(!everything.contains(SENTINEL), "{everything}");
    assert!(!app.files_text().contains(SENTINEL));
}

#[test]
fn the_end_of_a_session_is_announced_with_its_id() {
    let (app, web01) = app_with_ssh_probe("exit 255\n");
    let ended = Arc::new(Mutex::new(Vec::<SessionEnded>::new()));
    let heard = Arc::clone(&ended);
    app.state
        .sessions()
        .on_ended(move |session| heard.lock().unwrap().push(session));

    let (opened, recorder) = open_with(&app, web01, None);
    recorder.wait_for_end();

    let ended = ended.lock().unwrap();
    assert_eq!(ended.len(), 1);
    assert_eq!(ended[0].id, opened.unwrap().id);
    assert!(matches!(
        ended[0].state,
        SessionState::Failed {
            code: Some(255),
            ..
        }
    ));
}

#[test]
fn sessions_are_driven_by_their_id() {
    let (app, web01) = app_with_ssh_probe("read line\necho \"letto:$line\"\n");
    let (opened, recorder) = open_with(&app, web01, None);
    let id = opened.unwrap().id;

    app.state.sessions().input(id, b"ciao\r").unwrap();
    recorder.wait_for_output("letto:ciao");
    app.state.sessions().close(id).unwrap();

    let error = IpcError::from(app.state.sessions().close(id).unwrap_err());
    assert_eq!(error.code, ErrorCode::NotFound);
}

#[test]
fn a_session_needs_an_open_document() {
    cli_next_to_the_tests();
    let app = TestApp::new();

    let (opened, _) = open_with(&app, Uuid::new_v4(), None);

    assert_eq!(opened.unwrap_err().code, ErrorCode::NoDocument);
}

#[test]
fn a_folder_cannot_be_opened_as_a_session() {
    let (app, _) = app_with_ssh_probe("exit 0\n");
    let folder = app.add(json!({
        "parent": null,
        "name": "Prod",
        "kind": { "type": "folder", "credential": { "mode": "inherit" } },
    }));

    let (opened, recorder) = open_with(&app, folder, None);

    assert_eq!(opened.unwrap_err().code, ErrorCode::Connect);
    assert!(recorder.sent.lock().unwrap().is_empty());
}

/// The arguments the probe terminal received, one per line.
fn received_arguments(path: &Path) -> Vec<String> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect()
}

#[test]
fn a_new_window_reopens_the_same_document_and_configuration() {
    let (app, web01) = app_with_ssh_probe("exit 0\n");
    let received = app.dir.path().join("argv");
    let terminal = probe(
        &app,
        "terminal",
        &format!("printf '%s\\n' \"$@\" > '{}'\n", received.display()),
    );
    configure(
        &app,
        &format!(
            "[terminal]\napp = \"custom\"\ncommand = {:?}\n",
            format!("{} {{cmd}}", terminal.display())
        ),
    );

    open_new_window(&app.state, web01).unwrap();

    let arguments = received_arguments(&received);
    let config = app.state.paths().config.display().to_string();
    let document = app.document_path().display().to_string();
    let position = |value: &str| arguments.iter().position(|argument| argument == value);
    assert_eq!(
        position("--file").map(|i| &arguments[i + 1]),
        Some(&document)
    );
    assert_eq!(
        position("--config").map(|i| &arguments[i + 1]),
        Some(&config)
    );
    assert_eq!(arguments.last(), Some(&format!("@{web01}")));
    assert!(!arguments.iter().any(|argument| argument.contains(SENTINEL)));
}

#[test]
fn a_frame_travels_as_raw_bytes() {
    let body = OutputFrame(vec![1, 0, b'o', b'k']).body().unwrap();

    assert!(
        matches!(body, tauri::ipc::InvokeResponseBody::Raw(bytes) if bytes == [1, 0, b'o', b'k'])
    );
}
