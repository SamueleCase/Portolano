//! `secret set/clear/status` and the vault.

use std::fs;

use predicates::str::contains;
use pretty_assertions::assert_eq;
use serde_json::Value;

use crate::testutil::*;

#[test]
fn secret_value_is_in_neither_the_document_nor_the_vault_file() {
    let workspace = Workspace::new();
    workspace.add_admin();
    assert!(!workspace.doc_text().contains(SENTINEL));
    let vault = fs::read(workspace.path("vault.json")).unwrap();
    assert!(!String::from_utf8_lossy(&vault).contains(SENTINEL));
    workspace
        .run(&["secret", "status", "/Admin"])
        .success()
        .stdout("password: presente nel vault\n");
}

#[test]
fn secret_set_on_an_existing_reference_keeps_the_document() {
    let workspace = Workspace::new();
    workspace.add_admin();
    let before = workspace.doc_text();
    workspace
        .run_with_stdin(&["secret", "set", "/Admin", "--stdin"], "nuovo-valore\n")
        .success();
    assert_eq!(workspace.doc_text(), before);
}

#[test]
fn secret_set_creates_a_reference_when_missing() {
    let workspace = Workspace::new();
    workspace.ok(&["add", "cred", "/Admin", "--user", "admin"]);
    workspace
        .run(&["secret", "status", "/Admin"])
        .success()
        .stdout(contains("nessun segreto"));
    workspace
        .run_with_stdin(
            &[
                "secret",
                "set",
                "/Admin",
                "--field",
                "prop:Token",
                "--stdin",
            ],
            &format!("{SENTINEL}\n"),
        )
        .success();
    let status: Value =
        serde_json::from_str(&workspace.ok(&["secret", "status", "--json", "/Admin"])).unwrap();
    assert_eq!(status[0]["slot"]["property"], "Token");
    assert_eq!(status[0]["present"], true);
}

#[test]
fn secret_clear_removes_the_secret() {
    let workspace = Workspace::new();
    workspace.add_admin();
    workspace.ok(&["secret", "clear", "/Admin"]);
    workspace
        .run(&["secret", "status", "/Admin"])
        .success()
        .stdout(contains("nessun segreto"));
    workspace.run(&["secret", "clear", "/Admin"]).code(1);
}

#[test]
fn password_on_a_connection_without_inline_credential_is_refused() {
    let workspace = Workspace::new();
    workspace.ok(&[
        "add",
        "ssh",
        "/web01",
        "--host",
        "web01.example.com",
        "--cred",
        "prompt",
    ]);
    workspace
        .run_with_stdin(&["secret", "set", "/web01", "--stdin"], "x\n")
        .code(1)
        .stderr(contains("non è applicabile"));
}

#[test]
fn private_key_is_read_only_from_stdin() {
    let workspace = Workspace::new();
    workspace.ok(&["add", "cred", "/Deploy"]);
    workspace
        .run(&["secret", "set", "/Deploy", "--field", "key"])
        .code(2);
}

#[test]
fn new_file_vault_with_a_short_master_password_exits_with_5() {
    let workspace = Workspace::new();
    let vault = workspace.path("new-vault.json");
    workspace
        .bare_cmd()
        .env("PORTOLANO_VAULT_PASSWORD", &TEST_MASTER_PASSWORD[..11])
        .arg("--vault-file")
        .arg(&vault)
        .args([
            "add",
            "cred",
            "/Admin",
            "--user",
            "admin",
            "--password-stdin",
        ])
        .write_stdin(format!("{SENTINEL}\n"))
        .assert()
        .code(5)
        .stderr(contains("almeno 12 caratteri"));
    assert!(!vault.exists());
}

#[test]
fn password_from_stdin_beyond_64_kib_is_refused() {
    let workspace = Workspace::new();
    let too_long = format!("{}\n", "x".repeat(64 * 1024 + 1));
    workspace
        .run_with_stdin(
            &[
                "add",
                "cred",
                "/Admin",
                "--user",
                "admin",
                "--password-stdin",
            ],
            &too_long,
        )
        .code(1)
        .stderr(contains("supera il limite di 65536 byte"));
    workspace.run(&["show", "/Admin"]).code(3);
}

#[test]
fn private_key_from_stdin_beyond_64_kib_is_refused() {
    let workspace = Workspace::new();
    let too_long = "k".repeat(64 * 1024 + 1);
    workspace
        .run_with_stdin(&["add", "cred", "/Deploy", "--key-stdin"], &too_long)
        .code(1)
        .stderr(contains("supera il limite"));
}

#[test]
fn private_key_from_stdin_keeps_every_line() {
    let workspace = Workspace::new();
    let key = "-----BEGIN TEST KEY-----\nAAAA\n-----END TEST KEY-----\n";
    workspace.ok(&["add", "cred", "/Deploy"]);
    workspace
        .run_with_stdin(
            &["secret", "set", "/Deploy", "--field", "key", "--stdin"],
            key,
        )
        .success();
    let status = workspace.ok(&["secret", "status", "/Deploy"]);
    assert!(
        status.contains("chiave privata: presente nel vault"),
        "{status}"
    );
}

#[test]
fn missing_secret_is_reported_as_absent() {
    let workspace = Workspace::new();
    workspace.add_admin();
    let other = Workspace::empty();
    // The same document with another, empty vault: a second machine.
    let status = workspace
        .bare_cmd()
        .arg("--vault-file")
        .arg(other.path("vault.json"))
        .args(["secret", "status", "/Admin"])
        .assert()
        .success();
    assert_eq!(stdout(status), "password: assente nel vault\n");
}

#[test]
fn wrong_master_password_exits_with_5() {
    let workspace = Workspace::new();
    workspace.add_admin();
    workspace
        .cmd()
        .env("PORTOLANO_VAULT_PASSWORD", "not-the-test-password")
        .args(["secret", "status", "/Admin"])
        .assert()
        .code(5)
        .stderr(contains("master password errata"));
}

#[test]
fn missing_master_password_without_a_terminal_exits_with_5() {
    let workspace = Workspace::new();
    workspace.add_admin();
    workspace
        .cmd()
        .env_remove("PORTOLANO_VAULT_PASSWORD")
        .args(["secret", "status", "/Admin"])
        .assert()
        .code(5);
}

#[test]
fn commands_without_secrets_do_not_open_the_vault() {
    let workspace = Workspace::new();
    workspace.ok(&["add", "folder", "/a"]);
    workspace
        .cmd()
        .env("PORTOLANO_VAULT_PASSWORD", "not-the-test-password")
        .args(["delete", "-y", "/a"])
        .assert()
        .success();
}

#[test]
fn file_vault_is_created_on_first_use() {
    let workspace = Workspace::new();
    let vault = workspace.path("new-vault.json");
    workspace
        .bare_cmd()
        .arg("--vault-file")
        .arg(&vault)
        .args(["add", "cred", "/Admin", "--password-stdin"])
        .write_stdin(format!("{SENTINEL}\n"))
        .assert()
        .success();
    let bytes = fs::read(&vault).unwrap();
    assert!(String::from_utf8_lossy(&bytes).contains("portolano-vault"));
    assert!(!String::from_utf8_lossy(&bytes).contains(SENTINEL));
}

#[test]
fn inapplicable_secret_is_refused_before_asking() {
    let workspace = Workspace::new();
    workspace.ok(&["add", "ssh", "/web01", "--host", "web01.example.com"]);
    let assert = workspace
        .run(&["secret", "set", "/web01", "--field", "passphrase"])
        .code(1);
    let error = stderr(&assert);
    assert!(error.contains("non è applicabile"), "{error}");
    assert!(!error.contains("nessun terminale"), "{error}");
}
