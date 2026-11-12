use portolano_connectors::Registry;
use portolano_core::CredentialSpec;
use portolano_core::vault::MemoryVault;
use pretty_assertions::assert_eq;
use serde_json::json;

use super::*;
use crate::OpsError;
use crate::nodes::{CredentialInput, EditorChanges, NodeEdit, SecretChange, edit_node_staged};
use crate::testutil::*;

fn names(doc: &Document) -> Vec<String> {
    doc.nodes.iter().map(|node| node.name.clone()).collect()
}

fn delete(
    stored: &mut Stored,
    vault: &dyn Vault,
    path: &str,
    options: DeleteOptions,
) -> UndoableDelete {
    let id = id_of(&stored.doc, path);
    delete_undoable(&stored.store, &mut stored.doc, vault, id, options).unwrap()
}

const FORCE: DeleteOptions = DeleteOptions {
    recursive: true,
    force: true,
};

#[test]
fn deletion_is_saved_and_the_secrets_stay_in_the_vault() {
    let (vault, reference) = vault_with(SENTINEL);
    let mut stored = stored(vec![credential("Admin", "admin", Some(reference.clone()))]);

    let undoable = delete(&mut stored, &vault, "/Admin", DeleteOptions::default());

    assert!(stored.reloaded().nodes.is_empty());
    assert!(vault.contains(&reference).unwrap());
    assert_eq!(undoable.deleted.secrets(), [reference]);
    assert_eq!(undoable.outcome.secrets_deleted, 0);
}

#[test]
fn restore_puts_the_node_back_at_its_position() {
    let vault = MemoryVault::default();
    let mut stored = stored(vec![
        connection("a", "ssh"),
        connection("b", "ssh"),
        connection("c", "ssh"),
    ]);
    let undoable = delete(&mut stored, &vault, "/b", DeleteOptions::default());

    let restored =
        restore_deleted(&stored.store, &mut stored.doc, &vault, undoable.deleted).unwrap();

    assert_eq!(restored.path, "/b");
    assert_eq!(names(&stored.reloaded()), vec!["a", "b", "c"]);
}

#[test]
fn restore_gives_the_detached_nodes_their_credential_back() {
    let vault = MemoryVault::default();
    let admin = credential("Admin", "admin", None);
    let spec = CredentialSpec::ById { id: admin.id };
    let mut stored = stored(vec![admin, connection_with("web01", "ssh", spec.clone())]);
    let undoable = delete(&mut stored, &vault, "/Admin", FORCE);
    let web01 = id_of(&stored.doc, "/web01");
    assert_eq!(undoable.outcome.detached, vec![web01]);

    restore_deleted(&stored.store, &mut stored.doc, &vault, undoable.deleted).unwrap();

    let restored = stored.reloaded();
    assert_eq!(
        restored
            .find(web01)
            .unwrap()
            .as_connection()
            .unwrap()
            .credential,
        spec
    );
    assert!(restored.validate().is_empty());
}

#[test]
fn restore_is_refused_when_a_sibling_took_the_name() {
    let vault = MemoryVault::default();
    let mut stored = stored(vec![connection("web01", "ssh")]);
    let undoable = delete(&mut stored, &vault, "/web01", DeleteOptions::default());
    stored.doc.nodes.push(connection("web01", "ssh"));
    stored.store.save(&mut stored.doc).unwrap();

    let result = restore_deleted(&stored.store, &mut stored.doc, &vault, undoable.deleted);

    assert!(matches!(result, Err(OpsError::NameTaken { .. })));
    assert_eq!(stored.doc.nodes.len(), 1);
}

#[test]
fn restore_into_a_folder_that_is_gone_is_refused() {
    let vault = MemoryVault::default();
    let mut stored = stored(vec![folder("Prod", vec![connection("web01", "ssh")])]);
    let undoable = delete(&mut stored, &vault, "/Prod/web01", DeleteOptions::default());
    delete(&mut stored, &vault, "/Prod", DeleteOptions::default());

    let result = restore_deleted(&stored.store, &mut stored.doc, &vault, undoable.deleted);

    assert!(result.is_err());
}

#[test]
fn purge_deletes_the_secrets_of_the_deleted_subtree() {
    let (vault, reference) = vault_with(SENTINEL);
    let mut stored = stored(vec![credential("Admin", "admin", Some(reference.clone()))]);
    let undoable = delete(&mut stored, &vault, "/Admin", DeleteOptions::default());

    let purged = purge_secrets(
        &stored.store,
        &stored.doc,
        &vault,
        undoable.deleted.secrets(),
    );

    assert_eq!(purged.secrets_deleted, 1);
    assert!(!vault.contains(&reference).unwrap());
}

#[test]
fn purge_keeps_a_secret_the_document_references_again() {
    let (vault, reference) = vault_with(SENTINEL);
    let mut stored = stored(vec![credential("Admin", "admin", Some(reference.clone()))]);
    let undoable = delete(&mut stored, &vault, "/Admin", DeleteOptions::default());
    let secrets = undoable.deleted.secrets().to_vec();
    restore_deleted(&stored.store, &mut stored.doc, &vault, undoable.deleted).unwrap();

    let purged = purge_secrets(&stored.store, &stored.doc, &vault, &secrets);

    assert_eq!(purged.secrets_deleted, 0);
    assert!(vault.contains(&reference).unwrap());
}

#[test]
fn locked_vault_stops_the_deletion_before_the_document_changes() {
    let dir = tempfile::tempdir().unwrap();
    let vault = crate::vault::LazyVault::new(
        portolano_core::VaultSelection::File {
            path: fast_file_vault(dir.path()),
        },
        Box::new(NoMasterPassword),
    );
    let mut stored = stored(vec![credential("Admin", "admin", Some(SecretRef::new()))]);
    let id = id_of(&stored.doc, "/Admin");

    let result = delete_undoable(
        &stored.store,
        &mut stored.doc,
        &vault,
        id,
        DeleteOptions::default(),
    );

    assert!(result.is_err());
    assert_eq!(stored.reloaded().nodes.len(), 1);
}

/// A master password source that has none, as the app with a locked vault.
struct NoMasterPassword;

impl portolano_core::MasterPasswordSource for NoMasterPassword {
    fn master_password(
        &self,
        _creating: bool,
    ) -> std::result::Result<portolano_core::Secret, portolano_core::VaultError> {
        Err(portolano_core::VaultError::MasterPasswordUnavailable {
            reason: "vault bloccato".to_owned(),
        })
    }
}

#[test]
fn undo_keeps_a_credential_changed_after_the_deletion() {
    let vault = MemoryVault::default();
    let admin = credential("Admin", "admin", None);
    let spec = CredentialSpec::ById { id: admin.id };
    let mut stored = stored(vec![admin, connection_with("web01", "ssh", spec)]);
    let undoable = delete(&mut stored, &vault, "/Admin", FORCE);
    let web01 = id_of(&stored.doc, "/web01");
    edit_node_staged(
        &stored.store,
        &mut stored.doc,
        &vault,
        &Registry::with_defaults(),
        web01,
        NodeEdit {
            credential: Some(CredentialInput::Inline {
                username: Some("ops".to_owned()),
                domain: None,
            }),
            ..NodeEdit::default()
        },
        EditorChanges {
            password: Some(SecretChange::Set(secret(SENTINEL))),
            ..EditorChanges::default()
        },
    )
    .unwrap();

    let restored =
        restore_deleted(&stored.store, &mut stored.doc, &vault, undoable.deleted).unwrap();

    let CredentialSpec::Inline {
        password: Some(password),
        ..
    } = stored
        .reloaded()
        .find(web01)
        .unwrap()
        .as_connection()
        .unwrap()
        .credential
        .clone()
    else {
        panic!("the inline credential set after the deletion should stay");
    };
    assert_eq!(vault.get(&password).unwrap().unwrap().expose(), SENTINEL);
    assert_eq!(
        restored.kept.iter().map(|node| node.id).collect::<Vec<_>>(),
        vec![web01]
    );
}

#[test]
fn undo_restores_only_the_settings_the_deletion_changed() {
    let vault = MemoryVault::default();
    let admin = credential("Admin", "admin", None);
    let mut dc01 = connection("dc01", "rdp");
    if let Some(connection) = dc01.as_connection_mut() {
        connection.settings.insert(
            "gateway_credential".to_owned(),
            json!({ "mode": "by_id", "$ref": admin.id }),
        );
        connection.settings.insert("scale".to_owned(), json!(100));
    }
    let mut stored = stored(vec![admin, dc01]);
    let undoable = delete(&mut stored, &vault, "/Admin", FORCE);
    let dc01 = id_of(&stored.doc, "/dc01");
    if let Some(connection) = stored
        .doc
        .find_mut(dc01)
        .and_then(|node| node.as_connection_mut())
    {
        connection.settings.insert("scale".to_owned(), json!(140));
    }
    stored.store.save(&mut stored.doc).unwrap();
    let admin = undoable.deleted.id();

    let restored =
        restore_deleted(&stored.store, &mut stored.doc, &vault, undoable.deleted).unwrap();

    let settings = stored
        .reloaded()
        .find(dc01)
        .unwrap()
        .as_connection()
        .unwrap()
        .settings
        .clone();
    assert_eq!(
        settings["gateway_credential"],
        json!({ "mode": "by_id", "$ref": admin })
    );
    assert_eq!(settings["scale"], json!(140));
    assert!(restored.kept.is_empty());
}

#[test]
fn undo_does_not_touch_a_node_it_leaves_as_it_is() {
    let vault = MemoryVault::default();
    let admin = credential("Admin", "admin", None);
    let spec = CredentialSpec::ById { id: admin.id };
    let mut stored = stored(vec![admin, connection_with("web01", "ssh", spec)]);
    let undoable = delete(&mut stored, &vault, "/Admin", FORCE);
    let web01 = id_of(&stored.doc, "/web01");
    if let Some(connection) = stored
        .doc
        .find_mut(web01)
        .and_then(|node| node.as_connection_mut())
    {
        connection.credential = CredentialSpec::None;
    }
    stored.store.save(&mut stored.doc).unwrap();
    let modified = stored.doc.find(web01).unwrap().modified;

    restore_deleted(&stored.store, &mut stored.doc, &vault, undoable.deleted).unwrap();

    assert_eq!(stored.reloaded().find(web01).unwrap().modified, modified);
}
