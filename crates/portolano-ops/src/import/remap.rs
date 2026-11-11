//! `--on-conflict new-ids`: new ids for every imported node, with the
//! references inside the import remapped.

use std::collections::HashMap;
use std::{mem, slice};

use portolano_core::Node;
use serde_json::Value;
use uuid::Uuid;

use crate::Result;
use crate::tree_util::for_each_node_mut;

/// Key of a reference object (SPEC §3).
const REF_KEY: &str = "$ref";

/// Gives every node of `root`'s subtree a new id and rewrites the
/// `{"$ref": …}` objects that point inside the subtree, wherever they are
/// (credential specs, protocol settings). References to nodes outside the
/// import and `source.source_id` are left as they are.
///
/// # Errors
/// [`portolano_core::Error::Serialize`] if a node cannot be converted to
/// JSON and back.
pub(super) fn regenerate_ids(root: &mut Node) -> Result<()> {
    let mut new_ids = HashMap::new();
    for_each_node_mut(slice::from_mut(root), &mut |node| {
        let new_id = Uuid::new_v4();
        new_ids.insert(node.id, new_id);
        node.id = new_id;
    });
    let mut result = Ok(());
    for_each_node_mut(slice::from_mut(root), &mut |node| {
        if result.is_ok() {
            result = remap_references(node, &new_ids);
        }
    });
    result
}

/// Rewrites the references of `node` itself; its children are set aside
/// meanwhile, so that each node is converted once.
fn remap_references(node: &mut Node, new_ids: &HashMap<Uuid, Uuid>) -> Result<()> {
    let children = node
        .as_folder_mut()
        .map(|folder| mem::take(&mut folder.children));
    let serialize = |source| portolano_core::Error::Serialize { source };
    let mut value = serde_json::to_value(&*node).map_err(serialize)?;
    replace_references(&mut value, new_ids);
    let mut remapped: Node = serde_json::from_value(value).map_err(serialize)?;
    if let (Some(children), Some(folder)) = (children, remapped.as_folder_mut()) {
        folder.children = children;
    }
    *node = remapped;
    Ok(())
}

fn replace_references(value: &mut Value, new_ids: &HashMap<Uuid, Uuid>) {
    match value {
        Value::Object(object) => {
            let target = object
                .get(REF_KEY)
                .and_then(Value::as_str)
                .and_then(|text| Uuid::parse_str(text).ok())
                .and_then(|old| new_ids.get(&old));
            if let Some(new_id) = target {
                object.insert(REF_KEY.to_owned(), Value::String(new_id.to_string()));
            }
            object
                .values_mut()
                .for_each(|inner| replace_references(inner, new_ids));
        }
        Value::Array(items) => items
            .iter_mut()
            .for_each(|inner| replace_references(inner, new_ids)),
        _ => {}
    }
}
