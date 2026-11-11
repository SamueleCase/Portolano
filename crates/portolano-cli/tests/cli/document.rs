//! `init`, `tree`, `list` and `show`.

use std::fs;

use predicates::str::contains;
use pretty_assertions::assert_eq;
use serde_json::Value;

use crate::testutil::*;

#[test]
fn init_creates_the_document_with_the_given_name() {
    let workspace = Workspace::empty();
    workspace
        .run(&["init", "--name", "Infra"])
        .success()
        .stdout(contains("Infra"));
    let json: Value = serde_json::from_str(&workspace.doc_text()).unwrap();
    assert_eq!(json["name"], "Infra");
}

#[test]
fn init_refuses_an_existing_document() {
    Workspace::new()
        .run(&["init"])
        .code(1)
        .stderr(contains("esiste già"));
}

#[test]
fn init_with_force_replaces_the_document_and_keeps_a_backup() {
    let workspace = Workspace::new();
    workspace.ok(&["add", "folder", "/a"]);
    workspace.ok(&["init", "--force"]);
    assert!(workspace.ok(&["list"]).lines().count() == 1);
    assert!(
        fs::read_to_string(workspace.path("doc.json.bak"))
            .unwrap()
            .contains("\"a\"")
    );
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
fn tree_draws_folders_connections_and_credentials() {
    let text = sample().ok(&["tree"]);
    assert_eq!(
        text,
        "Portolano\n\
         ├── 📁 Clienti\n\
         │   └── 📁 ACME\n\
         │       ├── [ssh] web01  admin@web01.example.com:22\n\
         │       └── [rdp] dc01   ACME\\administrator@192.0.2.10\n\
         └── 🔑 Admin             admin\n"
    );
}

#[test]
fn tree_of_a_folder_with_depth_and_ids() {
    let workspace = sample();
    let text = workspace.ok(&["tree", "/Clienti", "--depth", "1", "--ids"]);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with("📁 Clienti  @"));
    assert!(lines[1].starts_with("└── 📁 ACME  @"));
}

#[test]
fn tree_json_is_the_model_without_secrets() {
    let workspace = sample();
    let json: Value = serde_json::from_str(&workspace.ok(&["tree", "--json"])).unwrap();
    assert_eq!(json["format"], "portolano");
    assert!(!json.to_string().contains(SENTINEL));
}

#[test]
fn list_filters_by_type() {
    let workspace = sample();
    let text = workspace.ok(&["list", "--type", "rdp"]);
    assert_eq!(
        text,
        "TIPO  PERCORSO            RIEPILOGO\n\
         rdp   /Clienti/ACME/dc01  ACME\\administrator@192.0.2.10\n"
    );
}

#[test]
fn list_json_has_one_entry_per_node() {
    let workspace = sample();
    let json: Value = serde_json::from_str(&workspace.ok(&["list", "--json"])).unwrap();
    assert_eq!(json.as_array().unwrap().len(), 5);
}

#[test]
fn show_resolved_tells_presence_without_the_secret() {
    let workspace = sample();
    let assert = workspace
        .run(&["show", "/Clienti/ACME/web01", "--resolved"])
        .success();
    let output = all_output(&assert);
    assert!(output.contains("credenziale /Admin"));
    assert!(output.contains("presente nel vault"));
    assert!(!output.contains(SENTINEL));
}

#[test]
fn show_json_includes_the_resolved_credential() {
    let workspace = sample();
    let text = workspace.ok(&["show", "--json", "--resolved", "/Clienti/ACME/web01"]);
    let json: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(json["resolved"]["password"], "present");
    assert_eq!(json["path"], "/Clienti/ACME/web01");
}

#[test]
fn unknown_path_exits_with_3() {
    sample().run(&["show", "/Clienti/nessuno"]).code(3);
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
fn tree_json_of_a_path_is_that_node() {
    let workspace = sample();
    let text = workspace.ok(&["tree", "--json", "/Clienti/ACME"]);
    let json: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(json["name"], "ACME");
    assert_eq!(json["children"].as_array().unwrap().len(), 2);
}
