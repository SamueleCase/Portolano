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

/// What the frontend would hear from the app: every answer and error of
/// the commands called through [`Heard::call`] and every event.
struct Heard {
    main: WebviewWindow<MockRuntime>,
    answers: Vec<String>,
    events: Arc<Mutex<String>>,
}

impl Heard {
    fn new(app: &App<MockRuntime>) -> Self {
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
        Self {
            main: window(app, "main"),
            answers: Vec::new(),
            events,
        }
    }

    fn call(&mut self, command: &str, args: Value) -> Result<Value, Value> {
        let answer = invoke(&self.main, command, args);
        self.answers.push(format!("{command}: {answer:?}"));
        answer
    }

    fn answers(&self) -> String {
        self.answers.join("\n")
    }

    fn events(&self) -> String {
        self.events.lock().unwrap().clone()
    }

    /// `secret` is in no answer, event, log, file of the app nor file of
    /// its temporary directory.
    fn assert_never(&self, secret: &str, test: &TestApp) {
        let answers = self.answers();
        assert!(
            !answers.contains(secret),
            "{secret} in una risposta: {answers}"
        );
        let events = self.events();
        assert!(!events.contains(secret), "{secret} in un evento: {events}");
        let logs = String::from_utf8_lossy(&LOGS.lock().unwrap()).into_owned();
        assert!(!logs.contains(secret), "{secret} nei log: {logs}");
        assert!(
            !test.files_text().contains(secret),
            "{secret} in un file dell'app"
        );
        assert!(
            !temporary_files_text().contains(secret),
            "{secret} in un file temporaneo"
        );
    }
}

/// The text of the temporary files of the connectors (the logs of FreeRDP,
/// the `.rdp` of Windows App): their per-user directory in the cache of
/// Portolano, `~/Library/Caches/dev.portolano.Portolano/tmp` on macOS.
fn temporary_files_text() -> String {
    let Some(home) = std::env::home_dir() else {
        return String::new();
    };
    let Ok(entries) = fs::read_dir(home.join("Library/Caches/dev.portolano.Portolano/tmp")) else {
        return String::new();
    };
    entries
        .filter_map(|entry| fs::read(entry.ok()?.path()).ok())
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .collect()
}

/// Writes in `test` an executable probe `name` that runs `script`.
fn probe(test: &TestApp, name: &str, script: &str) -> std::path::PathBuf {
    let path = test.dir.path().join(name);
    fs::write(&path, format!("#!/bin/sh\n{script}")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    path
}

/// Appends `toml` to the configuration of `test`.
fn configure(test: &TestApp, toml: &str) {
    let mut config = fs::OpenOptions::new()
        .append(true)
        .open(&test.state.paths().config)
        .unwrap();
    writeln!(config, "{toml}").unwrap();
}

/// The Royal document of the fixtures as the file of the import dialog;
/// the options of an import of it.
fn choose_royal_document(app: &App<MockRuntime>, test: &TestApp) -> Value {
    let import_file = test.dir.path().join("infra.rtsx");
    fs::write(&import_file, ROYAL_WORKSPACE).unwrap();
    app.state::<AppState>()
        .chosen()
        .set(crate::state::FileKind::Import, &import_file);
    json!({ "into": null, "onConflict": "newIds", "includeTrash": false })
}

/// SPEC-UI §7: a sentinel password typed in the interface goes into the
/// vault and nowhere else: not in the answers of the commands, not in the
/// events, not in the logs, not in the files of the app outside the
/// encrypted vault. Nor does the master password.
#[test]
fn sentinel_secret_never_reaches_the_frontend() {
    capture_logs();
    let test = TestApp::new();
    let app = mock_app(&test);
    let mut heard = Heard::new(&app);

    heard
        .call(
            "create_document",
            json!({ "source": { "kind": "default" }, "name": "Test" }),
        )
        .unwrap();
    heard
        .call(
            "unlock_vault",
            json!({ "masterPassword": TEST_MASTER_PASSWORD }),
        )
        .unwrap();
    let admin = heard
        .call("add_node", json!({ "node": admin_with_sentinel() }))
        .unwrap()["id"]
        .clone();
    let web01 = heard
        .call("add_node", json!({ "node": ssh_node("web01", None) }))
        .unwrap()["id"]
        .clone();
    heard
        .call(
            "edit_node",
            json!({ "id": web01, "edit": { "secrets": { "password": { "action": "set", "value": SENTINEL } } } }),
        )
        .unwrap();
    heard
        .call(
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
        heard.call(command, args).unwrap();
    }
    heard
        .call(
            "edit_node",
            json!({ "id": admin, "edit": { "secrets": {
                "passphrase": { "action": "set", "value": SENTINEL },
                "privateKey": { "action": "set", "value": SENTINEL },
            } } }),
        )
        .unwrap();
    // A command that receives a secret and fails: the name is taken.
    let refused = heard.call("add_node", json!({ "node": admin_with_sentinel() }));
    assert!(refused.is_err());
    let options = choose_royal_document(&app, &test);
    let preview = heard
        .call("import_preview", json!({ "options": options }))
        .unwrap();
    let answers_to_import: Vec<Value> = preview["requests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|request| {
            json!({ "nodePath": request["nodePath"], "field": request["field"], "value": SENTINEL })
        })
        .collect();
    assert!(!answers_to_import.is_empty());
    heard
        .call(
            "import_apply",
            json!({ "options": options, "answers": answers_to_import }),
        )
        .unwrap();
    let deleted = heard
        .call("delete_node", json!({ "id": web01, "options": {} }))
        .unwrap();
    heard
        .call("undo_delete", json!({ "undoId": deleted["undoId"] }))
        .unwrap();
    // A session whose login is asked at launch, with a probe as `ssh` that
    // gets the typed password from askpass: it exits with 0 only if the
    // answer has the length of the sentinel, so the password did reach it.
    // The output and state channels cannot be read on the mock runtime:
    // their content is checked with a recording sink by the tests of
    // `sessions/ipc/tests.rs` and `sessions/tests/secrets.rs`.
    let ssh = probe(
        &test,
        "ssh",
        &format!(
            "answer=$(\"$SSH_ASKPASS\" \"operatore@web02.example.com's password: \" </dev/null 2>/dev/null)\n[ \"${{#answer}}\" -eq {} ] || exit 3\n",
            SENTINEL.len()
        ),
    );
    configure(
        &test,
        &format!("[ssh]\nprogram = {:?}", ssh.display().to_string()),
    );
    crate::sessions::tests::cli_next_to_the_tests();
    let mut web02 = ssh_node("web02", None);
    web02["kind"]["credential"] = json!({ "mode": "prompt" });
    let web02 = heard.call("add_node", json!({ "node": web02 })).unwrap()["id"].clone();
    heard
        .call(
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
    // The payload of `session-ended`.
    wait_for(&heard.events, r#""state":"ended""#);
    assert!(
        heard.events().contains(r#""state":"ended","code":0"#),
        "the probe got the password: {}",
        heard.events()
    );
    heard.call("lock_vault", json!({})).unwrap();
    heard.call("tree", json!({})).unwrap();

    assert!(
        heard.answers().contains("web01"),
        "the commands ran: {}",
        heard.answers()
    );
    assert!(
        heard.events().contains("portolano.json"),
        "events were sent: {}",
        heard.events()
    );
    heard.assert_never(SENTINEL, &test);
    heard.assert_never(TEST_MASTER_PASSWORD, &test);
}

/// A wrong master password, refused by the vault.
const WRONG_MASTER_PASSWORD: &str = "sentinel-wrong-master-6d2a";

/// The master password of a vault created in the app.
const NEW_MASTER_PASSWORD: &str = "sentinel-new-master-8e71";

/// SPEC-UI §7, the paths left out above: the master passwords of a refused
/// unlock and of a new vault, the commands that fail while they receive a
/// secret, and the password of FreeRDP on its stdin, whose log the tab and
/// `session-ended` show on a failure.
#[test]
fn sentinel_secret_of_refusals_of_the_vault_and_of_freerdp_never_reaches_the_frontend() {
    capture_logs();
    let test = TestApp::new();
    let app = mock_app(&test);
    let mut heard = Heard::new(&app);
    heard
        .call(
            "create_document",
            json!({ "source": { "kind": "default" }, "name": "Test" }),
        )
        .unwrap();

    let wrong = heard.call(
        "unlock_vault",
        json!({ "masterPassword": WRONG_MASTER_PASSWORD }),
    );
    assert!(wrong.is_err(), "{wrong:?}");
    heard
        .call(
            "unlock_vault",
            json!({ "masterPassword": TEST_MASTER_PASSWORD }),
        )
        .unwrap();
    let admin = heard
        .call("add_node", json!({ "node": admin_with_sentinel() }))
        .unwrap()["id"]
        .clone();
    let folder = json!({ "parent": null, "name": "Rete",
        "kind": { "type": "folder", "credential": { "mode": "inherit" } } });
    let folder = heard.call("add_node", json!({ "node": folder })).unwrap()["id"].clone();
    // A folder that inherits its credential holds no password nor key.
    let set = heard.call(
        "set_secret",
        json!({ "id": folder, "slot": { "kind": "password" }, "value": SENTINEL }),
    );
    assert!(set.is_err(), "{set:?}");
    let edit = heard.call(
        "edit_node",
        json!({ "id": folder, "edit": { "secrets": { "privateKey": { "action": "set", "value": SENTINEL } } } }),
    );
    assert!(edit.is_err(), "{edit:?}");
    // Answers that are not those of the requests of the import.
    let options = choose_royal_document(&app, &test);
    heard
        .call("import_preview", json!({ "options": options }))
        .unwrap();
    let import = heard.call(
        "import_apply",
        json!({ "options": options, "answers": [
            { "nodePath": "Nessun/nodo", "field": { "kind": "password" }, "value": SENTINEL },
        ] }),
    );
    assert!(import.is_err(), "{import:?}");
    // FreeRDP reads the password of `admin` on its stdin, logs only its
    // length and fails: the end of its log reaches `session-ended`.
    let freerdp = probe(
        &test,
        "freerdp",
        "read -r password\nprintf 'letti %s caratteri\\n' \"${#password}\" >&2\nexit 2\n",
    );
    configure(
        &test,
        &format!(
            "[rdp]\nclient = \"freerdp\"\nfreerdp_program = {:?}",
            freerdp.display().to_string()
        ),
    );
    let dc01 = json!({ "parent": null, "name": "dc01", "kind": {
        "type": "connection", "protocol": "rdp", "host": "192.0.2.10", "port": null,
        "credential": { "mode": "byId", "id": admin }, "settings": {},
    } });
    let dc01 = heard.call("add_node", json!({ "node": dc01 })).unwrap()["id"].clone();
    heard
        .call(
            "open_session",
            json!({
                "node": dc01,
                "options": {},
                "login": null,
                "size": { "cols": 80, "rows": 24 },
                "output": "__CHANNEL__:1",
                "states": "__CHANNEL__:2",
            }),
        )
        .unwrap();
    wait_for(&heard.events, r#""state":"failed""#);
    assert!(
        heard
            .events()
            .contains(&format!("letti {} caratteri", SENTINEL.len())),
        "FreeRDP got the password and its log reached the event: {}",
        heard.events()
    );
    heard.call("lock_vault", json!({})).unwrap();
    // A new file vault, created with its own master password. The file of
    // the vault is chosen with a native dialog: here it changes in the
    // configuration, as another program would change it.
    let new_vault = test.dir.path().join("nuovo-vault.json");
    let config = &test.state.paths().config;
    let old_vault = test.dir.path().join("vault.json").display().to_string();
    let toml = fs::read_to_string(config).unwrap();
    fs::write(
        config,
        toml.replace(&old_vault, &new_vault.display().to_string()),
    )
    .unwrap();
    heard
        .call(
            "create_file_vault",
            json!({ "masterPassword": NEW_MASTER_PASSWORD }),
        )
        .unwrap();
    assert!(new_vault.is_file());

    assert!(
        heard.answers().contains("master password errata"),
        "the errors were heard: {}",
        heard.answers()
    );
    assert!(
        temporary_files_text().contains(&format!("letti {} caratteri", SENTINEL.len())),
        "the log of FreeRDP is among the temporary files read"
    );
    for secret in [
        SENTINEL,
        TEST_MASTER_PASSWORD,
        WRONG_MASTER_PASSWORD,
        NEW_MASTER_PASSWORD,
    ] {
        heard.assert_never(secret, &test);
    }
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
