//! The commands through the real IPC of Tauri, on the mock runtime: the
//! serialization of results and errors, and the permissions of the
//! capability.

use std::fs;

use serde_json::{Value, json};
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder};
use tauri::webview::InvokeRequest;
use tauri::{App, WebviewWindow, WebviewWindowBuilder};
use tempfile::TempDir;

use super::*;

/// The app with the real configuration of Tauri (capabilities included),
/// reading its own configuration from `config` in `dir`.
fn app_with_config(dir: &TempDir, config: &str) -> App<MockRuntime> {
    let config_path = dir.path().join("config.toml");
    fs::write(&config_path, config).unwrap();
    mock_builder()
        .invoke_handler(commands::builder().invoke_handler())
        .manage(AppState::new(config_path))
        .build(tauri::generate_context!(test = true))
        .unwrap()
}

fn window(app: &App<MockRuntime>, label: &str) -> WebviewWindow<MockRuntime> {
    WebviewWindowBuilder::new(app, label, Default::default())
        .build()
        .unwrap()
}

fn invoke(window: &WebviewWindow<MockRuntime>, command: &str) -> Result<Value, Value> {
    let request = InvokeRequest {
        cmd: command.into(),
        callback: CallbackFn(0),
        error: CallbackFn(1),
        url: "tauri://localhost".parse().unwrap(),
        body: InvokeBody::default(),
        headers: Default::default(),
        invoke_key: INVOKE_KEY.to_owned(),
    };
    get_ipc_response(window, request).map(|body| body.deserialize::<Value>().unwrap())
}

#[test]
fn app_status_answers_the_main_window_in_camel_case() {
    let dir = TempDir::new().unwrap();
    let document = dir.path().join("portolano.json").display().to_string();
    let app = app_with_config(
        &dir,
        &format!("default_document = {document:?}\n[vault]\nbackend = \"file\"\n"),
    );

    let response = invoke(&window(&app, "main"), "app_status");

    assert_eq!(
        response,
        Ok(json!({
            "defaultDocument": { "path": document, "exists": false },
            "vault": "file",
        }))
    );
}

#[test]
fn command_error_reaches_the_frontend_as_an_ipc_error() {
    let dir = TempDir::new().unwrap();
    let app = app_with_config(&dir, "default_documnt = \"refuso\"\n");

    let error = invoke(&window(&app, "main"), "app_status").unwrap_err();

    assert_eq!(error["code"], "invalid");
    assert!(
        error["message"]
            .as_str()
            .unwrap()
            .contains("default_documnt")
    );
}

#[test]
fn window_without_the_capability_cannot_call_commands() {
    let dir = TempDir::new().unwrap();
    let app = app_with_config(&dir, "");

    let error = invoke(&window(&app, "other"), "app_status").unwrap_err();

    assert!(
        error
            .as_str()
            .is_some_and(|message| message.contains("not allowed")),
        "{error}"
    );
}
