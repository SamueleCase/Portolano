//! The commands through the real IPC of Tauri, on the mock runtime: the
//! serialization of results and errors, the permissions of the capability,
//! and the sentinel secret that must never reach the frontend.

use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::sync::{Arc, Mutex, Once};

use serde_json::{Value, json};
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder};
use tauri::webview::InvokeRequest;
use tauri::{App, Listener, Manager, WebviewWindow, WebviewWindowBuilder};

use super::*;
use crate::commands::test_support::{
    SENTINEL, TEST_MASTER_PASSWORD, TestApp, admin_with_sentinel, ssh as ssh_node,
};

/// The app with the real configuration of Tauri (capabilities included),
/// over the files of `test`.
fn mock_app(test: &TestApp) -> App<MockRuntime> {
    let builder = commands::builder();
    let invoke_handler = builder.invoke_handler();
    let paths = test.state.paths().clone();
    let app = mock_builder()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(invoke_handler)
        .manage(AppState::new(paths))
        .build(tauri::generate_context!(test = true))
        .unwrap();
    builder.mount_events(&app);
    announce_ended_sessions(app.handle());
    app
}

fn window(app: &App<MockRuntime>, label: &str) -> WebviewWindow<MockRuntime> {
    WebviewWindowBuilder::new(app, label, Default::default())
        .build()
        .unwrap()
}

fn invoke(window: &WebviewWindow<MockRuntime>, command: &str, args: Value) -> Result<Value, Value> {
    let request = InvokeRequest {
        cmd: command.into(),
        callback: CallbackFn(0),
        error: CallbackFn(1),
        url: "tauri://localhost".parse().unwrap(),
        body: InvokeBody::Json(args),
        headers: Default::default(),
        invoke_key: INVOKE_KEY.to_owned(),
    };
    get_ipc_response(window, request).map(|body| body.deserialize::<Value>().unwrap())
}

#[test]
fn app_status_answers_the_main_window_in_camel_case() {
    let test = TestApp::new();
    let app = mock_app(&test);

    let response = invoke(&window(&app, "main"), "app_status", json!({})).unwrap();

    assert_eq!(
        response["defaultDocument"],
        json!({ "path": test.document_path().display().to_string(), "exists": false })
    );
    assert_eq!(response["vault"], "file");
    assert_eq!(response["preferences"]["density"], "normal");
}

#[test]
fn command_error_reaches_the_frontend_as_an_ipc_error() {
    let test = TestApp::new();
    fs::write(&test.state.paths().config, "default_documnt = \"refuso\"\n").unwrap();
    let app = mock_app(&test);

    let error = invoke(&window(&app, "main"), "app_status", json!({})).unwrap_err();

    assert_eq!(error["code"], "invalid");
    assert!(
        error["message"]
            .as_str()
            .unwrap()
            .contains("default_documnt")
    );
}

#[test]
fn command_without_a_document_is_a_no_document_error() {
    let test = TestApp::new();
    let app = mock_app(&test);

    let error = invoke(&window(&app, "main"), "tree", json!({})).unwrap_err();

    assert_eq!(error["code"], "noDocument");
}

#[test]
fn window_without_the_capability_cannot_call_commands() {
    let test = TestApp::new();
    let app = mock_app(&test);

    let error = invoke(&window(&app, "other"), "app_status", json!({})).unwrap_err();

    assert!(is_refused_by_the_acl(&error), "{error}");
}

#[test]
fn main_window_cannot_open_dialogs_of_the_plugin() {
    let test = TestApp::new();
    let app = mock_app(&test);

    let error = invoke(&window(&app, "main"), "plugin:dialog|message", json!({})).unwrap_err();

    assert!(is_refused_by_the_acl(&error), "{error}");
}

#[test]
fn every_command_of_the_bindings_is_allowed_to_the_main_window() {
    let test = TestApp::new();
    let app = mock_app(&test);
    let main = window(&app, "main");

    // The commands that open a native dialog would block: the capability
    // test below covers them.
    for command in build_commands()
        .into_iter()
        .filter(|c| !c.starts_with("choose_"))
    {
        let answer = invoke(&main, &command, json!({}));
        assert!(
            !answer.as_ref().is_err_and(is_refused_by_the_acl),
            "{command} is refused: {answer:?}"
        );
    }
}

/// The commands registered with `collect_commands!`, as the bindings call
/// them.
fn registered_commands() -> BTreeSet<String> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bindings.ts");
    export_bindings(&path).unwrap();
    let bindings = fs::read_to_string(path).unwrap();
    bindings
        .split("__TAURI_INVOKE(\"")
        .skip(1)
        .map(|rest| rest[..rest.find('"').unwrap()].to_owned())
        .collect()
}

/// The commands the capability of the main window allows.
fn allowed_commands() -> BTreeSet<String> {
    let capability: Value =
        serde_json::from_str(include_str!("../capabilities/main.json")).unwrap();
    capability["permissions"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .filter_map(|permission| permission.strip_prefix("allow-"))
        .map(|command| command.replace('-', "_"))
        .collect()
}

#[test]
fn registered_commands_are_exactly_those_of_build_rs_and_of_the_capability() {
    let registered = registered_commands();

    assert_eq!(
        build_commands().into_iter().collect::<BTreeSet<_>>(),
        registered
    );
    assert_eq!(allowed_commands(), registered);
}

/// A Royal document whose import asks for passwords.
const ROYAL_WORKSPACE: &[u8] =
    include_bytes!("../../portolano-import/tests/fixtures/workspace.royal.xml");

/// The commands listed in `build.rs`.
fn build_commands() -> Vec<String> {
    let build = include_str!("../build.rs");
    let list = &build[build.find("&[").unwrap()..build.find("];").unwrap()];
    list.split('"')
        .skip(1)
        .step_by(2)
        .map(str::to_owned)
        .collect()
}

/// What the app logs while the tests of this process run, at every level.
static LOGS: Mutex<Vec<u8>> = Mutex::new(Vec::new());

/// Appends to [`LOGS`].
struct LogBuffer;

impl std::io::Write for LogBuffer {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        LOGS.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Sends every log of the process to [`LOGS`], from any thread.
fn capture_logs() {
    static INSTALLED: Once = Once::new();
    INSTALLED.call_once(|| {
        tracing_subscriber::fmt()
            .with_writer(|| LogBuffer)
            .with_max_level(tracing::Level::TRACE)
            .init();
    });
}

/// SPEC-UI §7: a sentinel password typed in the interface goes into the
/// vault and nowhere else: not in the answers of the commands, not in the
/// events, not in the logs, not in the files of the app outside the
/// encrypted vault.
#[test]
fn sentinel_secret_never_reaches_the_frontend() {
    capture_logs();
    let test = TestApp::new();
    let app = mock_app(&test);
    let main = window(&app, "main");
    let events = Arc::new(Mutex::new(String::new()));
    for name in [
        "document-changed",
        "document-modified-externally",
        "document-missing",
        "vault-locked",
        "session-ended",
    ] {
        let events = Arc::clone(&events);
        app.listen_any(name, move |event| {
            events.lock().unwrap().push_str(event.payload())
        });
    }
    let mut answers = Vec::new();
    let mut call = |command: &str, args: Value| {
        let answer = invoke(&main, command, args);
        answers.push(format!("{command}: {answer:?}"));
        answer
    };

    call(
        "create_document",
        json!({ "source": { "kind": "default" }, "name": "Test" }),
    )
    .unwrap();
    call(
        "unlock_vault",
        json!({ "masterPassword": TEST_MASTER_PASSWORD }),
    )
    .unwrap();
    let admin = call("add_node", json!({ "node": admin_with_sentinel() })).unwrap()["id"].clone();
    let web01 = call("add_node", json!({ "node": ssh_node("web01", None) })).unwrap()["id"].clone();
    call(
        "edit_node",
        json!({ "id": web01, "edit": { "secrets": { "password": { "action": "set", "value": SENTINEL } } } }),
    )
    .unwrap();
    call(
        "set_secret",
        json!({ "id": admin, "slot": { "kind": "password" }, "value": SENTINEL }),
    )
    .unwrap();
    for (command, args) in [
        ("tree", json!({})),
        ("node_details", json!({ "id": admin })),
        ("node_details", json!({ "id": web01 })),
        ("secret_status", json!({ "id": admin })),
        ("search", json!({ "query": "a" })),
        ("validate", json!({})),
        ("connect_preview", json!({ "id": web01 })),
        ("preview_delete", json!({ "id": admin })),
        ("duplicate_node", json!({ "id": admin })),
        ("config_get", json!({})),
        ("vault_status", json!({})),
    ] {
        call(command, args).unwrap();
    }
    call(
        "edit_node",
        json!({ "id": admin, "edit": { "secrets": {
            "passphrase": { "action": "set", "value": SENTINEL },
            "privateKey": { "action": "set", "value": SENTINEL },
        } } }),
    )
    .unwrap();
    // A command that receives a secret and fails: the name is taken.
    let refused = call("add_node", json!({ "node": admin_with_sentinel() }));
    assert!(refused.is_err());
    let import_file = test.dir.path().join("infra.rtsx");
    fs::write(&import_file, ROYAL_WORKSPACE).unwrap();
    app.state::<AppState>()
        .chosen()
        .set(crate::state::FileKind::Import, &import_file);
    let options = json!({ "into": null, "onConflict": "newIds", "includeTrash": false });
    let preview = call("import_preview", json!({ "options": options })).unwrap();
    let answers_to_import: Vec<Value> = preview["requests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|request| {
            json!({ "nodePath": request["nodePath"], "field": request["field"], "value": SENTINEL })
        })
        .collect();
    assert!(!answers_to_import.is_empty());
    call(
        "import_apply",
        json!({ "options": options, "answers": answers_to_import }),
    )
    .unwrap();
    let deleted = call("delete_node", json!({ "id": web01, "options": {} })).unwrap();
    call("undo_delete", json!({ "undoId": deleted["undoId"] })).unwrap();
    // A session whose login is asked at launch, with a probe as `ssh`
    // that gets the typed password from askpass and prints its length.
    let ssh = test.dir.path().join("ssh");
    fs::write(
        &ssh,
        "#!/bin/sh\nanswer=$(\"$SSH_ASKPASS\" \"operatore@web02.example.com's password: \" </dev/null 2>/dev/null)\nprintf 'LUNGHEZZA=%s\\n' \"${#answer}\"\n",
    )
    .unwrap();
    fs::set_permissions(&ssh, fs::Permissions::from_mode(0o755)).unwrap();
    let mut config = fs::OpenOptions::new()
        .append(true)
        .open(&test.state.paths().config)
        .unwrap();
    writeln!(config, "[ssh]\nprogram = {:?}", ssh.display().to_string()).unwrap();
    crate::sessions::tests::cli_next_to_the_tests();
    let mut web02 = ssh_node("web02", None);
    web02["kind"]["credential"] = json!({ "mode": "prompt" });
    let web02 = call("add_node", json!({ "node": web02 })).unwrap()["id"].clone();
    call(
        "open_session",
        json!({
            "node": web02,
            "options": {},
            "login": { "username": "operatore", "domain": null, "password": SENTINEL },
            "size": { "cols": 80, "rows": 24 },
            "output": "__CHANNEL__:1",
            "states": "__CHANNEL__:2",
        }),
    )
    .unwrap();
    // The payload of `session-ended`: the probe exits with 0.
    wait_for(&events, r#""state":"ended""#);
    call("lock_vault", json!({})).unwrap();
    call("tree", json!({})).unwrap();

    let answers = answers.join("\n");
    assert!(answers.contains("web01"), "the commands ran: {answers}");
    assert!(!answers.contains(SENTINEL), "{answers}");
    let events = events.lock().unwrap().clone();
    assert!(
        events.contains("portolano.json"),
        "events were sent: {events}"
    );
    assert!(!events.contains(SENTINEL));
    let logs = String::from_utf8_lossy(&LOGS.lock().unwrap()).into_owned();
    assert!(!logs.contains(SENTINEL), "{logs}");
    assert!(!test.files_text().contains(SENTINEL));
}

/// Waits until the events heard contain `text`.
fn wait_for(events: &Mutex<String>, text: &str) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    while !events.lock().unwrap().contains(text) {
        assert!(
            std::time::Instant::now() < deadline,
            "{text} non ricevuto: {}",
            events.lock().unwrap()
        );
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

fn is_refused_by_the_acl(error: &Value) -> bool {
    error
        .as_str()
        .is_some_and(|message| message.contains("not allowed"))
}
