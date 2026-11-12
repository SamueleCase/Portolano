//! The commands through the real IPC of Tauri, on the mock runtime: the
//! serialization of results and errors, the permissions of the capability,
//! and the sentinel secret that must never reach the frontend.

use std::fs;
use std::sync::{Arc, Mutex};

use serde_json::{Value, json};
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder};
use tauri::webview::InvokeRequest;
use tauri::{App, Listener, WebviewWindow, WebviewWindowBuilder};

use super::*;
use crate::commands::test_support::{
    SENTINEL, TEST_MASTER_PASSWORD, TestApp, admin_with_sentinel, ssh,
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

#[test]
fn capability_grants_every_command_of_build_rs() {
    let capability = include_str!("../capabilities/main.json");

    for command in build_commands() {
        let permission = format!("\"allow-{}\"", command.replace('_', "-"));
        assert!(capability.contains(&permission), "{permission} missing");
    }
}

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

/// SPEC-UI §7: a sentinel password typed in the interface goes into the
/// vault and nowhere else: not in the answers of the commands, not in the
/// events, not in the files of the app outside the encrypted vault.
#[test]
fn sentinel_secret_never_reaches_the_frontend() {
    let test = TestApp::new();
    let app = mock_app(&test);
    let main = window(&app, "main");
    let events = Arc::new(Mutex::new(String::new()));
    for name in [
        "document-changed",
        "document-modified-externally",
        "document-missing",
        "vault-locked",
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
    let web01 = call("add_node", json!({ "node": ssh("web01", None) })).unwrap()["id"].clone();
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
    call("delete_node", json!({ "id": web01, "options": {} })).unwrap();
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
    assert!(!test.files_text().contains(SENTINEL));
}

fn is_refused_by_the_acl(error: &Value) -> bool {
    error
        .as_str()
        .is_some_and(|message| message.contains("not allowed"))
}
