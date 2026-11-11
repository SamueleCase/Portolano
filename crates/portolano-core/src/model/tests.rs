use pretty_assertions::assert_eq;
use serde_json::json;

use super::*;

const DOC_ID: Uuid = Uuid::from_u128(0x6f1c0000_0000_4000_8000_000000000001);
const FOLDER_ID: Uuid = Uuid::from_u128(0xa1000000_0000_4000_8000_000000000002);
const CONNECTION_ID: Uuid = Uuid::from_u128(0xb2000000_0000_4000_8000_000000000003);
const CREDENTIAL_ID: Uuid = Uuid::from_u128(0xc3000000_0000_4000_8000_000000000004);
const JUMP_ID: Uuid = Uuid::from_u128(0xd4000000_0000_4000_8000_000000000005);

fn at(timestamp: &str) -> DateTime<Utc> {
    timestamp.parse().unwrap()
}

fn fixed(mut node: Node, id: Uuid) -> Node {
    node.id = id;
    node.created = at("2026-11-10T10:00:00Z");
    node.modified = at("2026-11-10T10:05:00Z");
    node
}

/// The document of SPEC §3.1, with fixed ids and timestamps.
fn spec_example(password: &SecretRef) -> Document {
    let mut connection = fixed(
        Node::connection("web01", "ssh", "web01.example.com"),
        CONNECTION_ID,
    );
    connection.icon = Some(Icon::Royal {
        name: "Flat/Hardware/Server".to_owned(),
    });
    let web01 = connection.as_connection_mut().unwrap();
    web01.port = Some(2222);
    web01.settings.insert(
        "jump_hosts".to_owned(),
        json!([{ "$ref": JUMP_ID.to_string() }]),
    );
    web01
        .settings
        .insert("agent_forwarding".to_owned(), json!(true));

    let mut credential = fixed(Node::credential("Admin ACME"), CREDENTIAL_ID);
    let admin_acme = credential.as_credential_mut().unwrap();
    admin_acme.username = Some("admin".to_owned());
    admin_acme.domain = Some("ACME".to_owned());
    admin_acme.password = Some(password.clone());

    let mut folder = fixed(Node::folder("Clienti"), FOLDER_ID);
    let clienti = folder.as_folder_mut().unwrap();
    clienti.credential = CredentialSpec::ById { id: CREDENTIAL_ID };
    clienti.children = vec![connection, credential];

    let mut document = Document::new("Portolano");
    document.id = DOC_ID;
    document.created = at("2026-11-10T10:00:00Z");
    document.modified = at("2026-11-10T10:05:00Z");
    document.nodes = vec![folder];
    document
}

fn spec_example_json(password: &SecretRef) -> serde_json::Value {
    json!({
        "format": "portolano",
        "schema_version": 1,
        "id": DOC_ID.to_string(),
        "name": "Portolano",
        "created": "2026-11-10T10:00:00Z",
        "modified": "2026-11-10T10:05:00Z",
        "nodes": [{
            "id": FOLDER_ID.to_string(),
            "name": "Clienti",
            "created": "2026-11-10T10:00:00Z",
            "modified": "2026-11-10T10:05:00Z",
            "type": "folder",
            "credential": { "mode": "by_id", "$ref": CREDENTIAL_ID.to_string() },
            "children": [
                {
                    "id": CONNECTION_ID.to_string(),
                    "name": "web01",
                    "created": "2026-11-10T10:00:00Z",
                    "modified": "2026-11-10T10:05:00Z",
                    "icon": { "kind": "royal", "name": "Flat/Hardware/Server" },
                    "type": "connection",
                    "protocol": "ssh",
                    "host": "web01.example.com",
                    "port": 2222,
                    "settings": {
                        "jump_hosts": [{ "$ref": JUMP_ID.to_string() }],
                        "agent_forwarding": true
                    }
                },
                {
                    "id": CREDENTIAL_ID.to_string(),
                    "name": "Admin ACME",
                    "created": "2026-11-10T10:00:00Z",
                    "modified": "2026-11-10T10:05:00Z",
                    "type": "credential",
                    "username": "admin",
                    "domain": "ACME",
                    "password": password.as_str()
                }
            ]
        }]
    })
}

#[test]
fn document_serializes_to_the_spec_example_shape() {
    let password = SecretRef::new();
    let document = spec_example(&password);

    let value = serde_json::to_value(&document).unwrap();

    assert_eq!(value, spec_example_json(&password));
}

#[test]
fn document_round_trips_through_json_text() {
    let password = SecretRef::new();
    let document = spec_example(&password);

    let text = serde_json::to_string_pretty(&document).unwrap();
    let back: Document = serde_json::from_str(&text).unwrap();

    assert_eq!(back, document);
}

#[test]
fn spec_example_json_deserializes_to_the_model() {
    let password = SecretRef::new();

    let document: Document = serde_json::from_value(spec_example_json(&password)).unwrap();

    assert_eq!(document, spec_example(&password));
}

#[test]
fn inherit_credential_is_omitted_when_serialized() {
    let connection = Node::connection("web01", "ssh", "web01.example.com");
    let folder = Node::folder("Clienti");

    let connection_json = serde_json::to_value(&connection).unwrap();
    let folder_json = serde_json::to_value(&folder).unwrap();

    assert!(connection_json.get("credential").is_none());
    assert!(folder_json.get("credential").is_none());
}

#[test]
fn missing_credential_deserializes_as_inherit() {
    let json = json!({
        "id": CONNECTION_ID.to_string(),
        "name": "web01",
        "created": "2026-11-10T10:00:00Z",
        "modified": "2026-11-10T10:05:00Z",
        "type": "connection",
        "protocol": "ssh",
        "host": "web01.example.com"
    });

    let node: Node = serde_json::from_value(json).unwrap();

    let connection = node.as_connection().unwrap();
    assert_eq!(connection.credential, CredentialSpec::Inherit);
    assert_eq!(connection.port, None);
    assert!(connection.settings.is_empty());
}

#[test]
fn explicit_inherit_mode_deserializes_as_inherit() {
    let json = json!({ "mode": "inherit" });

    let spec: CredentialSpec = serde_json::from_value(json).unwrap();

    assert_eq!(spec, CredentialSpec::Inherit);
}

/// Asserts that `spec` serializes to `expected` and deserializes back to itself.
fn assert_credential_spec_shape(spec: CredentialSpec, expected: serde_json::Value) {
    let value = serde_json::to_value(&spec).unwrap();
    let back: CredentialSpec = serde_json::from_value(value.clone()).unwrap();

    assert_eq!(value, expected);
    assert_eq!(back, spec);
}

#[test]
fn none_credential_spec_is_tagged_none() {
    assert_credential_spec_shape(CredentialSpec::None, json!({ "mode": "none" }));
}

#[test]
fn prompt_credential_spec_is_tagged_prompt() {
    assert_credential_spec_shape(CredentialSpec::Prompt, json!({ "mode": "prompt" }));
}

#[test]
fn inline_credential_spec_omits_missing_fields() {
    let password = SecretRef::new();
    let spec = CredentialSpec::Inline {
        username: Some("admin".to_owned()),
        domain: None,
        password: Some(password.clone()),
    };

    assert_credential_spec_shape(
        spec,
        json!({ "mode": "inline", "username": "admin", "password": password.as_str() }),
    );
}

#[test]
fn by_id_credential_spec_uses_the_ref_key() {
    let spec = CredentialSpec::ById { id: CREDENTIAL_ID };

    assert_credential_spec_shape(
        spec,
        json!({ "mode": "by_id", "$ref": CREDENTIAL_ID.to_string() }),
    );
}

#[test]
fn by_name_credential_spec_is_tagged_by_name() {
    let spec = CredentialSpec::ByName {
        name: "Admin ACME".to_owned(),
    };

    assert_credential_spec_shape(spec, json!({ "mode": "by_name", "name": "Admin ACME" }));
}

#[test]
fn private_key_file_is_tagged_by_kind() {
    let key = PrivateKey::File {
        path: "~/.ssh/id_ed25519".to_owned(),
    };

    let value = serde_json::to_value(&key).unwrap();

    assert_eq!(
        value,
        json!({ "kind": "file", "path": "~/.ssh/id_ed25519" })
    );
}

#[test]
fn builtin_icon_is_tagged_by_kind() {
    let icon = Icon::Builtin {
        name: "server".to_owned(),
    };

    let value = serde_json::to_value(&icon).unwrap();

    assert_eq!(value, json!({ "kind": "builtin", "name": "server" }));
}

#[test]
fn node_with_unknown_type_is_rejected() {
    let json = json!({
        "id": CONNECTION_ID.to_string(),
        "name": "web01",
        "created": "2026-11-10T10:00:00Z",
        "modified": "2026-11-10T10:05:00Z",
        "type": "telnet"
    });

    let result = serde_json::from_value::<Node>(json);

    assert!(result.is_err());
}

#[test]
fn node_without_type_is_rejected() {
    let json = json!({
        "id": CONNECTION_ID.to_string(),
        "name": "web01",
        "created": "2026-11-10T10:00:00Z",
        "modified": "2026-11-10T10:05:00Z",
        "protocol": "ssh",
        "host": "web01.example.com"
    });

    let result = serde_json::from_value::<Node>(json);

    assert!(result.is_err());
}

#[test]
fn node_with_every_optional_field_round_trips() {
    let mut node = fixed(Node::credential("deploy key"), CREDENTIAL_ID);
    node.description = Some("chiave di deploy".to_owned());
    node.notes = Some("ruotare ogni anno".to_owned());
    node.icon = Some(Icon::Custom {
        sha256: "ab".repeat(32),
        mime: "image/png".to_owned(),
    });
    node.color = Some("#336699".to_owned());
    node.favorite = true;
    node.tags = vec!["prod".to_owned()];
    node.custom_properties = vec![
        CustomProperty {
            name: "owner".to_owned(),
            value: PropertyValue::Text {
                value: "team-a".to_owned(),
            },
        },
        CustomProperty {
            name: "pin".to_owned(),
            value: PropertyValue::Secret {
                secret: SecretRef::new(),
            },
        },
    ];
    node.extra.insert("royal.Field".to_owned(), json!("value"));
    node.source = Some(SourceInfo {
        system: "royal".to_owned(),
        source_type: "RoyalCredential".to_owned(),
        source_id: Some("royal-id".to_owned()),
        file: Some("example.rtsz".to_owned()),
        imported_at: at("2026-11-10T09:00:00Z"),
    });
    let credential = node.as_credential_mut().unwrap();
    credential.username = Some("deploy".to_owned());
    credential.private_key = Some(PrivateKey::Vault {
        secret: SecretRef::new(),
    });
    credential.passphrase = Some(SecretRef::new());

    let text = serde_json::to_string(&node).unwrap();
    let back: Node = serde_json::from_str(&text).unwrap();

    assert_eq!(back, node);
}

#[test]
fn custom_property_value_is_flattened_next_to_the_name() {
    let property = CustomProperty {
        name: "owner".to_owned(),
        value: PropertyValue::Text {
            value: "team-a".to_owned(),
        },
    };

    let value = serde_json::to_value(&property).unwrap();

    assert_eq!(
        value,
        json!({ "name": "owner", "kind": "text", "value": "team-a" })
    );
}

#[test]
fn unsupported_node_keeps_the_raw_fragment() {
    let json = json!({
        "id": CONNECTION_ID.to_string(),
        "name": "vnc-legacy",
        "created": "2026-11-10T10:00:00Z",
        "modified": "2026-11-10T10:05:00Z",
        "type": "unsupported",
        "source_type": "RoyalVNCConnection",
        "raw": "<RoyalVNCConnection />"
    });

    let node: Node = serde_json::from_value(json.clone()).unwrap();

    assert_eq!(
        node.kind,
        NodeKind::Unsupported(Unsupported {
            source_type: "RoyalVNCConnection".to_owned(),
            raw: "<RoyalVNCConnection />".to_owned(),
        })
    );
    assert_eq!(serde_json::to_value(&node).unwrap(), json);
}

#[test]
fn folder_constructor_creates_an_empty_inheriting_folder() {
    let node = Node::folder("Clienti");

    assert!(node.is_folder());
    assert_eq!(node.name, "Clienti");
    let clienti = node.as_folder().unwrap();
    assert!(clienti.children.is_empty());
    assert_eq!(clienti.credential, CredentialSpec::Inherit);
    assert_eq!(node.created, node.modified);
}

#[test]
fn connection_constructor_uses_default_port_and_inherited_credential() {
    let node = Node::connection("web01", "ssh", "web01.example.com");

    assert!(!node.is_folder());
    let web01 = node.as_connection().unwrap();
    assert_eq!(web01.protocol, "ssh");
    assert_eq!(web01.host, "web01.example.com");
    assert_eq!(web01.port, None);
    assert_eq!(web01.credential, CredentialSpec::Inherit);
    assert!(web01.settings.is_empty());
}

#[test]
fn credential_constructor_creates_a_credential_without_fields() {
    let node = Node::credential("Admin ACME");

    let admin_acme = node.as_credential().unwrap();
    assert_eq!(admin_acme.username, None);
    assert_eq!(admin_acme.domain, None);
    assert_eq!(admin_acme.password, None);
    assert_eq!(admin_acme.private_key, None);
    assert_eq!(admin_acme.passphrase, None);
}

#[test]
fn constructors_assign_distinct_ids() {
    let first = Node::folder("Clienti");
    let second = Node::folder("Clienti");

    assert_ne!(first.id, second.id);
}

#[test]
fn as_helpers_return_none_for_other_kinds() {
    let mut folder = Node::folder("Clienti");

    assert!(folder.as_connection().is_none());
    assert!(folder.as_credential().is_none());
    assert!(folder.as_connection_mut().is_none());
    assert!(folder.as_credential_mut().is_none());
}

#[test]
fn touch_updates_modified_and_keeps_created() {
    let mut node = Node::folder("Clienti");
    node.created = at("2000-01-01T00:00:00Z");
    node.modified = at("2000-01-01T00:00:00Z");

    node.touch();

    assert!(node.modified > at("2000-01-01T00:00:00Z"));
    assert_eq!(node.created, at("2000-01-01T00:00:00Z"));
}

#[test]
fn new_document_has_current_format_and_schema() {
    let document = Document::new("Portolano");

    assert_eq!(document.format, FORMAT_ID);
    assert_eq!(document.schema_version, SCHEMA_VERSION);
    assert!(document.nodes.is_empty());
    assert_eq!(document.created, document.modified);
}
