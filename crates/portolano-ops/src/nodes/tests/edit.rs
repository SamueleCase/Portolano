use portolano_core::CredentialSpec;
use portolano_core::vault::MemoryVault;
use pretty_assertions::assert_eq;
use serde_json::json;

use super::*;

#[test]
fn rename_to_a_sibling_name_is_refused() {
    let mut stored = stored(vec![connection("web01", "ssh"), connection("web02", "ssh")]);
    let result = edit(
        &mut stored,
        &MemoryVault::default(),
        "/web02",
        NodeEdit {
            name: Some("web01".to_owned()),
            ..NodeEdit::default()
        },
    );
    assert!(matches!(result, Err(OpsError::NameTaken { .. })));
}

#[test]
fn edit_changes_host_port_and_settings() {
    let mut stored = stored(vec![connection("web01", "ssh")]);
    edit(
        &mut stored,
        &MemoryVault::default(),
        "/web01",
        NodeEdit {
            host: Some("192.0.2.10".to_owned()),
            port: Some(2222),
            settings_set: vec![("agent_forwarding".to_owned(), json!(true))],
            ..NodeEdit::default()
        },
    )
    .unwrap();
    let connection = node(&stored, "/web01").as_connection().unwrap().clone();
    assert_eq!(connection.host, "192.0.2.10");
    assert_eq!(connection.port, Some(2222));
    assert_eq!(
        connection.settings.get("agent_forwarding"),
        Some(&json!(true))
    );
}

#[test]
fn edit_with_invalid_settings_changes_nothing() {
    let mut stored = stored(vec![connection("web01", "ssh")]);
    let result = edit(
        &mut stored,
        &MemoryVault::default(),
        "/web01",
        NodeEdit {
            settings_set: vec![("x11".to_owned(), json!("yes"))],
            ..NodeEdit::default()
        },
    );
    assert!(matches!(result, Err(OpsError::InvalidSettings { .. })));
    assert!(
        node(&stored, "/web01")
            .as_connection()
            .unwrap()
            .settings
            .is_empty()
    );
}

#[test]
fn host_of_a_credential_is_not_editable() {
    let mut stored = stored(vec![credential("Admin", "admin", None)]);
    let result = edit(
        &mut stored,
        &MemoryVault::default(),
        "/Admin",
        NodeEdit {
            host: Some("web01.example.com".to_owned()),
            ..NodeEdit::default()
        },
    );
    assert!(matches!(
        result,
        Err(OpsError::NotEditable { field: "host", .. })
    ));
}

#[test]
fn user_on_a_connection_makes_its_credential_inline() {
    let mut stored = stored(vec![connection("web01", "ssh")]);
    edit(
        &mut stored,
        &MemoryVault::default(),
        "/web01",
        NodeEdit {
            username: Some("deploy".to_owned()),
            ..NodeEdit::default()
        },
    )
    .unwrap();
    assert_eq!(
        spec_of(&stored, "/web01"),
        CredentialSpec::Inline {
            username: Some("deploy".to_owned()),
            domain: None,
            password: None
        }
    );
}

#[test]
fn replacing_an_inline_credential_deletes_its_password_after_saving() {
    let (vault, reference) = vault_with(SENTINEL);
    let inline = CredentialSpec::Inline {
        username: Some("admin".to_owned()),
        domain: None,
        password: Some(reference.clone()),
    };
    let mut stored = stored(vec![connection_with("web01", "ssh", inline)]);
    let outcome = edit(
        &mut stored,
        &vault,
        "/web01",
        NodeEdit {
            credential: Some(CredentialInput::Prompt),
            ..NodeEdit::default()
        },
    )
    .unwrap();
    assert_eq!(outcome.secrets_deleted, 1);
    assert_eq!(spec_of(&stored, "/web01"), CredentialSpec::Prompt);
    assert!(!vault.contains(&reference).unwrap());
}

#[test]
fn secret_property_replacement_deletes_the_previous_value() {
    let mut stored = stored(vec![credential("Admin", "admin", None)]);
    let vault = MemoryVault::default();
    let set_token = |value: &str| NodeEdit {
        secret_properties: vec![("Token".to_owned(), secret(value))],
        ..NodeEdit::default()
    };
    edit(&mut stored, &vault, "/Admin", set_token("first")).unwrap();
    let outcome = edit(&mut stored, &vault, "/Admin", set_token(SENTINEL)).unwrap();
    assert_eq!((outcome.secrets_written, outcome.secrets_deleted), (1, 1));
    assert_eq!(node(&stored, "/Admin").custom_properties.len(), 1);
}

#[test]
fn tags_and_properties_are_removed() {
    let mut admin = credential("Admin", "admin", None);
    admin.tags = vec!["prod".to_owned(), "old".to_owned()];
    let mut stored = stored(vec![admin]);
    edit(
        &mut stored,
        &MemoryVault::default(),
        "/Admin",
        NodeEdit {
            details: NodeDetails {
                remove_tags: vec!["old".to_owned()],
                description: Some(String::new()),
                ..NodeDetails::default()
            },
            ..NodeEdit::default()
        },
    )
    .unwrap();
    assert_eq!(node(&stored, "/Admin").tags, vec!["prod".to_owned()]);
    assert_eq!(node(&stored, "/Admin").description, None);
}
