//! References between nodes and structural validation (SPEC §4.4, DEC-014).
//!
//! Every reference from a node to another one is an object
//! `{"$ref": "<uuid>"}` wherever it appears, protocol `settings` included, so
//! references are found by serializing a node without its children and
//! scanning the JSON, without knowing the protocols.

use std::collections::{HashMap, HashSet};

use serde_json::Value;
use uuid::Uuid;

use crate::error::{Error, Issue, Severity};
use crate::model::{CredentialSpec, Document, Folder, Node, NodeKind};

/// Key of a reference object (SPEC §3).
const REF_KEY: &str = "$ref";

impl Document {
    /// The nodes that refer to `target`: those containing `{"$ref": target}`
    /// anywhere (children excluded) and, if `target` is a credential, those
    /// with a `ByName` spec that resolves to it, case-insensitive fallback
    /// included (DEC-014).
    pub fn references_to(&self, target: Uuid) -> Vec<Uuid> {
        self.walk()
            .map(|(_, node)| node)
            .filter(|node| {
                reference_values(node)
                    .iter()
                    .any(|value| referenced_id(value) == Some(target))
                    || self.by_name_resolves_to(node, target)
            })
            .map(|node| node.id)
            .collect()
    }

    /// Structural problems of the document, with the codes `duplicate_id`,
    /// `dangling_ref`, `credential_ref_not_credential`, `by_name_unresolved`,
    /// `by_name_ambiguous` and `empty_host`, all of severity
    /// [`Severity::Error`], plus `unsupported_node` of severity
    /// [`Severity::Info`] (DEC-016). Protocol and vault checks are added by the
    /// caller.
    pub fn validate(&self) -> Vec<Issue> {
        let mut issues = self.duplicate_ids();
        let known: HashSet<Uuid> = self.walk().map(|(_, node)| node.id).collect();
        for (_, node) in self.walk() {
            issues.extend(dangling_references(node, &known));
            issues.extend(self.credential_spec_issue(node));
            if let Some(connection) = node.as_connection()
                && connection.host.trim().is_empty()
            {
                issues.push(error(
                    node,
                    "empty_host",
                    format!("la connessione '{}' non ha un host", node.name),
                ));
            }
            if let NodeKind::Unsupported(unsupported) = &node.kind {
                issues.push(issue(
                    Severity::Info,
                    node,
                    "unsupported_node",
                    format!(
                        "il nodo '{}' non è supportato ed è conservato così com'è (tipo di origine: {})",
                        node.name, unsupported.source_type
                    ),
                ));
            }
        }
        issues
    }

    /// One issue per id used by more than one node, on its first occurrence.
    fn duplicate_ids(&self) -> Vec<Issue> {
        let mut counts: HashMap<Uuid, usize> = HashMap::new();
        for (_, node) in self.walk() {
            *counts.entry(node.id).or_default() += 1;
        }
        let mut reported = HashSet::new();
        self.walk()
            .filter_map(|(_, node)| {
                let count = counts.get(&node.id).copied().unwrap_or_default();
                (count > 1 && reported.insert(node.id)).then(|| {
                    error(
                        node,
                        "duplicate_id",
                        format!("l'id {} è usato da {count} nodi", node.id),
                    )
                })
            })
            .collect()
    }

    /// Checks the node's own credential spec. `$ref`s inside `settings` that
    /// point to credentials are checked by their protocol: the core cannot
    /// tell them apart from references to other kinds of node.
    fn credential_spec_issue(&self, node: &Node) -> Option<Issue> {
        match credential_spec(node)? {
            CredentialSpec::ById { id } => {
                let target = self.find(*id)?;
                target.as_credential().is_none().then(|| {
                    error(
                        node,
                        "credential_ref_not_credential",
                        format!(
                            "la credenziale del nodo '{}' fa riferimento a '{}', che non è una credenziale",
                            node.name, target.name
                        ),
                    )
                })
            }
            CredentialSpec::ByName { name } => match self.credential_named(name) {
                Ok(_) => None,
                Err(Error::Ambiguous { candidates, .. }) => Some(error(
                    node,
                    "by_name_ambiguous",
                    format!(
                        "la credenziale '{name}' del nodo '{}' corrisponde a {} credenziali",
                        node.name,
                        candidates.len()
                    ),
                )),
                Err(_) => Some(error(
                    node,
                    "by_name_unresolved",
                    format!(
                        "nessuna credenziale si chiama '{name}' (richiesta dal nodo '{}')",
                        node.name
                    ),
                )),
            },
            _ => None,
        }
    }

    fn by_name_resolves_to(&self, node: &Node, target: Uuid) -> bool {
        match credential_spec(node) {
            Some(CredentialSpec::ByName { name }) => self
                .credential_named(name)
                .is_ok_and(|(credential, _)| credential == target),
            _ => false,
        }
    }
}

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

/// One `dangling_ref` issue per `$ref` of `node` whose value is not the id of
/// a node in `known`, including values that are not ids at all.
fn dangling_references(node: &Node, known: &HashSet<Uuid>) -> Vec<Issue> {
    reference_values(node)
        .into_iter()
        .filter(|value| referenced_id(value).is_none_or(|id| !known.contains(&id)))
        .map(|value| {
            error(
                node,
                "dangling_ref",
                format!(
                    "il nodo '{}' fa riferimento a {value}, che non esiste",
                    node.name
                ),
            )
        })
        .collect()
}

/// The values of every `$ref` key in the node, children excluded.
fn reference_values(node: &Node) -> Vec<Value> {
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

fn referenced_id(value: &Value) -> Option<Uuid> {
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
fn credential_spec(node: &Node) -> Option<&CredentialSpec> {
    match &node.kind {
        NodeKind::Folder(folder) => Some(&folder.credential),
        NodeKind::Connection(connection) => Some(&connection.credential),
        NodeKind::Credential(_) | NodeKind::Unsupported(_) => None,
    }
}

fn error(node: &Node, code: &'static str, message: String) -> Issue {
    issue(Severity::Error, node, code, message)
}

fn issue(severity: Severity, node: &Node, code: &'static str, message: String) -> Issue {
    Issue {
        severity,
        node: Some(node.id),
        code,
        message,
    }
}

#[cfg(test)]
mod tests;
