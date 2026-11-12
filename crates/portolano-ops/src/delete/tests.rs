use portolano_core::vault::MemoryVault;
use portolano_core::{CredentialSpec, SecretRef};
use pretty_assertions::assert_eq;
use serde_json::json;

use super::*;
use crate::testutil::*;

fn by_id(id: Uuid) -> CredentialSpec {
    CredentialSpec::ById { id }
}

fn remove(
    stored: &mut Stored,
    vault: &dyn Vault,
    path: &str,
    options: DeleteOptions,
) -> Result<DeleteOutcome> {
    let id = id_of(&stored.doc, path);
    delete(&stored.store, &mut stored.doc, vault, id, options)
}

/// A bastion used as jump host by a connection in another folder.
fn with_jump_host() -> Vec<portolano_core::Node> {
    let bastion = connection("bastion", "ssh");
    let mut web01 = connection("web01", "ssh");
    if let Some(connection) = web01.as_connection_mut() {
        connection
            .settings
            .insert("jump_hosts".to_owned(), json!([{ "$ref": bastion.id }]));
    }
    vec![folder("Rete", vec![bastion]), web01]
}

#[test]
fn preview_counts_descendants_and_secrets() {
    let doc = document(vec![folder(
        "Clienti",
        vec![
            credential("Admin", "admin", Some(SecretRef::new())),
            connection("web01", "ssh"),
        ],
    )]);
    let preview = preview_delete(&doc, id_of(&doc, "/Clienti")).unwrap();
    assert_eq!(
        (preview.children, preview.descendants, preview.secrets),
        (2, 2, 1)
    );
    assert!(preview.references.is_empty());
}

#[test]
fn preview_lists_references_from_outside_the_subtree_only() {
    let doc = document(with_jump_host());
    let preview = preview_delete(&doc, id_of(&doc, "/Rete")).unwrap();
    assert_eq!(preview.references.len(), 1);
    assert_eq!(preview.references[0].from_path, "/web01");
    assert_eq!(preview.references[0].target_path, "/Rete/bastion");
}

#[test]
fn non_empty_folder_needs_recursive() {
    let mut stored = stored(vec![folder("Clienti", vec![connection("web01", "ssh")])]);
    let result = remove(
        &mut stored,
        &MemoryVault::default(),
        "/Clienti",
        DeleteOptions::default(),
    );
    assert!(matches!(
        result,
        Err(OpsError::FolderNotEmpty { children: 1, .. })
    ));
}

#[test]
fn referenced_credential_is_refused_without_force() {
    let admin = credential("Admin", "admin", None);
    let web01 = connection_with("web01", "ssh", by_id(admin.id));
    let mut stored = stored(vec![admin, web01]);
    let result = remove(
        &mut stored,
        &MemoryVault::default(),
        "/Admin",
        DeleteOptions::default(),
    );
    assert!(
        matches!(result, Err(OpsError::Referenced { references, .. }) if references.len() == 1)
    );
    assert_eq!(stored.reloaded().nodes.len(), 2);
}

#[test]
fn referenced_jump_host_is_refused_without_force() {
    let mut stored = stored(with_jump_host());
    let options = DeleteOptions {
        recursive: true,
        force: false,
    };
    let result = remove(&mut stored, &MemoryVault::default(), "/Rete", options);
    assert!(matches!(result, Err(OpsError::Referenced { .. })));
}

#[test]
fn forced_deletion_of_a_credential_makes_users_ask_at_launch() {
    let admin = credential("Admin", "admin", None);
    let web01 = connection_with("web01", "ssh", by_id(admin.id));
    let web01_id = web01.id;
    let mut stored = stored(vec![admin, web01]);
    let options = DeleteOptions {
        recursive: false,
        force: true,
    };
    let outcome = remove(&mut stored, &MemoryVault::default(), "/Admin", options).unwrap();
    assert_eq!(outcome.detached, vec![web01_id]);
    let saved = stored.reloaded();
    let web01 = saved.find(web01_id).unwrap();
    assert_eq!(
        web01.as_connection().unwrap().credential,
        CredentialSpec::Prompt
    );
}

#[test]
fn forced_deletion_of_a_jump_host_leaves_the_reference_dangling() {
    let mut stored = stored(with_jump_host());
    let options = DeleteOptions {
        recursive: true,
        force: true,
    };
    let outcome = remove(&mut stored, &MemoryVault::default(), "/Rete", options).unwrap();
    assert!(outcome.detached.is_empty());
    let codes: Vec<_> = stored
        .reloaded()
        .validate()
        .into_iter()
        .map(|issue| issue.code)
        .collect();
    assert!(codes.contains(&"dangling_ref"));
}

#[test]
fn secrets_of_the_subtree_are_deleted_after_saving() {
    let (vault, reference) = vault_with(SENTINEL);
    let mut stored = stored(vec![folder(
        "Clienti",
        vec![credential("Admin", "admin", Some(reference.clone()))],
    )]);
    let options = DeleteOptions {
        recursive: true,
        force: false,
    };
    let outcome = remove(&mut stored, &vault, "/Clienti", options).unwrap();
    assert_eq!((outcome.removed_nodes, outcome.secrets_deleted), (2, 1));
    assert!(!vault.contains(&reference).unwrap());
    assert!(stored.reloaded().nodes.is_empty());
}

#[test]
fn failed_save_keeps_nodes_and_secrets() {
    let (vault, reference) = vault_with(SENTINEL);
    let mut stored = stored(vec![credential("Admin", "admin", Some(reference.clone()))]);
    stored.modify_externally();
    let result = remove(&mut stored, &vault, "/Admin", DeleteOptions::default());
    assert!(result.is_err());
    assert!(vault.contains(&reference).unwrap());
    assert_eq!(stored.doc.nodes.len(), 1);
}

#[test]
fn forced_deletion_of_two_credentials_used_by_one_node_leaves_no_reference() {
    let first = credential("Admin", "admin", None);
    let second = credential("Gateway", "gw", None);
    let mut dc01 = connection_with("dc01", "rdp", by_id(first.id));
    if let Some(connection) = dc01.as_connection_mut() {
        connection.settings.insert(
            "gateway".to_owned(),
            json!({ "host": "gw.example.com", "credential": { "mode": "by_id", "$ref": second.id } }),
        );
    }
    let dc01_id = dc01.id;
    let mut stored = stored(vec![folder("Credenziali", vec![first, second]), dc01]);
    let preview = preview_delete(&stored.doc, id_of(&stored.doc, "/Credenziali")).unwrap();
    assert_eq!(preview.references.len(), 2);
    let options = DeleteOptions {
        recursive: true,
        force: true,
    };
    let outcome = remove(
        &mut stored,
        &MemoryVault::default(),
        "/Credenziali",
        options,
    )
    .unwrap();
    assert_eq!(outcome.detached, vec![dc01_id]);
    assert!(outcome.remaining_references.is_empty());
}

#[test]
fn forced_deletion_reports_the_references_left_dangling() {
    let mut stored = stored(with_jump_host());
    let options = DeleteOptions {
        recursive: true,
        force: true,
    };
    let outcome = remove(&mut stored, &MemoryVault::default(), "/Rete", options).unwrap();
    assert_eq!(outcome.remaining_references.len(), 1);
    assert_eq!(outcome.remaining_references[0].from_path, "/web01");
}

#[test]
fn secret_shared_with_a_node_outside_the_subtree_stays_in_the_vault() {
    let (vault, shared) = vault_with(SENTINEL);
    let mut stored = stored(vec![
        folder(
            "Vecchi",
            vec![credential("Admin", "admin", Some(shared.clone()))],
        ),
        credential("Copia", "admin", Some(shared.clone())),
    ]);
    let options = DeleteOptions {
        recursive: true,
        force: false,
    };
    let outcome = remove(&mut stored, &vault, "/Vecchi", options).unwrap();
    assert_eq!(outcome.secrets_deleted, 0);
    assert!(vault.contains(&shared).unwrap());
}

#[test]
fn forced_deletion_detaches_references_by_name() {
    let admin = credential("Admin", "admin", None);
    let web01 = connection_with(
        "web01",
        "ssh",
        CredentialSpec::ByName {
            name: "Admin".to_owned(),
        },
    );
    let web01_id = web01.id;
    let mut stored = stored(vec![admin, web01]);
    let options = DeleteOptions {
        recursive: false,
        force: true,
    };
    let outcome = remove(&mut stored, &MemoryVault::default(), "/Admin", options).unwrap();
    assert_eq!(outcome.detached, vec![web01_id]);
    let web01 = stored.doc.find(web01_id).unwrap();
    assert_eq!(
        web01.as_connection().unwrap().credential,
        CredentialSpec::Prompt
    );
}

/// A master password source that must never be asked.
struct NeverAsked;

impl portolano_core::MasterPasswordSource for NeverAsked {
    fn master_password(
        &self,
        _creating: bool,
    ) -> Result<portolano_core::Secret, portolano_core::VaultError> {
        panic!("the vault must not be opened");
    }
}

#[test]
fn deletion_without_secrets_does_not_open_the_vault() {
    let dir = tempfile::tempdir().unwrap();
    let vault = crate::vault::LazyVault::new(
        portolano_core::VaultSelection::File {
            path: dir.path().join("vault.json"),
        },
        Box::new(NeverAsked),
    );
    let mut stored = stored(vec![folder("Vuota", vec![])]);
    remove(&mut stored, &vault, "/Vuota", DeleteOptions::default()).unwrap();
    assert!(!vault.is_open());
}

#[test]
fn preview_lists_the_connections_that_inherit_a_credential_of_the_subtree() {
    let admin = credential("Admin", "admin", None);
    let mut prod = folder("Prod", vec![connection("web01", "ssh")]);
    if let Some(folder) = prod.as_folder_mut() {
        folder.credential = by_id(admin.id);
    }
    let doc = document(vec![admin, prod]);

    let preview = preview_delete(&doc, id_of(&doc, "/Admin")).unwrap();

    let inherited: Vec<&str> = preview
        .inherited
        .iter()
        .map(|node| node.path.as_str())
        .collect();
    assert_eq!(inherited, vec!["/Prod/web01"]);
}
