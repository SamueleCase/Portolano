//! References between nodes (SPEC §4.4, DEC-014, DEC-016.5).
//!
//! Every reference from a node to another one is an object
//! `{"$ref": "<uuid>"}` wherever it appears, protocol `settings` included, so
//! references are found by serializing a node without its children and
//! scanning the JSON, without knowing the protocols. Credential specs embedded
//! in `settings` (such as an RDP gateway's) are recognized by the serialized
//! form of [`CredentialSpec`], tagged by `"mode"`.

use std::collections::HashSet;

use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use crate::model::{CredentialSpec, Document, Folder, Node, NodeKind};
use crate::tree::for_each_node_mut;

/// Key of a reference object (SPEC §3).
const REF_KEY: &str = "$ref";

/// Tag key of a serialized [`CredentialSpec`].
const MODE_KEY: &str = "mode";

impl Document {
    /// The nodes that refer to `target`: those containing `{"$ref": target}`
    /// anywhere (children excluded) and, if `target` is a credential, those
    /// with a `ByName` spec, on the node or in its `settings`, that resolves
    /// to it, case-insensitive fallback included (DEC-014).
    pub fn references_to(&self, target: Uuid) -> Vec<Uuid> {
        self.walk()
            .map(|(_, node)| node)
            .filter(|node| {
                reference_values(node)
                    .iter()
                    .any(|value| referenced_id(value) == Some(target))
                    || credential_specs_of(node).iter().any(|spec| {
                        matches!(spec, CredentialSpec::ByName { name } if self.name_resolves_to(name, target))
                    })
            })
            .map(|node| node.id)
            .collect()
    }

    /// Turns every credential spec that refers to `credential` into
    /// [`CredentialSpec::Prompt`], on the node itself and inside `settings`.
    /// A spec refers to it by id (`ById`) or by a name that resolves to it as
    /// in SPEC §4.3, the rule of [`Document::references_to`].
    ///
    /// Used by `delete --force` on a referenced credential (DEC-016.5): the
    /// connections stay usable and ask for the login data at connection time.
    /// Updates `modified` of the nodes changed and returns their ids, in tree
    /// order; empty if nothing refers to the credential.
    pub fn detach_credential_references(&mut self, credential: Uuid) -> Vec<Uuid> {
        // Names are resolved once, before any change, against the whole
        // document: detaching one spec must not alter how another resolves.
        let names: HashSet<String> = self
            .walk()
            .flat_map(|(_, node)| credential_specs_of(node))
            .filter_map(|spec| match spec {
                CredentialSpec::ByName { name } => Some(name),
                _ => None,
            })
            .filter(|name| self.name_resolves_to(name, credential))
            .collect();
        let refers = |spec: &CredentialSpec| match spec {
            CredentialSpec::ById { id } => *id == credential,
            CredentialSpec::ByName { name } => names.contains(name),
            _ => false,
        };
        let mut detached = Vec::new();
        for_each_node_mut(&mut self.nodes, &mut |node| {
            if detach_specs(node, &refers) {
                node.touch();
                detached.push(node.id);
            }
        });
        detached
    }

    fn name_resolves_to(&self, name: &str, target: Uuid) -> bool {
        self.credential_named(name)
            .is_ok_and(|(credential, _)| credential == target)
    }
}

/// Calls `visit` on every JSON object inside `value`, `value` included, in
/// pre-order. Core formats embedded in protocol data (`$ref`, inline
/// credential specs) are found this way without knowing the protocols.
fn for_each_object(value: &Value, visit: &mut impl FnMut(&Value)) {
    match value {
        Value::Object(object) => {
            visit(value);
            for child in object.values() {
                for_each_object(child, visit);
            }
        }
        Value::Array(items) => {
            for item in items {
                for_each_object(item, visit);
            }
        }
        _ => {}
    }
}

/// Calls `visit` on every JSON object inside `value`, `value` included, in
/// pre-order; `visit` may replace the object.
fn for_each_object_mut(value: &mut Value, visit: &mut impl FnMut(&mut Value)) {
    if value.is_object() {
        visit(value);
    }
    match value {
        Value::Object(object) => {
            for child in object.values_mut() {
                for_each_object_mut(child, visit);
            }
        }
        Value::Array(items) => {
            for item in items {
                for_each_object_mut(item, visit);
            }
        }
        _ => {}
    }
}

/// The credential specs held by `node` itself: its own `credential` and the
/// objects in its `settings` that have the form of a [`CredentialSpec`].
pub(crate) fn credential_specs_of(node: &Node) -> Vec<CredentialSpec> {
    let mut specs: Vec<CredentialSpec> = credential_spec(node).into_iter().cloned().collect();
    if let Some(connection) = node.as_connection() {
        for value in connection.settings.values() {
            for_each_object(value, &mut |object| specs.extend(embedded_spec(object)));
        }
    }
    specs
}

/// The [`CredentialSpec`] a JSON object represents, if it has that form. An
/// object without the tag, or with a tag that is not a credential mode (such
/// as an RDP resolution `{"mode": "fit"}`), is protocol data.
fn embedded_spec(object: &Value) -> Option<CredentialSpec> {
    object.get(MODE_KEY)?;
    CredentialSpec::deserialize(object).ok()
}

/// Replaces with [`CredentialSpec::Prompt`] the specs of `node` for which
/// `refers` holds, own and embedded; returns whether anything changed.
fn detach_specs(node: &mut Node, refers: &impl Fn(&CredentialSpec) -> bool) -> bool {
    let mut changed = false;
    if let Some(spec) = credential_spec_mut(node)
        && refers(spec)
    {
        *spec = CredentialSpec::Prompt;
        changed = true;
    }
    if let Some(connection) = node.as_connection_mut() {
        for value in connection.settings.values_mut() {
            for_each_object_mut(value, &mut |object| {
                if embedded_spec(object).is_some_and(|spec| refers(&spec)) {
                    *object = prompt_value();
                    changed = true;
                }
            });
        }
    }
    changed
}

fn prompt_value() -> Value {
    #[expect(
        clippy::expect_used,
        reason = "a unit variant tagged by a string always serializes to a JSON object"
    )]
    serde_json::to_value(CredentialSpec::Prompt).expect("CredentialSpec::Prompt serializes to JSON")
}

/// The values of every `$ref` key in the node, children excluded.
pub(super) fn reference_values(node: &Node) -> Vec<Value> {
    #[expect(
        clippy::expect_used,
        reason = "a node always serializes: every map key is a string and every field type has an infallible Serialize"
    )]
    let serialized =
        serde_json::to_value(without_children(node)).expect("a node always serializes to JSON");
    let mut values = Vec::new();
    for_each_object(&serialized, &mut |object| {
        if let Some(value) = object.get(REF_KEY) {
            values.push(value.clone());
        }
    });
    values
}

pub(super) fn referenced_id(value: &Value) -> Option<Uuid> {
    value.as_str().and_then(|text| Uuid::parse_str(text).ok())
}

/// A copy of `node` without its children, so that a folder is not credited
/// with the references of its descendants and they are not cloned.
fn without_children(node: &Node) -> Node {
    let kind = match &node.kind {
        NodeKind::Folder(folder) => NodeKind::Folder(Folder {
            children: Vec::new(),
            credential: folder.credential.clone(),
        }),
        other => other.clone(),
    };
    Node {
        id: node.id,
        name: node.name.clone(),
        description: node.description.clone(),
        notes: node.notes.clone(),
        icon: node.icon.clone(),
        color: node.color.clone(),
        favorite: node.favorite,
        tags: node.tags.clone(),
        custom_properties: node.custom_properties.clone(),
        created: node.created,
        modified: node.modified,
        extra: node.extra.clone(),
        source: node.source.clone(),
        kind,
    }
}

/// The credential spec of a folder or a connection; other kinds have none.
pub(super) fn credential_spec(node: &Node) -> Option<&CredentialSpec> {
    match &node.kind {
        NodeKind::Folder(folder) => Some(&folder.credential),
        NodeKind::Connection(connection) => Some(&connection.credential),
        NodeKind::Credential(_) | NodeKind::Unsupported(_) => None,
    }
}

fn credential_spec_mut(node: &mut Node) -> Option<&mut CredentialSpec> {
    match &mut node.kind {
        NodeKind::Folder(folder) => Some(&mut folder.credential),
        NodeKind::Connection(connection) => Some(&mut connection.credential),
        NodeKind::Credential(_) | NodeKind::Unsupported(_) => None,
    }
}
