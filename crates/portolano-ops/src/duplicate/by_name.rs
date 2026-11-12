//! Credentials by name in a copy (DEC-031.3): no reference of the document
//! may resolve differently after "Duplica", and the copy uses its own
//! credentials.
//!
//! - A name inside the copy that resolves to a credential of the copied
//!   subtree becomes a reference by id to the copy of that credential.
//! - A copied credential whose name some reference by name looks for gets a
//!   name no other node has, so that the name stays unambiguous.

use std::collections::{HashMap, HashSet};
use std::{mem, slice};

use portolano_core::{CredentialOrigin, CredentialSpec, Document, Node};
use serde_json::{Map, Value, json};
use uuid::Uuid;

use crate::Result;
use crate::tree_util::for_each_node_mut;

/// Key and value of the tag of a credential spec by name, wherever it is
/// (on the node or inside the settings of a protocol).
const MODE: &str = "mode";
const BY_NAME: &str = "by_name";

/// The ids of the nodes of `node`'s subtree, in tree order.
pub(super) fn ids_in_order(node: &Node) -> Vec<Uuid> {
    let mut ids = Vec::new();
    collect_ids(node, &mut ids);
    ids
}

fn collect_ids(node: &Node, ids: &mut Vec<Uuid>) {
    ids.push(node.id);
    if let Some(folder) = node.as_folder() {
        for child in &folder.children {
            collect_ids(child, ids);
        }
    }
}

/// Rewrites the specs by name of `copy` that, in `doc`, resolve to a
/// credential with a copy in `copies` (original id → copy id) into specs by
/// id of that copy.
///
/// # Errors
/// [`portolano_core::Error::Serialize`] if a node cannot be converted to
/// JSON and back.
pub(super) fn retarget(
    copy: &mut Node,
    doc: &Document,
    copies: &HashMap<Uuid, Uuid>,
) -> Result<()> {
    let mut result = Ok(());
    for_each_node_mut(slice::from_mut(copy), &mut |node| {
        if result.is_ok() {
            result = retarget_node(node, doc, copies);
        }
    });
    result
}

/// Rewrites the specs of `node` itself; its children are set aside meanwhile.
fn retarget_node(node: &mut Node, doc: &Document, copies: &HashMap<Uuid, Uuid>) -> Result<()> {
    let children = node
        .as_folder_mut()
        .map(|folder| mem::take(&mut folder.children));
    let serialize = |source| portolano_core::Error::Serialize { source };
    let mut value = serde_json::to_value(&*node).map_err(serialize)?;
    let changed = replace_by_name(&mut value, &|name| copy_of_named(doc, name, copies));
    if changed {
        *node = serde_json::from_value(value).map_err(serialize)?;
    }
    if let (Some(children), Some(folder)) = (children, node.as_folder_mut()) {
        folder.children = children;
    }
    Ok(())
}

/// The copy of the credential that `name` resolves to in `doc`, if that
/// credential was copied.
fn copy_of_named(doc: &Document, name: &str, copies: &HashMap<Uuid, Uuid>) -> Option<Uuid> {
    let first = doc.nodes.first()?.id;
    let spec = CredentialSpec::ByName {
        name: name.to_owned(),
    };
    match doc.resolve_spec(first, &spec).ok()?.origin {
        CredentialOrigin::Credential { node } => copies.get(&node).copied(),
        _ => None,
    }
}

/// Replaces every spec by name whose target `copy_of` gives with a spec by
/// id; returns whether anything changed.
fn replace_by_name(value: &mut Value, copy_of: &dyn Fn(&str) -> Option<Uuid>) -> bool {
    match value {
        Value::Object(object) => {
            if let Some(target) = by_name(object).and_then(copy_of) {
                // The form of `CredentialSpec::ById` in the document.
                *value = json!({ MODE: "by_id", "$ref": target });
                return true;
            }
            let mut changed = false;
            for inner in object.values_mut() {
                changed |= replace_by_name(inner, copy_of);
            }
            changed
        }
        Value::Array(items) => items.iter_mut().fold(false, |changed, item| {
            replace_by_name(item, copy_of) | changed
        }),
        _ => false,
    }
}

/// The name of a spec by name.
fn by_name(object: &Map<String, Value>) -> Option<&str> {
    (object.get(MODE)?.as_str()? == BY_NAME)
        .then(|| object.get("name")?.as_str())
        .flatten()
}

/// The names, lowercase, that references by name of `doc` look for.
pub(super) fn wanted_names(doc: &Document) -> HashSet<String> {
    let mut names = HashSet::new();
    for (_, node) in doc.walk() {
        let mut own = node.clone();
        if let Some(folder) = own.as_folder_mut() {
            folder.children.clear();
        }
        if let Ok(value) = serde_json::to_value(&own) {
            collect_names(&value, &mut names);
        }
    }
    names
}

fn collect_names(value: &Value, names: &mut HashSet<String>) {
    match value {
        Value::Object(object) => {
            if let Some(name) = by_name(object) {
                names.insert(name.to_lowercase());
            }
            object
                .values()
                .for_each(|inner| collect_names(inner, names));
        }
        Value::Array(items) => items.iter().for_each(|item| collect_names(item, names)),
        _ => {}
    }
}

/// Gives every credential of `copy` whose name is in `wanted` a name that
/// no node of `doc` or of the copy has, ignoring case: `<name> (copia)`,
/// then `(copia 2)`, … Returns the renamed credentials.
pub(super) fn rename_wanted(
    copy: &mut Node,
    doc: &Document,
    wanted: &HashSet<String>,
) -> Vec<Uuid> {
    let mut taken: HashSet<String> = doc
        .walk()
        .map(|(_, node)| node.name.to_lowercase())
        .collect();
    for_each_node_mut(slice::from_mut(copy), &mut |node| {
        taken.insert(node.name.to_lowercase());
    });
    let mut renamed = Vec::new();
    for_each_node_mut(slice::from_mut(copy), &mut |node| {
        if node.as_credential().is_none() || !wanted.contains(&node.name.to_lowercase()) {
            return;
        }
        let mut candidate = format!("{} (copia)", node.name);
        let mut counter = 2;
        while taken.contains(&candidate.to_lowercase()) {
            candidate = format!("{} (copia {counter})", node.name);
            counter += 1;
        }
        taken.insert(candidate.to_lowercase());
        node.name = candidate;
        renamed.push(node.id);
    });
    renamed
}
