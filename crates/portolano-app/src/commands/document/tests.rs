use std::fs;

use pretty_assertions::assert_eq;

use super::*;
use crate::commands::test_support::TestApp;
use crate::error::ErrorCode;

#[test]
fn status_reports_the_configured_document_and_vault() {
    let app = TestApp::new();

    let status = status(&app.state).unwrap();

    assert_eq!(
        status.default_document,
        DocumentLocation {
            path: app.document_path().display().to_string(),
            exists: false,
        }
    );
    assert_eq!(status.vault, VaultKind::File);
    assert_eq!(status.document, None);
}

#[test]
fn invalid_configuration_is_an_invalid_error() {
    let app = TestApp::new();
    fs::write(&app.state.paths().config, "default_documnt = \"refuso\"\n").unwrap();

    let error = status(&app.state).unwrap_err();

    assert_eq!(error.code, ErrorCode::Invalid);
    assert!(
        error.message.contains("default_documnt"),
        "{}",
        error.message
    );
}

#[test]
fn created_document_is_open_and_remembered() {
    let app = TestApp::new();

    let info = app.create_default();

    assert_eq!(info.name, "Test");
    assert_eq!(info.file_name, "portolano.json");
    let status = status(&app.state).unwrap();
    assert_eq!(status.document, Some(info));
    assert_eq!(
        status.last_document.map(|last| last.path),
        Some(app.document_path().display().to_string())
    );
}

#[test]
fn creating_over_an_existing_file_is_refused() {
    let app = TestApp::new();
    app.create_default();

    let error = create(&app.state, &app.document_path(), "Altro").unwrap_err();

    assert_eq!(error.code, ErrorCode::Invalid);
}

#[test]
fn missing_document_cannot_be_opened() {
    let app = TestApp::new();

    let error = open(&app.state, &app.document_path()).unwrap_err();

    assert_eq!(error.code, ErrorCode::DocumentMissing);
}

#[test]
fn recent_document_opens_only_from_the_list() {
    let app = TestApp::new();
    app.create_default();
    close(&app.state);

    let listed = source_path(
        &app.state,
        &DocumentSource::Recent {
            path: app.document_path().display().to_string(),
        },
        FileKind::Document,
    );
    let other = source_path(
        &app.state,
        &DocumentSource::Recent {
            path: "/etc/passwd".to_owned(),
        },
        FileKind::Document,
    );

    assert_eq!(listed.unwrap(), app.document_path());
    assert_eq!(other.unwrap_err().code, ErrorCode::NotFound);
}

#[test]
fn chosen_source_without_a_chosen_file_is_refused() {
    let app = TestApp::new();

    let error = source_path(&app.state, &DocumentSource::Chosen, FileKind::Document).unwrap_err();

    assert_eq!(error.code, ErrorCode::NotFound);
}

#[test]
fn commands_without_a_document_say_so() {
    let app = TestApp::new();

    let error = reload(&app.state).unwrap_err();

    assert_eq!(error.code, ErrorCode::NoDocument);
}

#[test]
fn closed_document_is_no_longer_open() {
    let app = TestApp::new();
    app.create_default();

    close(&app.state);

    assert_eq!(app.state.document_path(), None);
}

#[test]
fn reload_reads_the_changes_made_by_others() {
    let app = TestApp::new();
    app.create_default();
    let store = portolano_core::Store::open(app.document_path()).unwrap();
    let mut doc = store.load().unwrap();
    doc.name = "Cambiato dalla CLI".to_owned();
    store.save(&mut doc).unwrap();

    let info = reload(&app.state).unwrap();

    assert_eq!(info.name, "Cambiato dalla CLI");
}

#[test]
fn recent_documents_say_whether_they_still_exist() {
    let app = TestApp::new();
    app.create_default();
    fs::remove_file(app.document_path()).unwrap();

    let recents = recents(&app.state);

    assert_eq!(recents.len(), 1);
    assert!(!recents[0].exists);
}
