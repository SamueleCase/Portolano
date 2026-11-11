//! `delete`.

use predicates::str::contains;
use pretty_assertions::assert_eq;

use crate::testutil::*;

/// A workspace with `/Admin` (password in the vault) used by `/web01`.
fn admin_in_use() -> Workspace {
    let workspace = Workspace::new();
    workspace.add_admin();
    workspace.ok(&[
        "add",
        "ssh",
        "/web01",
        "--host",
        "web01.example.com",
        "--cred",
        "/Admin",
    ]);
    workspace
}

/// A workspace with `/bastion` used as jump host by `/web01`.
fn jump_host_in_use() -> Workspace {
    let workspace = Workspace::new();
    workspace.ok(&["add", "ssh", "/bastion", "--host", "bastion.example.com"]);
    workspace.ok(&[
        "add",
        "ssh",
        "/web01",
        "--host",
        "web01.example.com",
        "--jump",
        "/bastion",
    ]);
    workspace
}

#[test]
fn delete_without_a_terminal_needs_yes() {
    let workspace = Workspace::new();
    workspace.ok(&["add", "folder", "/a"]);
    workspace
        .run(&["delete", "/a"])
        .code(1)
        .stderr(contains("-y"));
    workspace.ok(&["show", "/a"]);
}

#[test]
fn delete_with_yes_removes_the_node() {
    let workspace = Workspace::new();
    workspace.ok(&["add", "folder", "/a"]);
    workspace.ok(&["delete", "-y", "/a"]);
    workspace.run(&["show", "/a"]).code(3);
}

#[test]
fn delete_of_a_folder_with_content_needs_recursive() {
    let workspace = Workspace::new();
    workspace.ok(&["add", "folder", "-p", "/a/b"]);
    workspace
        .run(&["delete", "-y", "/a"])
        .code(1)
        .stderr(contains("-r"));
}

#[test]
fn recursive_delete_removes_the_content() {
    let workspace = Workspace::new();
    workspace.ok(&["add", "folder", "-p", "/a/b"]);
    workspace
        .run(&["delete", "-y", "-r", "/a"])
        .success()
        .stdout(contains("2 nodi"));
}

#[test]
fn delete_of_a_referenced_credential_lists_the_references() {
    admin_in_use()
        .run(&["delete", "-y", "/Admin"])
        .code(1)
        .stderr(contains("/web01 -> /Admin"))
        .stderr(contains("--force"));
}

#[test]
fn forced_delete_of_a_credential_makes_its_users_ask_at_launch() {
    let workspace = admin_in_use();
    workspace
        .run(&["delete", "-y", "--force", "/Admin"])
        .success()
        .stdout(contains("1 segreti cancellati"))
        .stdout(contains("chiedono le credenziali"));
    assert_eq!(node(&workspace, "/web01")["credential"]["mode"], "prompt");
}

#[test]
fn delete_of_a_referenced_jump_host_is_refused() {
    jump_host_in_use()
        .run(&["delete", "-y", "/bastion"])
        .code(1)
        .stderr(contains("/web01 -> /bastion"));
}

#[test]
fn forced_delete_of_a_jump_host_reports_the_dangling_reference() {
    jump_host_in_use()
        .run(&["delete", "-y", "--force", "/bastion"])
        .success()
        .stdout(contains("restano riferimenti"))
        .stdout(contains("/web01 -> /bastion"));
}

#[test]
fn dangling_reference_left_by_a_forced_delete_fails_validation() {
    let workspace = jump_host_in_use();
    workspace.ok(&["delete", "-y", "--force", "/bastion"]);
    workspace
        .run(&["validate"])
        .code(4)
        .stdout(contains("dangling_ref"));
}

#[test]
fn forced_deletion_of_two_credentials_used_by_one_node_reports_no_leftover() {
    let workspace = Workspace::new();
    workspace.ok(&["add", "cred", "-p", "/Credenziali/Admin", "--user", "admin"]);
    workspace.ok(&["add", "cred", "/Credenziali/Gateway", "--user", "gw"]);
    workspace.ok(&[
        "add",
        "rdp",
        "/dc01",
        "--host",
        "192.0.2.10",
        "--cred",
        "/Credenziali/Admin",
        "--gateway",
        "gw.example.com",
        "--gateway-cred",
        "/Credenziali/Gateway",
    ]);
    let output = workspace.ok(&["delete", "-y", "-r", "--force", "/Credenziali"]);
    assert!(output.contains("1 nodi ora chiedono le credenziali"));
    assert!(!output.contains("restano riferimenti"));
}
