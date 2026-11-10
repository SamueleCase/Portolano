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
    let data = connection.as_connection_mut().unwrap();
    data.port = Some(2222);
    data.settings.insert(
        "jump_hosts".to_owned(),
        json!([{ "$ref": JUMP_ID.to_string() }]),
    );
    data.settings
        .insert("agent_forwarding".to_owned(), json!(true));

    let mut credential = fixed(Node::credential("Admin ACME"), CREDENTIAL_ID);
    let data = credential.as_credential_mut().unwrap();
    data.username = Some("admin".to_owned());
    data.domain = Some("ACME".to_owned());
    data.password = Some(password.clone());

    let mut folder = fixed(Node::folder("Clienti"), FOLDER_ID);
    let data = folder.as_folder_mut().unwrap();
    data.credential = CredentialSpec::ById { id: CREDENTIAL_ID };
    data.children = vec![connection, credential];

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
fn credential_spec_modes_are_tagged_by_mode() {
    let password = SecretRef::new();
    let cases = [
        (CredentialSpec::None, json!({ "mode": "none" })),
        (CredentialSpec::Prompt, json!({ "mode": "prompt" })),
        (
            CredentialSpec::Inline {
                username: Some("admin".to_owned()),
                domain: None,
                password: Some(password.clone()),
            },
            json!({ "mode": "inline", "username": "admin", "password": password.as_str() }),
        ),
        (
            CredentialSpec::ByName {
                name: "Admin ACME".to_owned(),
            },
            json!({ "mode": "by_name", "name": "Admin ACME" }),
        ),
    ];

    for (spec, expected) in cases {
        let value = serde_json::to_value(&spec).unwrap();
        let back: CredentialSpec = serde_json::from_value(value.clone()).unwrap();

        assert_eq!(value, expected);
        assert_eq!(back, spec);
    }
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
fn constructors_create_the_requested_kind_with_defaults() {
    let folder = Node::folder("Clienti");
    let connection = Node::connection("web01", "ssh", "web01.example.com");
    let credential = Node::credential("Admin ACME");

    assert!(folder.is_folder());
    assert_eq!(
        folder.as_folder().unwrap().credential,
        CredentialSpec::Inherit
    );
    let data = connection.as_connection().unwrap();
    assert_eq!(
        (data.protocol.as_str(), data.host.as_str(), data.port),
        ("ssh", "web01.example.com", None)
    );
    assert!(!connection.is_folder());
    assert!(credential.as_credential().unwrap().username.is_none());
    assert_eq!(connection.created, connection.modified);
    assert_ne!(folder.id, connection.id);
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
