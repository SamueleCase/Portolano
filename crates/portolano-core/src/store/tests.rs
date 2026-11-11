use chrono::{DateTime, Utc};
use pretty_assertions::assert_eq;
use serde_json::json;
use tempfile::TempDir;
use uuid::Uuid;

use super::*;
use crate::model::{Credential, CredentialSpec, CustomProperty, Node, NodeKind, PropertyValue};
use crate::secret::{Secret, SecretRef};

const SENTINEL: &str = "sentinel-not-a-real-password";

fn at(timestamp: &str) -> DateTime<Utc> {
    timestamp.parse().unwrap()
}

fn fixed(mut node: Node, id: u128) -> Node {
    node.id = Uuid::from_u128(id);
    node.created = at("2026-11-10T10:00:00Z");
    node.modified = at("2026-11-10T10:05:00Z");
    node
}

/// A document with a folder, a connection whose credential is referenced by
/// id, and a credential whose password and custom property are in the vault.
fn sample_document(password: &SecretRef, token: &SecretRef) -> Document {
    let mut credential = fixed(Node::credential("Admin ACME"), 3);
    credential.kind = NodeKind::Credential(Credential {
        username: Some("admin".to_owned()),
        domain: Some("ACME".to_owned()),
        password: Some(password.clone()),
        private_key: None,
        passphrase: None,
    });
    credential.custom_properties.push(CustomProperty {
        name: "token".to_owned(),
        value: PropertyValue::Secret {
            secret: token.clone(),
        },
    });

    let mut connection = fixed(Node::connection("web01", "ssh", "web01.example.com"), 2);
    let web01 = connection.as_connection_mut().unwrap();
    web01.port = Some(2222);
    web01.credential = CredentialSpec::ById { id: credential.id };
    web01
        .settings
        .insert("agent_forwarding".to_owned(), json!(true));

    let mut folder = fixed(Node::folder("Clienti"), 1);
    let clients = folder.as_folder_mut().unwrap();
    clients.children = vec![connection, credential];

    let mut doc = Document::new("Portolano");
    doc.id = Uuid::from_u128(0x6f1c);
    doc.created = at("2026-11-10T10:00:00Z");
    doc.modified = at("2026-11-10T10:05:00Z");
    doc.nodes.push(folder);
    doc
}

fn document_path(dir: &TempDir) -> PathBuf {
    dir.path().join("portolano.json")
}

/// A store over a file with the given content, written without the store.
fn store_over(dir: &TempDir, contents: &str) -> Store {
    let path = document_path(dir);
    fs::write(&path, contents).unwrap();
    Store::open(path).unwrap()
}

/// A saved copy of the sample document and its store.
fn saved_sample(dir: &TempDir) -> (Store, Document) {
    let (store, _) = Store::create(document_path(dir), "Portolano").unwrap();
    let mut doc = sample_document(&SecretRef::new(), &SecretRef::new());
    store.save(&mut doc).unwrap();
    (store, doc)
}

fn file_names(dir: &TempDir) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn create_writes_the_returned_document() {
    let dir = TempDir::new().unwrap();

    let (_store, doc) = Store::create(document_path(&dir), "Portolano").unwrap();

    let on_disk: Document =
        serde_json::from_slice(&fs::read(document_path(&dir)).unwrap()).unwrap();
    assert_eq!(on_disk, doc);
    assert_eq!(doc.name, "Portolano");
    assert!(doc.nodes.is_empty());
}

#[test]
fn create_makes_the_missing_directory() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("nested").join("portolano.json");

    Store::create(&path, "Portolano").unwrap();

    assert!(path.is_file());
}

#[test]
fn create_refuses_an_existing_file() {
    let dir = TempDir::new().unwrap();
    drop(Store::create(document_path(&dir), "Portolano").unwrap());

    let error = Store::create(document_path(&dir), "Altro").unwrap_err();

    assert!(matches!(error, Error::AlreadyExists { .. }));
}

#[test]
fn open_of_a_missing_file_fails_without_creating_a_lock() {
    let dir = TempDir::new().unwrap();

    let error = Store::open(document_path(&dir)).unwrap_err();

    assert!(matches!(error, Error::Io { .. }));
    assert!(file_names(&dir).is_empty());
}

#[test]
fn open_fails_while_another_store_holds_the_lock() {
    let dir = TempDir::new().unwrap();
    let (_first, _) = Store::create(document_path(&dir), "Portolano").unwrap();

    let error = Store::open(document_path(&dir)).unwrap_err();

    assert!(matches!(error, Error::Locked { .. }));
}

#[test]
fn open_succeeds_once_the_previous_store_is_dropped() {
    let dir = TempDir::new().unwrap();
    let (first, _) = Store::create(document_path(&dir), "Portolano").unwrap();
    drop(first);

    let reopened = Store::open(document_path(&dir));

    assert!(reopened.is_ok());
}

// Depends on WP1: `load` calls `Document::validate`.
#[test]
fn save_then_load_round_trips_byte_identical() {
    let dir = TempDir::new().unwrap();
    let (store, saved) = saved_sample(&dir);
    let written = fs::read(document_path(&dir)).unwrap();

    let loaded = store.load().unwrap();

    assert_eq!(loaded, saved);
    assert_eq!(to_json(&loaded).unwrap(), written);
}

#[test]
fn saved_json_holds_secret_refs_but_never_secret_values() {
    let dir = TempDir::new().unwrap();
    let (store, _) = Store::create(document_path(&dir), "Portolano").unwrap();
    let (password, token) = (SecretRef::new(), SecretRef::new());
    // The secret value only exists in memory, next to its reference, as the
    // vault would hold it.
    let _vault_entry = (password.clone(), Secret::new(SENTINEL.to_owned()));
    let mut doc = sample_document(&password, &token);

    store.save(&mut doc).unwrap();

    let written = fs::read_to_string(document_path(&dir)).unwrap();
    assert!(!written.contains(SENTINEL));
    assert!(written.contains(password.as_str()));
    assert!(written.contains(token.as_str()));
}

#[test]
fn saved_json_is_two_space_pretty_with_a_final_newline() {
    let dir = TempDir::new().unwrap();
    saved_sample(&dir);

    let written = fs::read_to_string(document_path(&dir)).unwrap();

    assert!(written.starts_with("{\n  \"format\": \"portolano\",\n  \"schema_version\": 1,\n"));
    assert!(written.ends_with("\n}\n"));
}

#[test]
fn serialization_does_not_depend_on_insertion_order() {
    let refs = (SecretRef::new(), SecretRef::new());
    let mut first = sample_document(&refs.0, &refs.1);
    let mut second = first.clone();
    first.extra.insert("a".to_owned(), json!(1));
    first
        .extra
        .insert("b".to_owned(), json!({ "y": 1, "x": 2 }));
    second
        .extra
        .insert("b".to_owned(), json!({ "x": 2, "y": 1 }));
    second.extra.insert("a".to_owned(), json!(1));

    assert_eq!(to_json(&first).unwrap(), to_json(&second).unwrap());
}

#[test]
fn save_updates_modified() {
    let dir = TempDir::new().unwrap();
    let (store, _) = Store::create(document_path(&dir), "Portolano").unwrap();
    let mut doc = sample_document(&SecretRef::new(), &SecretRef::new());
    let before = Utc::now();

    store.save(&mut doc).unwrap();

    assert!(doc.modified >= before);
}

#[test]
fn save_backs_up_the_previous_file() {
    let dir = TempDir::new().unwrap();
    let (store, mut doc) = Store::create(document_path(&dir), "Portolano").unwrap();
    let first_version = fs::read(document_path(&dir)).unwrap();
    doc.name = "Rinominato".to_owned();

    store.save(&mut doc).unwrap();

    let backup = fs::read(dir.path().join("portolano.json.bak")).unwrap();
    assert_eq!(backup, first_version);
}

#[test]
fn save_leaves_no_temporary_file() {
    let dir = TempDir::new().unwrap();

    saved_sample(&dir);

    assert_eq!(
        file_names(&dir),
        [
            "portolano.json",
            "portolano.json.bak",
            "portolano.json.lock"
        ]
    );
}

#[test]
fn first_save_makes_no_backup() {
    let dir = TempDir::new().unwrap();

    Store::create(document_path(&dir), "Portolano").unwrap();

    assert!(!dir.path().join("portolano.json.bak").exists());
}

#[cfg(unix)]
#[test]
fn save_keeps_the_permissions_of_the_existing_file() {
    use std::os::unix::fs::PermissionsExt;

    let dir = TempDir::new().unwrap();
    let (store, mut doc) = Store::create(document_path(&dir), "Portolano").unwrap();
    fs::set_permissions(document_path(&dir), fs::Permissions::from_mode(0o640)).unwrap();

    store.save(&mut doc).unwrap();

    let mode = fs::metadata(document_path(&dir))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o640);
}

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

// Depends on WP1: the duplicate is found by `Document::validate`.
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
