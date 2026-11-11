use portolano_core::vault::MemoryVault;
use portolano_core::{CredentialSpec, Icon, PrivateKey, PropertyValue};
use pretty_assertions::assert_eq;
use serde_json::json;

use super::*;

#[test]
fn credential_password_goes_to_the_vault_and_never_to_the_document_file() {
    let mut stored = stored(vec![]);
    let vault = MemoryVault::default();
    let mut new = cred("/Admin");
    new.secrets.password = Some(secret(SENTINEL));
    add(&mut stored, &vault, new).unwrap();
    let reference = node(&stored, "/Admin")
        .as_credential()
        .unwrap()
        .password
        .clone()
        .unwrap();
    assert!(!stored.file_text().contains(SENTINEL));
    assert!(stored.file_text().contains(reference.as_str()));
    assert_eq!(vault.get(&reference).unwrap().unwrap().expose(), SENTINEL);
}

#[test]
fn added_node_is_saved_and_reported() {
    let mut stored = stored(vec![folder("Clienti", vec![])]);
    let outcome = add(
        &mut stored,
        &MemoryVault::default(),
        ssh("/Clienti/web01", CredentialInput::Inherit),
    )
    .unwrap();
    assert_eq!(outcome.path, "/Clienti/web01");
    assert_eq!(
        stored.reloaded().path_of(outcome.id).unwrap(),
        "/Clienti/web01"
    );
}

#[test]
fn missing_parents_are_created_with_create_parents() {
    let mut stored = stored(vec![]);
    let mut new = ssh("/Clienti/ACME/web01", CredentialInput::Inherit);
    new.create_parents = true;
    add(&mut stored, &MemoryVault::default(), new).unwrap();
    assert!(node(&stored, "/Clienti/ACME").is_folder());
}

#[test]
fn missing_parent_is_an_error_without_create_parents() {
    let mut stored = stored(vec![]);
    let result = add(
        &mut stored,
        &MemoryVault::default(),
        ssh("/Clienti/web01", CredentialInput::Inherit),
    );
    assert!(matches!(
        result,
        Err(OpsError::Core(portolano_core::Error::NotFound { .. }))
    ));
}

#[test]
fn name_used_by_a_sibling_is_refused() {
    let mut stored = stored(vec![connection("web01", "ssh")]);
    let result = add(
        &mut stored,
        &MemoryVault::default(),
        ssh("/web01", CredentialInput::Inherit),
    );
    assert!(matches!(result, Err(OpsError::NameTaken { .. })));
}

#[test]
fn password_without_credential_makes_an_inline_credential() {
    let mut stored = stored(vec![]);
    let vault = MemoryVault::default();
    let mut new = ssh("/web01", CredentialInput::Inherit);
    new.secrets.password = Some(secret(SENTINEL));
    add(&mut stored, &vault, new).unwrap();
    let CredentialSpec::Inline {
        username,
        password: Some(reference),
        ..
    } = spec_of(&stored, "/web01")
    else {
        panic!("not inline with a password");
    };
    assert_eq!(username, None);
    assert_eq!(vault.get(&reference).unwrap().unwrap().expose(), SENTINEL);
}

#[test]
fn password_with_a_referenced_credential_is_refused() {
    let mut stored = stored(vec![credential("Admin", "admin", None)]);
    let vault = MemoryVault::default();
    let mut new = ssh("/web01", CredentialInput::ByPath("/Admin".to_owned()));
    new.secrets.password = Some(secret(SENTINEL));
    let result = add(&mut stored, &vault, new);
    assert!(matches!(result, Err(OpsError::SecretNotApplicable { .. })));
}

#[test]
fn credential_by_path_is_stored_by_id() {
    let mut stored = stored(vec![credential("Admin", "admin", None)]);
    let admin = id_of(&stored.doc, "/Admin");
    add(
        &mut stored,
        &MemoryVault::default(),
        ssh("/web01", CredentialInput::ByPath("/Admin".to_owned())),
    )
    .unwrap();
    assert_eq!(
        spec_of(&stored, "/web01"),
        CredentialSpec::ById { id: admin }
    );
}

#[test]
fn credential_path_to_another_kind_of_node_is_refused() {
    let mut stored = stored(vec![folder("Clienti", vec![])]);
    let result = add(
        &mut stored,
        &MemoryVault::default(),
        ssh("/web01", CredentialInput::ByPath("/Clienti".to_owned())),
    );
    assert!(matches!(result, Err(OpsError::NotACredential { .. })));
}

#[test]
fn unknown_protocol_is_refused() {
    let mut stored = stored(vec![]);
    let mut new = ssh("/vnc", CredentialInput::Inherit);
    if let NewNodeKind::Connection { protocol, .. } = &mut new.kind {
        *protocol = "vnc".to_owned();
    }
    let result = add(&mut stored, &MemoryVault::default(), new);
    assert!(matches!(result, Err(OpsError::UnknownProtocol { .. })));
}

#[test]
fn settings_rejected_by_the_protocol_are_refused() {
    let mut stored = stored(vec![]);
    let mut new = ssh("/web01", CredentialInput::Inherit);
    if let NewNodeKind::Connection { settings, .. } = &mut new.kind {
        settings.insert("keepalive_s".to_owned(), json!("often"));
    }
    let result = add(&mut stored, &MemoryVault::default(), new);
    assert!(matches!(result, Err(OpsError::InvalidSettings { .. })));
}

#[test]
fn credential_with_a_key_in_the_vault_and_a_passphrase() {
    let mut stored = stored(vec![]);
    let vault = MemoryVault::default();
    let mut new = cred("/Deploy");
    new.secrets.private_key = Some(secret("sentinel-key"));
    new.secrets.passphrase = Some(secret("sentinel-passphrase"));
    let outcome = add(&mut stored, &vault, new).unwrap();
    assert_eq!(outcome.secrets_written, 2);
    let credential = node(&stored, "/Deploy").as_credential().unwrap();
    assert!(matches!(
        credential.private_key,
        Some(PrivateKey::Vault { .. })
    ));
    assert!(credential.passphrase.is_some());
}

#[test]
fn key_file_and_key_in_the_vault_together_are_refused() {
    let mut stored = stored(vec![]);
    let mut new = cred("/Deploy");
    if let NewNodeKind::Credential { key_file, .. } = &mut new.kind {
        *key_file = Some("~/.ssh/id_ed25519".to_owned());
    }
    new.secrets.private_key = Some(secret("sentinel-key"));
    let result = add(&mut stored, &MemoryVault::default(), new);
    assert!(matches!(result, Err(OpsError::KeyFileAndVaultKey)));
}

#[test]
fn failed_save_leaves_no_secret_and_no_node() {
    let mut stored = stored(vec![]);
    let vault = MemoryVault::default();
    let mut new = cred("/Admin");
    new.secrets.password = Some(secret(SENTINEL));
    stored.modify_externally();
    let result = add(&mut stored, &vault, new);
    assert!(result.is_err());
    assert!(stored.doc.nodes.is_empty());
    assert!(crate::tree_util::document_secret_refs(&stored.doc).is_empty());
}

#[test]
fn common_details_are_applied() {
    let mut stored = stored(vec![]);
    let mut new = cred("/Admin");
    new.details = NodeDetails {
        description: Some("amministratore".to_owned()),
        color: Some("#1A2B3C".to_owned()),
        favorite: Some(true),
        add_tags: vec!["prod".to_owned()],
        set_properties: vec![("Ticket".to_owned(), "INC-1".to_owned())],
        icon: Some(IconInput::Builtin("server".to_owned())),
        ..NodeDetails::default()
    };
    new.secrets.properties = vec![("Token".to_owned(), secret(SENTINEL))];
    add(&mut stored, &MemoryVault::default(), new).unwrap();
    let admin = node(&stored, "/Admin");
    assert_eq!(admin.description.as_deref(), Some("amministratore"));
    assert_eq!(admin.color.as_deref(), Some("#1A2B3C"));
    assert!(admin.favorite);
    assert_eq!(admin.tags, vec!["prod".to_owned()]);
    assert_eq!(
        admin.icon,
        Some(Icon::Builtin {
            name: "server".to_owned()
        })
    );
    let token = admin
        .custom_properties
        .iter()
        .find(|property| property.name == "Token")
        .unwrap();
    assert!(matches!(token.value, PropertyValue::Secret { .. }));
    assert_eq!(admin.custom_properties.len(), 2);
}

#[test]
fn invalid_color_is_refused() {
    let mut stored = stored(vec![]);
    let mut new = cred("/Admin");
    new.details.color = Some("rosso".to_owned());
    let result = add(&mut stored, &MemoryVault::default(), new);
    assert!(matches!(result, Err(OpsError::InvalidColor { .. })));
}

#[test]
fn custom_image_icon_is_stored_next_to_the_document() {
    let mut stored = stored(vec![]);
    let mut new = cred("/Admin");
    new.details.icon = Some(IconInput::Image {
        bytes: PNG.to_vec(),
        declared_mime: "image/png".to_owned(),
    });
    add(&mut stored, &MemoryVault::default(), new).unwrap();
    let icon = node(&stored, "/Admin").icon.clone().unwrap();
    assert!(matches!(icon, Icon::Custom { .. }));
    assert!(stored.store.icons().path_for(&icon).unwrap().exists());
}
