use chrono::{DateTime, Utc};
use pretty_assertions::assert_eq;
use serde_json::json;
use tempfile::TempDir;
use uuid::Uuid;

use super::*;
use crate::model::{Credential, CredentialSpec, CustomProperty, Node, NodeKind, PropertyValue};
use crate::secret::{Secret, SecretRef};

mod concurrency;
mod load;
mod save;
#[cfg(unix)]
mod symlinks;

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
    Store::create(document_path(&dir), "Portolano").unwrap();

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
fn icons_live_next_to_the_document() {
    let dir = TempDir::new().unwrap();
    let (store, _) = Store::create(document_path(&dir), "Portolano").unwrap();
    let icon = crate::model::Icon::Custom {
        sha256: "ab".repeat(32),
        mime: "image/png".to_owned(),
    };

    let path = store.icons().path_for(&icon);

    // The store resolves symlinks, and temporary directories may be under
    // one (on macOS `/var` points to `/private/var`).
    let expected = fs::canonicalize(dir.path())
        .unwrap()
        .join("portolano.icons")
        .join(format!("{}.png", "ab".repeat(32)));
    assert_eq!(path, Some(expected));
}
