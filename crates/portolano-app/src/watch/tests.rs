use std::fs;
use std::sync::mpsc;

use portolano_core::Store;

use super::*;
use crate::commands::test_support::TestApp;

/// Saves the document through another store, as the CLI would.
fn change_elsewhere(path: &Path) {
    let store = Store::open(path).unwrap();
    let mut doc = store.load().unwrap();
    doc.name = "Cambiato dalla CLI".to_owned();
    store.save(&mut doc).unwrap();
}

#[test]
fn unchanged_document_needs_no_event() {
    let app = TestApp::new();
    app.create_default();

    assert_eq!(check_document(&app.state), None);
}

#[test]
fn document_saved_by_another_process_is_modified_externally() {
    let app = TestApp::new();
    app.create_default();

    change_elsewhere(&app.document_path());

    assert_eq!(
        check_document(&app.state),
        Some(FileChange::ModifiedExternally)
    );
}

#[test]
fn deleted_document_is_missing() {
    let app = TestApp::new();
    app.create_default();

    fs::remove_file(app.document_path()).unwrap();

    assert_eq!(check_document(&app.state), Some(FileChange::Missing));
}

#[test]
fn without_a_document_there_is_nothing_to_check() {
    let app = TestApp::new();

    assert_eq!(check_document(&app.state), None);
}

#[test]
fn watcher_reports_a_change_of_the_file() {
    let app = TestApp::new();
    app.create_default();
    let (sender, receiver) = mpsc::channel();
    let _watcher = DocumentWatcher::start(&app.document_path(), move || {
        let _ = sender.send(());
    })
    .unwrap();

    change_elsewhere(&app.document_path());

    // FSEvents may take a few seconds to deliver on a loaded machine.
    assert!(receiver.recv_timeout(Duration::from_secs(15)).is_ok());
}

#[test]
fn watcher_ignores_other_files_of_the_directory() {
    let app = TestApp::new();
    app.create_default();
    let (sender, receiver) = mpsc::channel();
    let _watcher = DocumentWatcher::start(&app.document_path(), move || {
        let _ = sender.send(());
    })
    .unwrap();

    fs::write(app.dir.path().join("note.txt"), "altro").unwrap();

    assert!(receiver.recv_timeout(Duration::from_secs(2)).is_err());
}
