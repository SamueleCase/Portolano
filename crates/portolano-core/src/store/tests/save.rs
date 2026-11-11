use pretty_assertions::assert_eq;

use super::*;
use crate::model::MAX_DEPTH;
use crate::testutil::folder_chain;

/// An RDP connection with an inline credential and a gateway whose own
/// inline credential is nested in the settings.
fn rdp_with_gateway() -> Node {
    let mut node = fixed(Node::connection("dc01", "rdp", "dc01.example.com"), 0xd1);
    let rdp = node.as_connection_mut().unwrap();
    rdp.credential = CredentialSpec::Inline {
        username: Some("administrator".to_owned()),
        domain: Some("ACME".to_owned()),
        password: Some(SecretRef::new()),
    };
    rdp.settings.insert(
        "gateway".to_owned(),
        json!({
            "host": "gw.example.com",
            "port": 443,
            "credential": {
                "mode": "inline",
                "username": "gateway-user",
                "password": SecretRef::new().as_str(),
            },
        }),
    );
    node
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
fn a_document_at_the_maximum_depth_round_trips() {
    let dir = TempDir::new().unwrap();
    let (store, _) = Store::create(document_path(&dir), "Portolano").unwrap();
    // The connection sits at the deepest allowed level, below MAX_DEPTH - 1
    // folders.
    let (mut doc, ids) = folder_chain(MAX_DEPTH - 1);
    doc.insert(ids.last().copied(), None, rdp_with_gateway())
        .unwrap();
    store.save(&mut doc).unwrap();
    let written = fs::read(document_path(&dir)).unwrap();

    let loaded = store.load().unwrap();

    assert_eq!(loaded, doc);
    assert_eq!(to_json(&loaded).unwrap(), written);
}
