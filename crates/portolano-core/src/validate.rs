//! Structural validation (SPEC §4.4, DEC-016) and, in [`references`], the
//! references between nodes.

mod references;

use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::error::{Issue, Severity};
use crate::model::{CredentialSpec, Document, Node, NodeKind};
use crate::resolve::CredentialNames;
use crate::secret::SecretRef;
use crate::tree::collect_secret_refs;

use crate::spec_scan::credential_spec;
use references::{reference_values, referenced_id};

impl Document {
    /// Structural problems of the document, with the codes `duplicate_id`,
    /// `dangling_ref`, `credential_ref_not_credential`, `by_name_unresolved`,
    /// `by_name_ambiguous`, `empty_host` and `duplicate_secret_ref`, all of severity
    /// [`Severity::Error`], plus `unsupported_node` of severity
    /// [`Severity::Info`] (DEC-016). Protocol and vault checks are added by the
    /// caller.
    pub fn validate(&self) -> Vec<Issue> {
        let mut issues = self.duplicate_ids();
        issues.extend(self.duplicate_secret_refs());
        let lookup = Lookup::new(self);
        for (_, node) in self.walk() {
            issues.extend(dangling_references(node, &lookup.nodes));
            if let Some(spec) = credential_spec(node) {
                issues.extend(lookup.credential_spec_issue(node, spec));
            }
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

    /// One issue per [`SecretRef`] held in more than one place, on its first
    /// holder: a secret belongs to a single place, so that deleting a node
    /// never deletes the secret of another one (DEC-016.4).
    fn duplicate_secret_refs(&self) -> Vec<Issue> {
        let mut holders: HashMap<SecretRef, Vec<&Node>> = HashMap::new();
        let mut first_seen = Vec::new();
        for (_, node) in self.walk() {
            let mut refs = Vec::new();
            collect_secret_refs(node, &mut refs);
            for secret in refs {
                let nodes = holders.entry(secret.clone()).or_default();
                if nodes.is_empty() {
                    first_seen.push(secret);
                }
                nodes.push(node);
            }
        }
        first_seen
            .into_iter()
            .filter_map(|secret| {
                let nodes = holders.get(&secret).filter(|nodes| nodes.len() > 1)?;
                let mut paths: Vec<String> = nodes
                    .iter()
                    .filter_map(|node| self.path_of(node.id))
                    .collect();
                paths.dedup();
                Some(error(
                    nodes.first()?,
                    "duplicate_secret_ref",
                    format!(
                        "il segreto {} è usato in {} punti: {}",
                        secret.as_str(),
                        nodes.len(),
                        paths.join(", ")
                    ),
                ))
            })
            .collect()
    }
}

/// Lookups built once per validation, so that each check is a map access
/// and validation stays linear in the document size.
struct Lookup<'a> {
    nodes: HashMap<Uuid, &'a Node>,
    credentials: CredentialNames<'a>,
}

impl<'a> Lookup<'a> {
    fn new(document: &'a Document) -> Self {
        let mut nodes = HashMap::new();
        for (_, node) in document.walk() {
            // With duplicate ids the first node wins, as in `Document::find`.
            nodes.entry(node.id).or_insert(node);
        }
        Self {
            nodes,
            credentials: CredentialNames::new(document),
        }
    }

    /// Checks the node's own credential spec. `$ref`s inside `settings` that
    /// point to credentials are checked by their protocol: the core cannot
    /// tell them apart from references to other kinds of node.
    fn credential_spec_issue(&self, node: &Node, spec: &CredentialSpec) -> Option<Issue> {
        match spec {
            CredentialSpec::ById { id } => {
                let target = self.nodes.get(id)?;
                target.as_credential().is_none().then(|| {
                    error(
                        node,
                        "credential_ref_not_credential",
                        format!(
                            "una credenziale del nodo '{}' fa riferimento a '{}', che non è una credenziale",
                            node.name, target.name
                        ),
                    )
                })
            }
            CredentialSpec::ByName { name } => match self.credentials.matching(name).len() {
                1 => None,
                0 => Some(error(
                    node,
                    "by_name_unresolved",
                    format!(
                        "nessuna credenziale si chiama '{name}' (richiesta dal nodo '{}')",
                        node.name
                    ),
                )),
                count => Some(error(
                    node,
                    "by_name_ambiguous",
                    format!(
                        "la credenziale '{name}' del nodo '{}' corrisponde a {count} credenziali",
                        node.name
                    ),
                )),
            },
            _ => None,
        }
    }
}

/// One `dangling_ref` issue per `$ref` of `node` whose value is not the id of
/// a node in `nodes`, including values that are not ids at all.
fn dangling_references(node: &Node, nodes: &HashMap<Uuid, &Node>) -> Vec<Issue> {
    reference_values(node)
        .into_iter()
        .filter(|value| referenced_id(value).is_none_or(|id| !nodes.contains_key(&id)))
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
