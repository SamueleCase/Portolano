use portolano_core::vault::MemoryVault;
use portolano_core::{CustomProperty, PrivateKey};
use pretty_assertions::assert_eq;

use super::*;
use crate::testutil::*;

fn set(stored: &mut Stored, vault: &dyn Vault, path: &str, slot: SecretSlot) -> Result<SecretRef> {
    let id = id_of(&stored.doc, path);
    set_secret(
        &stored.store,
        &mut stored.doc,
        vault,
        id,
        slot,
        secret(SENTINEL),
    )
}

fn clear(
    stored: &mut Stored,
    vault: &dyn Vault,
    path: &str,
    slot: SecretSlot,
) -> Result<EditOutcome> {
    let id = id_of(&stored.doc, path);
    clear_secret(&stored.store, &mut stored.doc, vault, id, slot)
}

fn inline(password: Option<SecretRef>) -> CredentialSpec {
    CredentialSpec::Inline {
        username: Some("admin".to_owned()),
        domain: None,
        password,
    }
}

#[test]
fn existing_reference_is_overwritten_in_place_without_saving() {
    let (vault, reference) = vault_with("old-value");
    let mut stored = stored(vec![credential("Admin", "admin", Some(reference.clone()))]);
    let before = stored.file_text();
    let returned = set(&mut stored, &vault, "/Admin", SecretSlot::Password).unwrap();
    assert_eq!(returned, reference);
    assert_eq!(vault.get(&reference).unwrap().unwrap().expose(), SENTINEL);
    assert_eq!(stored.file_text(), before);
}

#[test]
fn missing_reference_is_created_and_saved() {
    let vault = MemoryVault::default();
    let mut stored = stored(vec![credential("Admin", "admin", None)]);
    let reference = set(&mut stored, &vault, "/Admin", SecretSlot::Password).unwrap();
    let saved = stored.reloaded();
    let admin = saved.find(id_of(&saved, "/Admin")).unwrap();
    assert_eq!(
        admin.as_credential().unwrap().password,
        Some(reference.clone())
    );
    assert!(vault.contains(&reference).unwrap());
    assert!(!stored.file_text().contains(SENTINEL));
}

#[test]
fn new_reference_is_rolled_back_when_the_save_fails() {
    let vault = RecordingVault::default();
    let mut stored = stored(vec![credential("Admin", "admin", None)]);
    stored.modify_externally();
    let result = set(&mut stored, &vault, "/Admin", SecretSlot::Password);
    assert!(result.is_err());
    assert!(crate::tree_util::document_secret_refs(&stored.doc).is_empty());
    let (written, deleted) = vault.history();
    assert_eq!(written.len(), 1);
    assert_eq!(written, deleted);
}

#[test]
fn password_of_an_inline_connection_is_settable() {
    let vault = MemoryVault::default();
    let mut stored = stored(vec![connection_with("web01", "ssh", inline(None))]);
    let reference = set(&mut stored, &vault, "/web01", SecretSlot::Password).unwrap();
    let node = stored.doc.find(id_of(&stored.doc, "/web01")).unwrap();
    assert_eq!(
        node.as_connection().unwrap().credential,
        inline(Some(reference))
    );
}

#[test]
fn password_of_a_connection_without_inline_credential_is_refused() {
    let mut stored = stored(vec![connection("web01", "ssh")]);
    let result = set(
        &mut stored,
        &MemoryVault::default(),
        "/web01",
        SecretSlot::Password,
    );
    assert!(matches!(result, Err(OpsError::SecretNotApplicable { .. })));
}

#[test]
fn passphrase_of_a_connection_is_refused() {
    let mut stored = stored(vec![connection_with("web01", "ssh", inline(None))]);
    let result = set(
        &mut stored,
        &MemoryVault::default(),
        "/web01",
        SecretSlot::Passphrase,
    );
    assert!(matches!(result, Err(OpsError::SecretNotApplicable { .. })));
}

#[test]
fn key_in_the_vault_replaces_a_key_file() {
    let mut admin = credential("Admin", "admin", None);
    if let Some(credential) = admin.as_credential_mut() {
        credential.private_key = Some(PrivateKey::File {
            path: "~/.ssh/id_ed25519".to_owned(),
        });
    }
    let mut stored = stored(vec![admin]);
    let reference = set(
        &mut stored,
        &MemoryVault::default(),
        "/Admin",
        SecretSlot::PrivateKey,
    )
    .unwrap();
    let node = stored.doc.find(id_of(&stored.doc, "/Admin")).unwrap();
    assert_eq!(
        node.as_credential().unwrap().private_key,
        Some(PrivateKey::Vault { secret: reference })
    );
}

#[test]
fn secret_property_is_created_on_any_node() {
    let mut stored = stored(vec![folder("Clienti", vec![])]);
    let slot = SecretSlot::Property("Token".to_owned());
    set(&mut stored, &MemoryVault::default(), "/Clienti", slot).unwrap();
    let node = stored.doc.find(id_of(&stored.doc, "/Clienti")).unwrap();
    assert!(matches!(
        node.custom_properties[0].value,
        PropertyValue::Secret { .. }
    ));
}

#[test]
fn text_property_cannot_hold_a_secret() {
    let mut admin = credential("Admin", "admin", None);
    admin.custom_properties.push(CustomProperty {
        name: "Ticket".to_owned(),
        value: PropertyValue::Text {
            value: "INC-1".to_owned(),
        },
    });
    let mut stored = stored(vec![admin]);
    let slot = SecretSlot::Property("Ticket".to_owned());
    let result = set(&mut stored, &MemoryVault::default(), "/Admin", slot);
    assert!(matches!(result, Err(OpsError::PropertyIsText { .. })));
}

#[test]
fn clear_removes_the_reference_then_the_vault_entry() {
    let (vault, reference) = vault_with(SENTINEL);
    let mut stored = stored(vec![credential("Admin", "admin", Some(reference.clone()))]);
    let outcome = clear(&mut stored, &vault, "/Admin", SecretSlot::Password).unwrap();
    assert_eq!(outcome.secrets_deleted, 1);
    assert!(!vault.contains(&reference).unwrap());
    let saved = stored.reloaded();
    let admin = saved.find(id_of(&saved, "/Admin")).unwrap();
    assert_eq!(admin.as_credential().unwrap().password, None);
}

#[test]
fn clear_of_an_unset_secret_is_an_error() {
    let mut stored = stored(vec![credential("Admin", "admin", None)]);
    let result = clear(
        &mut stored,
        &MemoryVault::default(),
        "/Admin",
        SecretSlot::Password,
    );
    assert!(matches!(result, Err(OpsError::SecretNotSet { .. })));
}

#[test]
fn clear_of_a_secret_property_removes_the_property() {
    let mut stored = stored(vec![credential("Admin", "admin", None)]);
    let vault = MemoryVault::default();
    let slot = SecretSlot::Property("Token".to_owned());
    set(&mut stored, &vault, "/Admin", slot.clone()).unwrap();
    clear(&mut stored, &vault, "/Admin", slot).unwrap();
    let node = stored.doc.find(id_of(&stored.doc, "/Admin")).unwrap();
    assert!(node.custom_properties.is_empty());
}

#[test]
fn status_reports_presence_of_each_secret_of_the_node() {
    let (vault, present) = vault_with(SENTINEL);
    let missing = SecretRef::new();
    let mut admin = credential("Admin", "admin", Some(present.clone()));
    if let Some(credential) = admin.as_credential_mut() {
        credential.passphrase = Some(missing.clone());
    }
    let doc = document(vec![admin]);
    let status = secret_status(&doc, &vault, id_of(&doc, "/Admin")).unwrap();
    assert_eq!(
        status,
        vec![
            SecretStatus {
                slot: SecretSlot::Password,
                reference: present,
                present: true
            },
            SecretStatus {
                slot: SecretSlot::Passphrase,
                reference: missing,
                present: false
            },
        ]
    );
}

#[test]
fn check_tells_whether_a_slot_can_hold_a_secret() {
    let doc = document(vec![
        credential("Admin", "admin", None),
        connection("web01", "ssh"),
    ]);
    let admin = id_of(&doc, "/Admin");
    let web01 = id_of(&doc, "/web01");
    assert!(check_secret_slot(&doc, admin, &SecretSlot::Passphrase).is_ok());
    assert!(matches!(
        check_secret_slot(&doc, web01, &SecretSlot::Password),
        Err(OpsError::SecretNotApplicable { .. })
    ));
}
