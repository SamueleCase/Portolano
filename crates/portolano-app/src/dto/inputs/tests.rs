use portolano_core::{CustomProperty, Node};
use pretty_assertions::assert_eq;
use serde_json::json;

use super::*;

fn edit_of(json: serde_json::Value) -> NodeEditInput {
    serde_json::from_value(json).unwrap()
}

fn web01() -> Node {
    let mut node = Node::connection("web01", "ssh", "web01.example.com");
    node.tags = vec!["a".to_owned(), "b".to_owned()];
    node.custom_properties = vec![
        CustomProperty {
            name: "rack".to_owned(),
            value: PropertyValue::Text {
                value: "R1".to_owned(),
            },
        },
        CustomProperty {
            name: "token".to_owned(),
            value: PropertyValue::Secret {
                secret: portolano_core::SecretRef::new(),
            },
        },
    ];
    if let Some(connection) = node.as_connection_mut() {
        connection
            .settings
            .insert("agent_forwarding".to_owned(), json!(true));
    }
    node
}

#[test]
fn credential_by_id_is_resolved_as_a_path_by_id() {
    let id = Uuid::from_u128(5);

    let input = CredentialChoice::ById { id }.into_input();

    assert_eq!(input, CredentialInput::ByPath(format!("@{id}")));
}

#[test]
fn new_tag_list_becomes_tags_to_add_and_to_remove() {
    let (edit, _) = edit_of(json!({ "details": { "tags": ["b", "c"] } })).into_edit(&web01());

    assert_eq!(edit.details.add_tags, vec!["c"]);
    assert_eq!(edit.details.remove_tags, vec!["a"]);
}

#[test]
fn text_properties_left_out_are_removed_and_secret_ones_stay() {
    let (edit, _) = edit_of(json!({ "details": { "properties": [] } })).into_edit(&web01());

    assert_eq!(edit.details.remove_properties, vec!["rack"]);
}

#[test]
fn new_settings_replace_the_old_ones() {
    let (edit, _) = edit_of(json!({ "settings": { "x11": true } })).into_edit(&web01());

    assert_eq!(edit.settings_set, vec![("x11".to_owned(), json!(true))]);
    assert_eq!(edit.settings_unset, vec!["agent_forwarding"]);
}

#[test]
fn staged_secrets_become_editor_changes() {
    let (_, changes) = edit_of(json!({
        "secrets": {
            "password": { "action": "set", "value": "sentinel-input" },
            "passphrase": { "action": "clear" },
        },
        "defaultPort": true,
    }))
    .into_edit(&web01());

    assert!(
        matches!(changes.password, Some(SecretChange::Set(secret)) if secret.expose() == "sentinel-input")
    );
    assert!(matches!(changes.passphrase, Some(SecretChange::Clear)));
    assert!(changes.default_port);
}

#[test]
fn inline_choice_on_a_credential_that_is_not_inline_replaces_it() {
    let (edit, _) =
        edit_of(json!({ "credential": { "mode": "inline", "username": "ops", "domain": null } }))
            .into_edit(&web01());

    assert_eq!(
        edit.credential,
        Some(CredentialInput::Inline {
            username: Some("ops".to_owned()),
            domain: None
        })
    );
}

#[test]
fn new_node_path_is_built_from_the_parent_and_an_escaped_name() {
    let mut doc = Document::new("Test");
    let folder = Node::folder("Clienti");
    let parent = folder.id;
    doc.nodes.push(folder);
    let input: NewNodeInput = serde_json::from_value(json!({
        "parent": parent,
        "name": "a/b",
        "kind": { "type": "credential", "username": null, "domain": null, "keyFile": null },
    }))
    .unwrap();

    let node = input.into_new_node(&doc).unwrap();

    assert_eq!(node.path, "/Clienti/a\\/b");
}
