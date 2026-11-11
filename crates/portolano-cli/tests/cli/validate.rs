//! `validate`: exit 4 only for errors.

use predicates::str::contains;
use serde_json::Value;

use crate::testutil::*;

#[test]
fn clean_document_validates() {
    let workspace = Workspace::new();
    workspace.add_admin();
    workspace
        .run(&["validate"])
        .success()
        .stdout(contains("0 errori, 0 avvisi, 0 note"));
}

#[test]
fn missing_secret_is_only_a_warning() {
    let workspace = Workspace::new();
    workspace.add_admin();
    let other = Workspace::empty();
    workspace
        .bare_cmd()
        .arg("--vault-file")
        .arg(other.path("vault.json"))
        .arg("validate")
        .assert()
        .success()
        .stdout(contains("missing_secret"));
}

#[test]
fn unsupported_nodes_are_notes() {
    let workspace = Workspace::new();
    let file = fixture("full-synthetic.royal.xml");
    workspace.ok(&["import", file.to_str().unwrap()]);
    let assert = workspace.run(&["validate", "--json"]);
    let issues: Value = serde_json::from_str(&stdout(assert)).unwrap();
    assert!(
        issues
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue["code"] == "unsupported_node" && issue["severity"] == "info")
    );
}

#[test]
fn errors_exit_with_4() {
    let workspace = Workspace::new();
    workspace.ok(&[
        "add",
        "ssh",
        "/web01",
        "--host",
        "web01.example.com",
        "--cred",
        "name:Nessuna",
    ]);
    workspace
        .run(&["validate"])
        .code(4)
        .stdout(contains("by_name_unresolved"));
}
