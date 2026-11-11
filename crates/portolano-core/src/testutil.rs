//! Builders shared by the unit tests of the crate (SPEC §11.3).
//!
//! Nodes get fixed ids so that assertions and `@id` paths are stable. Hosts
//! are names under `example.com` (SPEC §1.3).

use uuid::Uuid;

use crate::model::{CredentialSpec, Document, Node};

/// A folder with a fixed id and the given children.
pub(crate) fn folder(id: Uuid, name: &str, children: Vec<Node>) -> Node {
    let mut node = Node::folder(name);
    node.id = id;
    if let Some(folder) = node.as_folder_mut() {
        folder.children = children;
    }
    node
}

/// An SSH connection with a fixed id, to `<name>.example.com`.
pub(crate) fn connection(id: Uuid, name: &str) -> Node {
    let mut node = Node::connection(name, "ssh", format!("{name}.example.com"));
    node.id = id;
    node
}

/// A credential with a fixed id and no fields set.
pub(crate) fn credential(id: Uuid, name: &str) -> Node {
    let mut node = Node::credential(name);
    node.id = id;
    node
}

/// Sets the credential spec of a folder or a connection.
pub(crate) fn with_spec(mut node: Node, spec: CredentialSpec) -> Node {
    if let Some(folder) = node.as_folder_mut() {
        folder.credential = spec;
    } else if let Some(connection) = node.as_connection_mut() {
        connection.credential = spec;
    } else {
        panic!("only folders and connections have a credential spec");
    }
    node
}

/// A document whose root holds `nodes`.
pub(crate) fn document(nodes: Vec<Node>) -> Document {
    let mut document = Document::new("Test");
    document.nodes = nodes;
    document
}

/// The ids of `nodes`, in order.
pub(crate) fn ids(nodes: &[Node]) -> Vec<Uuid> {
    nodes.iter().map(|node| node.id).collect()
}
