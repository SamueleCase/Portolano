//! Small tree helpers the core does not expose.

use std::collections::HashSet;

use portolano_core::{Document, Node, SecretRef};
use uuid::Uuid;

/// The ids of `node` and of its descendants.
pub(crate) fn subtree_ids(node: &Node) -> HashSet<Uuid> {
    let mut ids = HashSet::from([node.id]);
    if let Some(folder) = node.as_folder() {
        for child in &folder.children {
            ids.extend(subtree_ids(child));
        }
    }
    ids
}

/// Every secret reference in the document.
pub(crate) fn document_secret_refs(doc: &Document) -> HashSet<SecretRef> {
    doc.nodes
        .iter()
        .flat_map(|node| doc.secret_refs_in(node.id))
        .collect()
}

/// The secret references held by `node` itself, without its descendants.
pub(crate) fn own_secret_refs(doc: &Document, node: &Node) -> Vec<SecretRef> {
    let in_children: HashSet<SecretRef> = node
        .as_folder()
        .map(|folder| {
            folder
                .children
                .iter()
                .flat_map(|child| doc.secret_refs_in(child.id))
                .collect()
        })
        .unwrap_or_default();
    let mut seen = HashSet::new();
    doc.secret_refs_in(node.id)
        .into_iter()
        .filter(|reference| !in_children.contains(reference) && seen.insert(reference.clone()))
        .collect()
}

/// The path of `id` for messages, `/` for the root.
pub(crate) fn location_path(doc: &Document, id: Option<Uuid>) -> String {
    id.and_then(|id| doc.path_of(id))
        .unwrap_or_else(|| "/".to_owned())
}

/// The path of an existing node for messages; its id if it is missing.
pub(crate) fn node_path(doc: &Document, id: Uuid) -> String {
    doc.path_of(id).unwrap_or_else(|| format!("@{id}"))
}
