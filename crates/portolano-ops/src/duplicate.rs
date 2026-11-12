//! "Duplica" (SPEC-UI §3.6, DEC-016.4): a copy of a node and its subtree
//! next to the original, with new ids and new secret references, so that no
//! secret is shared between the two (deleting one must never delete the
//! secret of the other), and without changing what any reference by name
//! resolves to (DEC-031.3).

mod by_name;

use std::collections::HashMap;

use chrono::Utc;
use portolano_core::{Document, Node, SecretRef, Store, Vault};
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

use self::by_name::{ids_in_order, rename_credentials, retarget, wanted_names};
use crate::Result;
use crate::commit::Change;
use crate::import::remap::regenerate_ids;
use crate::inspect::NodeRef;
use crate::tree_util::{for_each_node_mut, node_path};

/// The result of [`duplicate_node`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct DuplicateOutcome {
    /// Id of the copy.
    pub id: Uuid,
    /// Path of the copy.
    pub path: String,
    /// The credentials of the copy, renamed so that their name is no other
    /// credential's and no reference by name looks for it (DEC-031.3).
    pub renamed: Vec<NodeRef>,
    /// Secrets copied in the vault under their new references.
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub secrets_copied: usize,
    /// Secrets of the original that the vault does not have: the copy
    /// references them under new references, absent like the originals.
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub secrets_missing: usize,
}

/// Copies node `id` with its subtree right after it, named
/// `<name> (copia)` (or `(copia 2)`, … if taken), and saves the document.
///
/// Every node of the copy gets a new id and the references inside the copy
/// point to the copy, those by name included; references to other nodes
/// stay. The copy keeps no import origin. Every secret
/// reference gets a new one, and the vault receives a copy of each value
/// before the document is saved.
///
/// # Errors
/// [`portolano_core::Error::NodeNotFound`],
/// [`portolano_core::Error::TooDeep`], errors of the vault (reading the
/// secrets to copy needs it open) and of the store. On error the document
/// and the vault are unchanged.
pub fn duplicate_node(
    store: &Store,
    doc: &mut Document,
    vault: &dyn Vault,
    id: Uuid,
) -> Result<DuplicateOutcome> {
    let original = doc
        .find(id)
        .ok_or(portolano_core::Error::NodeNotFound { id })?;
    let parent = doc
        .parent_of(id)
        .ok_or(portolano_core::Error::NodeNotFound { id })?;
    let position = doc.children(parent)?.iter().position(|n| n.id == id);
    let mut copy = original.clone();
    regenerate_ids(&mut copy)?;
    let copies: HashMap<Uuid, Uuid> = ids_in_order(original)
        .into_iter()
        .zip(ids_in_order(&copy))
        .collect();
    retarget(&mut copy, doc, &copies)?;
    let renamed = rename_credentials(&mut copy, doc, &wanted_names(doc));
    if !renamed.contains(&copy.id) {
        copy.name = copy_name(doc, parent, &original.name)?;
    }
    let now = Utc::now();
    for_each_node_mut(std::slice::from_mut(&mut copy), &mut |node| {
        node.created = now;
        node.modified = now;
        // A copy made here was not imported.
        node.source = None;
    });
    let mut change = Change::from(doc);
    let mut outcome = DuplicateOutcome {
        id: copy.id,
        path: String::new(),
        renamed: Vec::new(),
        secrets_copied: 0,
        secrets_missing: 0,
    };
    let new_refs = new_references(doc.secret_refs_in(id));
    for (old, new) in &new_refs {
        match vault.get(old)? {
            Some(secret) => {
                change.new_secrets.push((new.clone(), secret));
                outcome.secrets_copied += 1;
            }
            None => outcome.secrets_missing += 1,
        }
    }
    let copy = with_references(copy, &new_refs)?;
    change.doc.insert(parent, position.map(|p| p + 1), copy)?;
    change.commit(store, vault, doc, |_| Ok(()))?;
    outcome.path = node_path(doc, outcome.id);
    outcome.renamed = renamed
        .into_iter()
        .map(|node| NodeRef::of(doc, node))
        .collect();
    Ok(outcome)
}

/// A new reference for each distinct reference of `refs`.
fn new_references(refs: Vec<SecretRef>) -> HashMap<SecretRef, SecretRef> {
    refs.into_iter()
        .map(|old| (old, SecretRef::new()))
        .collect()
}

/// The first of `<name> (copia)`, `<name> (copia 2)`, … that no child of
/// `parent` uses (DEC-022.5).
fn copy_name(doc: &Document, parent: Option<Uuid>, name: &str) -> Result<String> {
    let siblings = doc.children(parent)?;
    let taken = |candidate: &str| siblings.iter().any(|sibling| sibling.name == candidate);
    let mut candidate = format!("{name} (copia)");
    let mut counter = 2;
    while taken(&candidate) {
        candidate = format!("{name} (copia {counter})");
        counter += 1;
    }
    Ok(candidate)
}

/// `node` with every secret reference replaced as `new_refs` says.
///
/// References are strings wherever the model puts them (credentials, inline
/// specs, secret properties, inline specs inside protocol settings, see
/// `Document::secret_refs_in`): every string equal to one of them is
/// replaced. A reference is a random UUID, so no other value of the node can
/// be equal to it by chance.
fn with_references(node: Node, new_refs: &HashMap<SecretRef, SecretRef>) -> Result<Node> {
    if new_refs.is_empty() {
        return Ok(node);
    }
    let by_text: HashMap<&str, &str> = new_refs
        .iter()
        .map(|(old, new)| (old.as_str(), new.as_str()))
        .collect();
    let serialize = |source| portolano_core::Error::Serialize { source };
    let mut value = serde_json::to_value(node).map_err(serialize)?;
    replace_strings(&mut value, &by_text);
    Ok(serde_json::from_value(value).map_err(serialize)?)
}

fn replace_strings(value: &mut Value, by_text: &HashMap<&str, &str>) {
    match value {
        Value::String(text) => {
            if let Some(new) = by_text.get(text.as_str()) {
                *text = (*new).to_owned();
            }
        }
        Value::Array(items) => items
            .iter_mut()
            .for_each(|item| replace_strings(item, by_text)),
        Value::Object(object) => object
            .values_mut()
            .for_each(|inner| replace_strings(inner, by_text)),
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

#[cfg(test)]
mod tests;
