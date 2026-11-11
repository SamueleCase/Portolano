use super::*;

#[test]
fn load_rejects_a_future_schema_version() {
    let dir = TempDir::new().unwrap();
    let store = store_over(&dir, r#"{ "format": "portolano", "schema_version": 2 }"#);

    let error = store.load().unwrap_err();

    assert!(matches!(
        error,
        Error::UnsupportedSchema {
            found: 2,
            supported: 1
        }
    ));
}

#[test]
fn load_sends_an_older_schema_version_through_migrations() {
    let dir = TempDir::new().unwrap();
    // Version 1 is the first one, so version 0 has no migration: reaching
    // `MissingMigration` proves that `load` went through `migrate`.
    let store = store_over(&dir, r#"{ "format": "portolano", "schema_version": 0 }"#);

    let error = store.load().unwrap_err();

    assert!(matches!(error, Error::MissingMigration { from: 0 }));
}

#[test]
fn load_rejects_a_file_of_another_format() {
    let dir = TempDir::new().unwrap();
    let store = store_over(&dir, r#"{ "format": "royal", "schema_version": 1 }"#);

    let error = store.load().unwrap_err();

    assert!(matches!(error, Error::NotADocument { .. }));
}

#[test]
fn load_rejects_a_document_without_schema_version() {
    let dir = TempDir::new().unwrap();
    let store = store_over(&dir, r#"{ "format": "portolano" }"#);

    let error = store.load().unwrap_err();

    assert!(matches!(error, Error::NotADocument { .. }));
}

#[test]
fn load_rejects_malformed_json() {
    let dir = TempDir::new().unwrap();
    let store = store_over(&dir, "{ \"format\": ");

    let error = store.load().unwrap_err();

    assert!(matches!(error, Error::InvalidDocument { .. }));
}

#[test]
fn load_rejects_a_document_that_does_not_match_the_model() {
    let dir = TempDir::new().unwrap();
    let store = store_over(&dir, r#"{ "format": "portolano", "schema_version": 1 }"#);

    let error = store.load().unwrap_err();

    assert!(matches!(error, Error::InvalidDocument { .. }));
}

#[test]
fn load_rejects_duplicate_ids() {
    let dir = TempDir::new().unwrap();
    let (store, mut doc) = Store::create(document_path(&dir), "Portolano").unwrap();
    let twin = fixed(Node::folder("Gemello"), 7);
    doc.nodes = vec![twin.clone(), twin];
    store.save(&mut doc).unwrap();

    let error = store.load().unwrap_err();

    assert!(matches!(error, Error::DuplicateId { id } if id == Uuid::from_u128(7)));
}
