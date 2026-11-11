//! Credential specs held by a node, including those embedded in protocol
//! `settings` (DEC-014, DEC-016).
//!
//! A protocol may store a [`CredentialSpec`] inside its settings, such as the
//! credential of an RDP gateway. The core recognizes it by its own serialized
//! form, tagged by `"mode"`, without knowing the protocol (SPEC §1.5).

use serde::Deserialize;
use serde_json::Value;

use crate::model::{CredentialSpec, Node, NodeKind};

/// Tag key of a serialized [`CredentialSpec`].
const MODE_KEY: &str = "mode";

/// Calls `visit` on every JSON object inside `value`, `value` included, in
/// pre-order. Core formats embedded in protocol data (`$ref`, inline
/// credential specs) are found this way without knowing the protocols.
pub(crate) fn for_each_object(value: &Value, visit: &mut impl FnMut(&Value)) {
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
pub(crate) fn for_each_object_mut(value: &mut Value, visit: &mut impl FnMut(&mut Value)) {
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
pub(crate) fn embedded_spec(object: &Value) -> Option<CredentialSpec> {
    object.get(MODE_KEY)?;
    CredentialSpec::deserialize(object).ok()
}

/// The JSON form of [`CredentialSpec::Prompt`], to replace an embedded spec.
pub(crate) fn prompt_value() -> Value {
    #[expect(
        clippy::expect_used,
        reason = "a unit variant tagged by a string always serializes to a JSON object"
    )]
    serde_json::to_value(CredentialSpec::Prompt).expect("CredentialSpec::Prompt serializes to JSON")
}

/// The credential spec of a folder or a connection; other kinds have none.
pub(crate) fn credential_spec(node: &Node) -> Option<&CredentialSpec> {
    match &node.kind {
        NodeKind::Folder(folder) => Some(&folder.credential),
        NodeKind::Connection(connection) => Some(&connection.credential),
        NodeKind::Credential(_) | NodeKind::Unsupported(_) => None,
    }
}

/// The mutable credential spec of a folder or a connection.
pub(crate) fn credential_spec_mut(node: &mut Node) -> Option<&mut CredentialSpec> {
    match &mut node.kind {
        NodeKind::Folder(folder) => Some(&mut folder.credential),
        NodeKind::Connection(connection) => Some(&mut connection.credential),
        NodeKind::Credential(_) | NodeKind::Unsupported(_) => None,
    }
}
