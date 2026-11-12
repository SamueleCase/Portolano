use portolano_core::vault::MemoryVault;
use portolano_core::{CredentialSpec, CustomProperty, PropertyValue};
use pretty_assertions::assert_eq;

use super::*;
use crate::testutil::*;

fn names(doc: &Document) -> Vec<String> {
    doc.nodes.iter().map(|node| node.name.clone()).collect()
}

fn duplicate(stored: &mut Stored, vault: &dyn Vault, path: &str) -> Result<DuplicateOutcome> {
    let id = id_of(&stored.doc, path);
    duplicate_node(&stored.store, &mut stored.doc, vault, id)
}

#[test]
fn copy_is_saved_right_after_the_original_with_a_new_name() {
    let mut stored = stored(vec![connection("web01", "ssh"), connection("web02", "ssh")]);

    let outcome = duplicate(&mut stored, &MemoryVault::default(), "/web01").unwrap();

    assert_eq!(outcome.path, "/web01 (copia)");
    assert_eq!(
        names(&stored.reloaded()),
        vec!["web01", "web01 (copia)", "web02"]
    );
}

#[test]
fn second_copy_gets_a_numbered_name() {
    let mut stored = stored(vec![connection("web01", "ssh")]);
    let vault = MemoryVault::default();
    duplicate(&mut stored, &vault, "/web01").unwrap();

    let outcome = duplicate(&mut stored, &vault, "/web01").unwrap();

    assert_eq!(outcome.path, "/web01 (copia 2)");
}

#[test]
fn copy_of_a_folder_has_new_ids_everywhere() {
    let mut stored = stored(vec![folder("Prod", vec![connection("web01", "ssh")])]);
    let original_child = id_of(&stored.doc, "/Prod/web01");

    duplicate(&mut stored, &MemoryVault::default(), "/Prod").unwrap();

    let copy_child = id_of(&stored.doc, "/Prod (copia)/web01");
    assert_ne!(copy_child, original_child);
    assert!(stored.doc.validate().is_empty());
}

#[test]
fn references_inside_the_copy_point_to_the_copy() {
    let admin = credential("Admin", "admin", None);
    let admin_id = admin.id;
    let web01 = connection_with("web01", "ssh", CredentialSpec::ById { id: admin_id });
    let mut stored = stored(vec![folder("Prod", vec![admin, web01])]);

    duplicate(&mut stored, &MemoryVault::default(), "/Prod").unwrap();

    let copy_admin = id_of(&stored.doc, "/Prod (copia)/Admin");
    let copy_web01 = stored
        .doc
        .find(id_of(&stored.doc, "/Prod (copia)/web01"))
        .unwrap();
    assert_eq!(
        copy_web01.as_connection().unwrap().credential,
        CredentialSpec::ById { id: copy_admin }
    );
}

#[test]
fn secrets_are_copied_under_new_references() {
    let (vault, reference) = vault_with(SENTINEL);
    let mut stored = stored(vec![credential("Admin", "admin", Some(reference.clone()))]);

    let outcome = duplicate(&mut stored, &vault, "/Admin").unwrap();

    let copy = stored.doc.find(outcome.id).unwrap();
    let copied = copy.as_credential().unwrap().password.clone().unwrap();
    assert_ne!(copied, reference);
    assert_eq!(vault.get(&copied).unwrap().unwrap().expose(), SENTINEL);
    assert_eq!(outcome.secrets_copied, 1);
    assert!(!stored.file_text().contains(SENTINEL));
}

#[test]
fn secret_properties_get_new_references_too() {
    let (vault, reference) = vault_with(SENTINEL);
    let mut node = connection("web01", "ssh");
    node.custom_properties.push(CustomProperty {
        name: "token".to_owned(),
        value: PropertyValue::Secret {
            secret: reference.clone(),
        },
    });
    let mut stored = stored(vec![node]);

    let outcome = duplicate(&mut stored, &vault, "/web01").unwrap();

    let copy = stored.doc.find(outcome.id).unwrap();
    assert!(!matches!(
        &copy.custom_properties[0].value,
        PropertyValue::Secret { secret } if *secret == reference
    ));
    assert!(stored.doc.validate().is_empty());
}

#[test]
fn secret_missing_from_the_vault_stays_missing_in_the_copy() {
    let mut stored = stored(vec![credential("Admin", "admin", Some(SecretRef::new()))]);

    let outcome = duplicate(&mut stored, &MemoryVault::default(), "/Admin").unwrap();

    assert_eq!(outcome.secrets_copied, 0);
    assert_eq!(outcome.secrets_missing, 1);
}

#[test]
fn failed_save_leaves_document_and_vault_unchanged() {
    let vault = RecordingVault::default();
    let reference = SecretRef::new();
    vault.set(&reference, &secret(SENTINEL)).unwrap();
    let mut stored = stored(vec![credential("Admin", "admin", Some(reference))]);
    stored.modify_externally();

    let result = duplicate(&mut stored, &vault, "/Admin");

    assert!(result.is_err());
    assert_eq!(names(&stored.doc), vec!["Admin"]);
    let (written, deleted) = vault.history();
    assert_eq!(deleted, written[1..].to_vec());
}

#[test]
fn duplicate_of_a_missing_node_is_not_found() {
    let mut stored = stored(vec![]);
    let id = Uuid::new_v4();

    let result = duplicate_node(&stored.store, &mut stored.doc, &MemoryVault::default(), id);

    assert!(matches!(
        result,
        Err(crate::OpsError::Core(
            portolano_core::Error::NodeNotFound { .. }
        ))
    ));
}

fn by_name(name: &str) -> CredentialSpec {
    CredentialSpec::ByName {
        name: name.to_owned(),
    }
}

/// `/Prod` with `Admin` and `web01` that names it, and `/db01` outside that
/// names it too.
fn named_credentials() -> Stored {
    stored(vec![
        folder(
            "Prod",
            vec![
                credential("Admin", "admin", None),
                connection_with("web01", "ssh", by_name("Admin")),
            ],
        ),
        connection_with("db01", "ssh", by_name("admin")),
    ])
}

fn credential_of(doc: &Document, path: &str) -> Uuid {
    match doc.resolve_credential(id_of(doc, path)).unwrap().origin {
        portolano_core::CredentialOrigin::Credential { node } => node,
        other => panic!("{path} has no credential node: {other:?}"),
    }
}

#[test]
fn copy_leaves_every_reference_by_name_as_it_resolved() {
    let mut stored = named_credentials();
    let admin = id_of(&stored.doc, "/Prod/Admin");

    duplicate(&mut stored, &MemoryVault::default(), "/Prod").unwrap();

    let doc = stored.reloaded();
    assert!(
        doc.validate()
            .iter()
            .all(|issue| issue.code != "by_name_ambiguous"),
        "{:?}",
        doc.validate()
    );
    assert_eq!(credential_of(&doc, "/Prod/web01"), admin);
    assert_eq!(credential_of(&doc, "/db01"), admin);
}

#[test]
fn copy_uses_its_own_credential_by_id() {
    let mut stored = named_credentials();

    duplicate(&mut stored, &MemoryVault::default(), "/Prod").unwrap();

    let copied_admin = id_of(&stored.doc, "/Prod (copia)/Admin (copia)");
    assert_eq!(
        credential_of(&stored.doc, "/Prod (copia)/web01"),
        copied_admin
    );
    let web01 = stored
        .doc
        .find(id_of(&stored.doc, "/Prod (copia)/web01"))
        .unwrap();
    assert_eq!(
        web01.as_connection().unwrap().credential,
        CredentialSpec::ById { id: copied_admin }
    );
}

#[test]
fn renamed_credentials_are_listed() {
    let mut stored = named_credentials();

    let outcome = duplicate(&mut stored, &MemoryVault::default(), "/Prod").unwrap();

    let renamed: Vec<&str> = outcome
        .renamed
        .iter()
        .map(|node| node.path.as_str())
        .collect();
    assert_eq!(renamed, vec!["/Prod (copia)/Admin (copia)"]);
}

#[test]
fn copied_credential_nobody_names_keeps_its_name() {
    let mut stored = stored(vec![folder(
        "Prod",
        vec![credential("Admin", "admin", None)],
    )]);

    let outcome = duplicate(&mut stored, &MemoryVault::default(), "/Prod").unwrap();

    assert!(outcome.renamed.is_empty());
    id_of(&stored.doc, "/Prod (copia)/Admin");
}

#[test]
fn copy_of_a_named_credential_gets_a_free_name() {
    let mut stored = stored(vec![
        credential("Admin", "admin", None),
        credential("Admin (copia)", "altro", None),
        connection_with("web01", "ssh", by_name("Admin")),
    ]);

    let outcome = duplicate(&mut stored, &MemoryVault::default(), "/Admin").unwrap();

    assert_eq!(outcome.path, "/Admin (copia 2)");
    assert_eq!(
        credential_of(&stored.doc, "/web01"),
        id_of(&stored.doc, "/Admin")
    );
}

#[test]
fn copy_keeps_no_import_origin() {
    let mut node = connection("web01", "ssh");
    node.source = Some(portolano_core::SourceInfo {
        system: "royal".to_owned(),
        source_type: "RoyalSSHConnection".to_owned(),
        source_id: Some("royal-1".to_owned()),
        file: None,
        imported_at: chrono::Utc::now(),
    });
    let mut stored = stored(vec![node]);

    let outcome = duplicate(&mut stored, &MemoryVault::default(), "/web01").unwrap();

    assert_eq!(stored.doc.find(outcome.id).unwrap().source, None);
}

#[test]
fn copy_with_secrets_needs_an_open_vault() {
    let mut stored = stored(vec![credential("Admin", "admin", Some(SecretRef::new()))]);
    let dir = tempfile::tempdir().unwrap();
    let locked = crate::vault::LazyVault::new(
        portolano_core::VaultSelection::File {
            path: fast_file_vault(dir.path()),
        },
        Box::new(NoMasterPassword),
    );

    let result = duplicate(&mut stored, &locked, "/Admin");

    assert_eq!(
        result.map_err(|error| error.category()).unwrap_err(),
        crate::ErrorCategory::VaultLocked
    );
    assert_eq!(names(&stored.reloaded()), vec!["Admin"]);
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
