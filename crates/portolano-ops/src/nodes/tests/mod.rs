use portolano_connectors::Registry;
use portolano_core::{CredentialSpec, Node, Vault};

pub(super) use super::*;
pub(super) use crate::testutil::*;

/// The smallest valid PNG header: enough for the icon store to recognize it.
pub(super) const PNG: &[u8] = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR";

pub(super) fn ssh(path: &str, credential: CredentialInput) -> NewNode {
    NewNode {
        path: path.to_owned(),
        create_parents: false,
        kind: NewNodeKind::Connection {
            protocol: "ssh".to_owned(),
            host: "web01.example.com".to_owned(),
            port: None,
            credential,
            settings: serde_json::Map::new(),
        },
        details: NodeDetails::default(),
        secrets: NodeSecrets::default(),
    }
}

pub(super) fn cred(path: &str) -> NewNode {
    NewNode {
        path: path.to_owned(),
        create_parents: false,
        kind: NewNodeKind::Credential {
            username: Some("admin".to_owned()),
            domain: None,
            key_file: None,
        },
        details: NodeDetails::default(),
        secrets: NodeSecrets::default(),
    }
}

pub(super) fn add(stored: &mut Stored, vault: &dyn Vault, node: NewNode) -> Result<AddOutcome> {
    add_node(
        &stored.store,
        &mut stored.doc,
        vault,
        &Registry::with_defaults(),
        node,
    )
}

pub(super) fn edit(
    stored: &mut Stored,
    vault: &dyn Vault,
    path: &str,
    edit: NodeEdit,
) -> Result<EditOutcome> {
    let id = id_of(&stored.doc, path);
    edit_node(
        &stored.store,
        &mut stored.doc,
        vault,
        &Registry::with_defaults(),
        id,
        edit,
    )
}

pub(super) fn node<'a>(stored: &'a Stored, path: &str) -> &'a Node {
    stored.doc.find(id_of(&stored.doc, path)).unwrap()
}

pub(super) fn spec_of(stored: &Stored, path: &str) -> CredentialSpec {
    node(stored, path)
        .as_connection()
        .unwrap()
        .credential
        .clone()
}

mod add;
mod edit;
