use std::fs;

use pretty_assertions::assert_eq;

use super::*;
use crate::commands::test_support::{SENTINEL, TEST_MASTER_PASSWORD, TestApp, admin_with_sentinel};
use crate::commands::tree::tree_of;
use crate::commands::validate::{VAULT_LOCKED, issues};
use crate::error::ErrorCode;

fn secret(text: &str) -> Secret {
    Secret::new(text.to_owned())
}

#[test]
fn file_vault_starts_locked() {
    let app = TestApp::new();

    let status = status(&app.state).unwrap();

    assert_eq!(status.kind, VaultKind::File);
    assert!(status.exists);
    assert!(status.locked);
}

#[test]
fn right_master_password_unlocks_the_vault() {
    let app = TestApp::new();

    let status = unlock(&app.state, secret(TEST_MASTER_PASSWORD)).unwrap();

    assert!(!status.locked);
}

#[test]
fn wrong_master_password_is_a_vault_error_and_keeps_it_locked() {
    let app = TestApp::new();

    let error = unlock(&app.state, secret("password-sbagliata-di-test")).unwrap_err();

    assert_eq!(error.code, ErrorCode::Vault);
    assert!(status(&app.state).unwrap().locked);
}

#[test]
fn locked_vault_needs_the_master_password_again() {
    let app = TestApp::new();
    unlock(&app.state, secret(TEST_MASTER_PASSWORD)).unwrap();

    let status = lock(&app.state).unwrap();

    assert!(status.locked);
}

#[test]
fn missing_vault_cannot_be_unlocked() {
    let app = TestApp::new();
    fs::remove_file(app.dir.path().join("vault.json")).unwrap();

    let error = unlock(&app.state, secret(TEST_MASTER_PASSWORD)).unwrap_err();

    assert_eq!(error.code, ErrorCode::NotFound);
    assert!(!app.dir.path().join("vault.json").exists());
}

#[test]
fn new_vault_needs_a_master_password_of_twelve_characters() {
    let app = TestApp::new();
    fs::remove_file(app.dir.path().join("vault.json")).unwrap();

    let error = create(&app.state, secret("corta")).unwrap_err();

    assert_eq!(error.code, ErrorCode::Vault);
    assert!(!app.dir.path().join("vault.json").exists());
}

#[test]
fn existing_vault_is_not_created_again() {
    let app = TestApp::new();

    let error = create(&app.state, secret(TEST_MASTER_PASSWORD)).unwrap_err();

    assert_eq!(error.code, ErrorCode::Invalid);
}

#[test]
fn locked_vault_shows_the_secrets_as_locked_in_the_tree() {
    let app = TestApp::with_document();
    app.add(admin_with_sentinel());
    lock(&app.state).unwrap();

    let tree = tree_of(&app.state).unwrap();

    assert_eq!(
        serde_json::to_value(tree.nodes[0].secrets).unwrap(),
        "vaultLocked"
    );
}

#[test]
fn locked_vault_gives_one_note_instead_of_missing_secrets() {
    let app = TestApp::with_document();
    app.add(admin_with_sentinel());
    lock(&app.state).unwrap();

    let issues = issues(&app.state).unwrap();

    assert_eq!(
        issues.iter().map(|issue| issue.code).collect::<Vec<_>>(),
        vec![VAULT_LOCKED]
    );
}

#[test]
fn the_vault_file_holds_no_typed_secret_in_clear() {
    let app = TestApp::with_document();

    app.add(admin_with_sentinel());

    assert!(!app.files_text().contains(SENTINEL));
}
