//! Tests of validation and references, sharing one sample document.

use serde_json::{Value, json};

use super::*;
use crate::testutil::{connection, credential, document, folder, with_spec};

const ADMIN: Uuid = Uuid::from_u128(0xad);
const BASTION: Uuid = Uuid::from_u128(0xba);
const CLIENTS: Uuid = Uuid::from_u128(0xc1);
const WEB: Uuid = Uuid::from_u128(0x3e);
const DB: Uuid = Uuid::from_u128(0xdb);
const OTHER: Uuid = Uuid::from_u128(0xe0);
const SECOND: Uuid = Uuid::from_u128(0xe1);
const MISSING: Uuid = Uuid::from_u128(0xf0);

fn by_id(id: Uuid) -> CredentialSpec {
    CredentialSpec::ById { id }
}

fn by_name(name: &str) -> CredentialSpec {
    CredentialSpec::ByName {
        name: name.to_owned(),
    }
}

fn with_jump(mut node: Node, jump: Value) -> Node {
    node.as_connection_mut()
        .unwrap()
        .settings
        .insert("jump_hosts".to_owned(), json!([jump]));
    node
}

/// A valid document: `Clients` uses `Admin ACME` by id, `web` jumps through
/// `bastion`, `db` inherits.
fn sample() -> Document {
    document(vec![
        credential(ADMIN, "Admin ACME"),
        connection(BASTION, "bastion"),
        with_spec(
            folder(
                CLIENTS,
                "Clients",
                vec![
                    with_jump(connection(WEB, "web"), json!({ "$ref": BASTION })),
                    connection(DB, "db"),
                ],
            ),
            by_id(ADMIN),
        ),
    ])
}

fn set_spec(document: &mut Document, id: Uuid, spec: CredentialSpec) {
    let node = document.find_mut(id).unwrap();
    *node = with_spec(node.clone(), spec);
}

fn with_setting(mut node: Node, key: &str, value: Value) -> Node {
    node.as_connection_mut()
        .unwrap()
        .settings
        .insert(key.to_owned(), value);
    node
}

mod references;
mod validation;
