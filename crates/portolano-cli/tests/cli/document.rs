//! `init`, `tree`, `list`, `show`, `config`, global options and exit codes.

use predicates::str::contains;
use pretty_assertions::assert_eq;
use serde_json::Value;

use crate::testutil::*;

#[test]
fn init_creates_the_document_and_refuses_to_overwrite_it() {
    let workspace = Workspace::empty();
    workspace
        .run(&["init", "--name", "Infra"])
        .success()
        .stdout(contains("Infra"));
    workspace
        .run(&["init"])
        .code(1)
        .stderr(contains("esiste già"));
    workspace.run(&["init", "--force"]).success();
    assert!(workspace.path("doc.json.bak").exists());
}

#[test]
fn missing_document_suggests_init() {
    let workspace = Workspace::empty();
    workspace
        .run(&["tree"])
        .code(1)
        .stderr(contains("portolano init"));
}

#[test]
fn ambiguous_id_prefix_lists_the_candidates() {
    let workspace = Workspace::new();
    let doc = workspace.doc_text();
    let mut json: Value = serde_json::from_str(&doc).unwrap();
    let node = |id: &str, name: &str| {
        serde_json::json!({
            "id": id, "name": name, "type": "folder",
            "created": "2026-11-10T10:00:00Z", "modified": "2026-11-10T10:00:00Z"
        })
    };
    json["nodes"] = serde_json::json!([
        node("abcdef00-0000-4000-8000-000000000001", "a"),
        node("abcdef00-0000-4000-8000-000000000002", "b"),
    ]);
    std::fs::write(workspace.doc(), json.to_string()).unwrap();
    workspace
        .run(&["show", "@abcdef00"])
        .code(3)
        .stderr(contains("candidati"));
}

#[test]
fn json_on_a_write_command_is_a_usage_error() {
    Workspace::new()
        .run(&["--json", "add", "folder", "/x"])
        .code(2)
        .stderr(contains("--json"));
}

#[test]
fn bad_arguments_exit_with_2() {
    Workspace::new().run(&["add", "ssh", "/web01"]).code(2);
}

#[test]
fn config_set_show_and_path() {
    let workspace = Workspace::new();
    workspace.ok(&["config", "set", "rdp.client", "freerdp"]);
    workspace.ok(&["config", "set", "terminal.app", "iterm"]);
    let shown = workspace.ok(&["config", "show"]);
    assert!(shown.contains("client = \"freerdp\""));
    assert!(shown.contains("app = \"iterm\""));
    let path = workspace.ok(&["config", "path"]);
    assert_eq!(path.trim(), workspace.path("config.toml").to_string_lossy());
}

#[test]
fn config_set_refuses_invalid_values_and_unknown_keys() {
    let workspace = Workspace::new();
    workspace
        .run(&["config", "set", "rdp.client", "telepatia"])
        .code(4);
    workspace
        .run(&["config", "set", "vnc.program", "x"])
        .code(4);
    assert!(!workspace.path("config.toml").exists());
}

#[test]
fn vault_file_with_the_keychain_backend_is_a_usage_error() {
    let workspace = Workspace::new();
    workspace
        .bare_cmd()
        .args(["tree", "--vault", "keychain", "--vault-file"])
        .arg(workspace.path("vault.json"))
        .assert()
        .code(2)
        .stderr(contains("vale solo con il backend 'file'"));
}
