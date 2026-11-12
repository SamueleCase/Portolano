use pretty_assertions::assert_eq;

use super::*;

#[test]
fn store_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<Store>();
}

#[test]
fn two_stores_can_be_open_on_the_same_file_together() {
    let dir = TempDir::new().unwrap();
    let (_first, _) = Store::create(document_path(&dir), "Portolano").unwrap();

    let second = Store::open(document_path(&dir));

    assert!(second.is_ok());
}

#[test]
fn save_after_a_save_through_another_store_is_refused() {
    let dir = TempDir::new().unwrap();
    let (first, mut doc) = Store::create(document_path(&dir), "Portolano").unwrap();
    let second = Store::open(document_path(&dir)).unwrap();
    second.save(&mut doc.clone()).unwrap();

    let error = first.save(&mut doc).unwrap_err();

    assert!(matches!(error, Error::ModifiedExternally { .. }));
}

#[test]
fn save_after_an_external_edit_is_refused_and_keeps_the_edit() {
    let dir = TempDir::new().unwrap();
    let (store, mut doc) = Store::create(document_path(&dir), "Portolano").unwrap();
    let edited = fs::read_to_string(document_path(&dir))
        .unwrap()
        .replace("Portolano", "Modificato a mano");
    fs::write(document_path(&dir), &edited).unwrap();

    let error = store.save(&mut doc).unwrap_err();

    assert!(matches!(error, Error::ModifiedExternally { .. }));
    assert_eq!(fs::read_to_string(document_path(&dir)).unwrap(), edited);
}

#[test]
fn save_after_the_file_was_deleted_reports_the_missing_document() {
    let dir = TempDir::new().unwrap();
    let (store, mut doc) = Store::create(document_path(&dir), "Portolano").unwrap();
    fs::remove_file(document_path(&dir)).unwrap();

    let error = store.save(&mut doc).unwrap_err();

    assert!(matches!(error, Error::DocumentMissing { .. }));
    assert!(!document_path(&dir).exists());
}

#[test]
fn save_after_the_file_was_moved_reports_the_missing_document() {
    let dir = TempDir::new().unwrap();
    let (store, mut doc) = Store::create(document_path(&dir), "Portolano").unwrap();
    fs::rename(document_path(&dir), dir.path().join("moved.json")).unwrap();

    let error = store.save(&mut doc).unwrap_err();

    assert!(matches!(error, Error::DocumentMissing { .. }));
}

#[test]
fn save_after_reloading_an_external_change_succeeds() {
    let dir = TempDir::new().unwrap();
    let (first, doc) = Store::create(document_path(&dir), "Portolano").unwrap();
    let second = Store::open(document_path(&dir)).unwrap();
    second.save(&mut doc.clone()).unwrap();
    let mut reloaded = first.load().unwrap();

    let saved = first.save(&mut reloaded);

    assert!(saved.is_ok());
}

#[test]
fn consecutive_saves_through_the_same_store_succeed() {
    let dir = TempDir::new().unwrap();
    let (store, mut doc) = Store::create(document_path(&dir), "Portolano").unwrap();
    store.save(&mut doc).unwrap();

    let saved = store.save(&mut doc);

    assert!(saved.is_ok());
}

#[test]
fn save_right_after_open_succeeds_if_the_file_is_unchanged() {
    let dir = TempDir::new().unwrap();
    let (_, mut doc) = Store::create(document_path(&dir), "Portolano").unwrap();
    let store = Store::open(document_path(&dir)).unwrap();

    let saved = store.save(&mut doc);

    assert!(saved.is_ok());
}

#[test]
fn file_is_unchanged_on_disk_right_after_create() {
    let dir = TempDir::new().unwrap();
    let (store, _) = Store::create(document_path(&dir), "Portolano").unwrap();

    assert!(!store.changed_on_disk().unwrap());
}

#[test]
fn own_save_does_not_count_as_a_change_on_disk() {
    let dir = TempDir::new().unwrap();
    let (store, mut doc) = Store::create(document_path(&dir), "Portolano").unwrap();
    doc.name = "Rinominato".to_owned();
    store.save(&mut doc).unwrap();

    assert!(!store.changed_on_disk().unwrap());
}

#[test]
fn save_through_another_store_is_a_change_on_disk() {
    let dir = TempDir::new().unwrap();
    let (first, doc) = Store::create(document_path(&dir), "Portolano").unwrap();
    let second = Store::open(document_path(&dir)).unwrap();
    second.save(&mut doc.clone()).unwrap();

    assert!(first.changed_on_disk().unwrap());
}

#[test]
fn reload_after_an_external_change_clears_the_change_on_disk() {
    let dir = TempDir::new().unwrap();
    let (first, doc) = Store::create(document_path(&dir), "Portolano").unwrap();
    let second = Store::open(document_path(&dir)).unwrap();
    second.save(&mut doc.clone()).unwrap();
    first.load().unwrap();

    assert!(!first.changed_on_disk().unwrap());
}

#[test]
fn deleted_file_is_reported_as_missing_by_the_change_check() {
    let dir = TempDir::new().unwrap();
    let (store, _) = Store::create(document_path(&dir), "Portolano").unwrap();
    fs::remove_file(document_path(&dir)).unwrap();

    let error = store.changed_on_disk().unwrap_err();

    assert!(matches!(error, Error::DocumentMissing { .. }));
}

#[test]
fn deleted_directory_is_reported_as_missing_by_the_change_check() {
    let dir = TempDir::new().unwrap();
    let folder = dir.path().join("documenti");
    let (store, _) = Store::create(folder.join("portolano.json"), "Portolano").unwrap();
    fs::remove_dir_all(&folder).unwrap();

    let error = store.changed_on_disk().unwrap_err();

    assert!(matches!(error, Error::DocumentMissing { .. }), "{error:?}");
}

#[test]
fn save_after_the_directory_was_deleted_reports_the_missing_document() {
    let dir = TempDir::new().unwrap();
    let folder = dir.path().join("documenti");
    let (store, mut doc) = Store::create(folder.join("portolano.json"), "Portolano").unwrap();
    fs::remove_dir_all(&folder).unwrap();

    let error = store.save(&mut doc).unwrap_err();

    assert!(matches!(error, Error::DocumentMissing { .. }), "{error:?}");
}
