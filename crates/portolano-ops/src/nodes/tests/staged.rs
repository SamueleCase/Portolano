//! The staged changes of the editor of the app ([`edit_node_staged`]).

use portolano_core::vault::MemoryVault;
use portolano_core::{CredentialSpec, PrivateKey, SecretRef};
use pretty_assertions::assert_eq;

use super::*;

fn stage(
    stored: &mut Stored,
    vault: &dyn Vault,
    path: &str,
    changes: EditorChanges,
) -> Result<EditOutcome> {
    let id = id_of(&stored.doc, path);
    edit_node_staged(
        &stored.store,
        &mut stored.doc,
        vault,
        &Registry::with_defaults(),
        id,
        NodeEdit::default(),
        changes,
    )
}

fn set(value: &str) -> Option<SecretChange> {
    Some(SecretChange::Set(secret(value)))
}

fn password_of(stored: &Stored, path: &str) -> Option<SecretRef> {
    node(stored, path).as_credential().unwrap().password.clone()
}

#[test]
fn replaced_password_gets_a_new_reference_and_the_old_one_is_deleted_after_saving() {
    let (vault, old) = vault_with("sentinel-old");
    let mut stored = stored(vec![credential("Admin", "admin", Some(old.clone()))]);

    let outcome = stage(
        &mut stored,
        &vault,
        "/Admin",
        EditorChanges {
            password: set(SENTINEL),
            ..EditorChanges::default()
        },
    )
    .unwrap();

    let new = password_of(&stored, "/Admin").unwrap();
    assert_ne!(new, old);
    assert_eq!(vault.get(&new).unwrap().unwrap().expose(), SENTINEL);
    assert!(!vault.contains(&old).unwrap());
    assert_eq!((outcome.secrets_written, outcome.secrets_deleted), (1, 1));
    assert!(!stored.file_text().contains(SENTINEL));
}

#[test]
fn failed_save_keeps_the_old_password_and_writes_nothing() {
    let (vault, old) = vault_with("sentinel-old");
    let mut stored = stored(vec![credential("Admin", "admin", Some(old.clone()))]);
    stored.modify_externally();

    let result = stage(
        &mut stored,
        &vault,
        "/Admin",
        EditorChanges {
            password: set(SENTINEL),
            ..EditorChanges::default()
        },
    );

    assert!(result.is_err());
    assert_eq!(password_of(&stored, "/Admin"), Some(old.clone()));
    assert!(vault.contains(&old).unwrap());
}

#[test]
fn cleared_password_is_deleted_after_saving() {
    let (vault, old) = vault_with("sentinel-old");
    let mut stored = stored(vec![credential("Admin", "admin", Some(old.clone()))]);

    stage(
        &mut stored,
        &vault,
        "/Admin",
        EditorChanges {
            password: Some(SecretChange::Clear),
            ..EditorChanges::default()
        },
    )
    .unwrap();

    assert_eq!(password_of(&stored, "/Admin"), None);
    assert!(!vault.contains(&old).unwrap());
}

#[test]
fn clearing_a_password_that_is_not_set_is_refused() {
    let mut stored = stored(vec![credential("Admin", "admin", None)]);

    let result = stage(
        &mut stored,
        &MemoryVault::default(),
        "/Admin",
        EditorChanges {
            password: Some(SecretChange::Clear),
            ..EditorChanges::default()
        },
    );

    assert!(matches!(result, Err(OpsError::SecretNotSet { .. })));
}

#[test]
fn password_of_an_inheriting_connection_makes_its_credential_inline() {
    let vault = MemoryVault::default();
    let mut stored = stored(vec![connection("web01", "ssh")]);

    stage(
        &mut stored,
        &vault,
        "/web01",
        EditorChanges {
            password: set(SENTINEL),
            ..EditorChanges::default()
        },
    )
    .unwrap();

    assert!(matches!(
        spec_of(&stored, "/web01"),
        CredentialSpec::Inline {
            password: Some(_),
            ..
        }
    ));
}

#[test]
fn password_of_a_connection_with_a_referenced_credential_is_refused() {
    let mut stored = stored(vec![connection_with(
        "web01",
        "ssh",
        CredentialSpec::Prompt,
    )]);

    let result = stage(
        &mut stored,
        &MemoryVault::default(),
        "/web01",
        EditorChanges {
            password: set(SENTINEL),
            ..EditorChanges::default()
        },
    );

    assert!(matches!(result, Err(OpsError::SecretNotApplicable { .. })));
}

#[test]
fn passphrase_of_a_connection_is_refused() {
    let mut stored = stored(vec![connection("web01", "ssh")]);

    let result = stage(
        &mut stored,
        &MemoryVault::default(),
        "/web01",
        EditorChanges {
            passphrase: set(SENTINEL),
            ..EditorChanges::default()
        },
    );

    assert!(matches!(result, Err(OpsError::SecretNotApplicable { .. })));
}

#[test]
fn key_in_the_vault_replaces_a_key_file() {
    let vault = MemoryVault::default();
    let mut stored = stored(vec![credential("Admin", "admin", None)]);
    stage(
        &mut stored,
        &vault,
        "/Admin",
        EditorChanges {
            key_file: Some("~/.ssh/id_ed25519".to_owned()),
            ..EditorChanges::default()
        },
    )
    .unwrap();

    stage(
        &mut stored,
        &vault,
        "/Admin",
        EditorChanges {
            private_key: set(SENTINEL),
            ..EditorChanges::default()
        },
    )
    .unwrap();

    let key = node(&stored, "/Admin")
        .as_credential()
        .unwrap()
        .private_key
        .clone();
    assert!(matches!(key, Some(PrivateKey::Vault { .. })));
}

#[test]
fn key_file_and_key_in_the_vault_together_are_refused() {
    let mut stored = stored(vec![credential("Admin", "admin", None)]);

    let result = stage(
        &mut stored,
        &MemoryVault::default(),
        "/Admin",
        EditorChanges {
            private_key: set(SENTINEL),
            key_file: Some("~/.ssh/id_ed25519".to_owned()),
            ..EditorChanges::default()
        },
    );

    assert!(matches!(result, Err(OpsError::KeyFileAndVaultKey)));
}

#[test]
fn default_port_removes_the_port() {
    let mut stored = stored(vec![connection("web01", "ssh")]);
    edit(
        &mut stored,
        &MemoryVault::default(),
        "/web01",
        NodeEdit {
            port: Some(2222),
            ..NodeEdit::default()
        },
    )
    .unwrap();

    stage(
        &mut stored,
        &MemoryVault::default(),
        "/web01",
        EditorChanges {
            default_port: true,
            ..EditorChanges::default()
        },
    )
    .unwrap();

    assert_eq!(node(&stored, "/web01").as_connection().unwrap().port, None);
}

#[test]
fn stored_image_becomes_the_icon() {
    let mut stored = stored(vec![connection("web01", "ssh")]);
    let icon = stored.store.icons().add(PNG, "image/png").unwrap();
    let portolano_core::Icon::Custom { sha256, mime } = icon.clone() else {
        panic!("custom icon expected");
    };

    edit(
        &mut stored,
        &MemoryVault::default(),
        "/web01",
        NodeEdit {
            details: NodeDetails {
                icon: Some(IconInput::Stored { sha256, mime }),
                ..NodeDetails::default()
            },
            ..NodeEdit::default()
        },
    )
    .unwrap();

    assert_eq!(node(&stored, "/web01").icon, Some(icon));
}

#[test]
fn image_that_the_store_does_not_have_is_refused() {
    let mut stored = stored(vec![connection("web01", "ssh")]);

    let result = edit(
        &mut stored,
        &MemoryVault::default(),
        "/web01",
        NodeEdit {
            details: NodeDetails {
                icon: Some(IconInput::Stored {
                    sha256: "ab".repeat(32),
                    mime: "image/png".to_owned(),
                }),
                ..NodeDetails::default()
            },
            ..NodeEdit::default()
        },
    );

    assert!(matches!(result, Err(OpsError::IconNotStored)));
    assert_eq!(node(&stored, "/web01").icon, None);
}
