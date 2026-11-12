//! Where the effective credential of a node comes from, step by step
//! (SPEC-UI §3.6 `resolve_trace`): the node, the folders it inherits from,
//! and the end of the chain. The detail of the app shows it, and so does
//! the credential preview of the editor, including the chains that lead
//! nowhere (a name that matches no credential, or several).

use portolano_core::{CredentialOrigin, CredentialSpec, Document, NodeKind};
use serde::Serialize;
use uuid::Uuid;

use crate::Result;
use crate::inspect::NodeRef;
use crate::nodes::credential_spec;
use crate::tree_util::node_path;

/// A credential spec as the trace shows it, without secrets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum SpecView {
    /// From the parent folder.
    Inherit,
    /// No credential.
    None,
    /// Asked at connection time.
    Prompt,
    /// Login data on the node itself.
    Inline {
        /// User name.
        username: Option<String>,
        /// Windows domain.
        domain: Option<String>,
    },
    /// A credential node by id.
    ById {
        /// The referenced node.
        target: Uuid,
    },
    /// A credential node by name, looked up in the whole document.
    ByName {
        /// The name.
        name: String,
    },
}

impl SpecView {
    /// The view of `spec`.
    pub fn of(spec: &CredentialSpec) -> SpecView {
        match spec {
            CredentialSpec::Inherit => SpecView::Inherit,
            CredentialSpec::None => SpecView::None,
            CredentialSpec::Prompt => SpecView::Prompt,
            CredentialSpec::Inline {
                username, domain, ..
            } => SpecView::Inline {
                username: username.clone(),
                domain: domain.clone(),
            },
            CredentialSpec::ById { id } => SpecView::ById { target: *id },
            CredentialSpec::ByName { name } => SpecView::ByName { name: name.clone() },
        }
    }
}

/// One node of the chain and its own spec.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct TraceStep {
    /// The node.
    pub node: Uuid,
    /// Its path.
    pub path: String,
    /// Its spec.
    pub spec: SpecView,
}

/// Where the chain ends.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TraceEnd {
    /// The inheritance reached the root: no credential.
    Root,
    /// A spec says "no credential".
    None,
    /// Asked at connection time.
    Prompt,
    /// Inline data of a node.
    Inline {
        /// The node holding the data.
        node: NodeRef,
    },
    /// A credential node.
    Credential {
        /// The credential node.
        node: NodeRef,
    },
    /// No credential has the name.
    NameNotFound {
        /// The name.
        name: String,
    },
    /// Several credentials have the name.
    NameAmbiguous {
        /// The name.
        name: String,
        /// The credentials it matches.
        candidates: Vec<NodeRef>,
    },
    /// The referenced node does not exist.
    Dangling {
        /// The missing node.
        target: Uuid,
    },
    /// The referenced node is not a credential.
    NotACredential {
        /// The referenced node.
        node: NodeRef,
    },
}

/// The chain from a node to its effective credential.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct CredentialTrace {
    /// The node first, then each folder reached by inheriting; the last step
    /// holds the spec that ends the chain, unless it reached the root. Empty
    /// for a credential node, which is its own credential.
    pub steps: Vec<TraceStep>,
    /// Where it ends.
    pub end: TraceEnd,
}

/// The chain that gives node `id` its effective credential (SPEC §4.3). A
/// chain that leads nowhere is a result, not an error.
///
/// # Errors
/// [`portolano_core::Error::NodeNotFound`],
/// [`portolano_core::Error::NoCredential`] for an unsupported node.
pub fn resolve_trace(doc: &Document, id: Uuid) -> Result<CredentialTrace> {
    let node = doc
        .find(id)
        .ok_or(portolano_core::Error::NodeNotFound { id })?;
    let (mut context, mut spec) = match (&node.kind, credential_spec(node)) {
        (NodeKind::Credential(_), _) => {
            return Ok(CredentialTrace {
                steps: Vec::new(),
                end: TraceEnd::Credential {
                    node: NodeRef::of(doc, id),
                },
            });
        }
        (_, Some(spec)) => (id, spec),
        (_, None) => return Err(portolano_core::Error::NoCredential { id }.into()),
    };
    let mut steps = Vec::new();
    loop {
        steps.push(TraceStep {
            node: context,
            path: node_path(doc, context),
            spec: SpecView::of(spec),
        });
        if !spec.is_inherit() {
            let end = end_of(doc, context, spec)?;
            return Ok(CredentialTrace { steps, end });
        }
        let parent = doc
            .parent_of(context)
            .ok_or(portolano_core::Error::NodeNotFound { id: context })?;
        let Some(parent) = parent else {
            return Ok(CredentialTrace {
                steps,
                end: TraceEnd::Root,
            });
        };
        // Only folders have children; the error covers a broken invariant.
        spec = doc
            .find(parent)
            .and_then(|folder| folder.as_folder())
            .map(|folder| &folder.credential)
            .ok_or(portolano_core::Error::NotAFolder { id: parent })?;
        context = parent;
    }
}

/// The end of a chain whose last spec, owned by `context`, is not
/// `Inherit`.
fn end_of(doc: &Document, context: Uuid, spec: &CredentialSpec) -> Result<TraceEnd> {
    use portolano_core::Error;
    Ok(match doc.resolve_spec(context, spec) {
        Ok(resolved) => match resolved.origin {
            CredentialOrigin::None => TraceEnd::None,
            CredentialOrigin::Prompt => TraceEnd::Prompt,
            CredentialOrigin::Inline { node } => TraceEnd::Inline {
                node: NodeRef::of(doc, node),
            },
            CredentialOrigin::Credential { node } => TraceEnd::Credential {
                node: NodeRef::of(doc, node),
            },
        },
        Err(Error::NotFound { query }) => TraceEnd::NameNotFound { name: query },
        Err(Error::Ambiguous { query, candidates }) => TraceEnd::NameAmbiguous {
            name: query,
            candidates: candidates
                .into_iter()
                .map(|(id, path)| NodeRef { id, path })
                .collect(),
        },
        Err(Error::DanglingReference { target, .. }) => TraceEnd::Dangling { target },
        Err(Error::NotACredential { target, .. }) => TraceEnd::NotACredential {
            node: NodeRef::of(doc, target),
        },
        Err(error) => return Err(error.into()),
    })
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::OpsError;
    use crate::testutil::*;

    fn by_id(target: Uuid) -> CredentialSpec {
        CredentialSpec::ById { id: target }
    }

    fn folder_with(
        name: &str,
        spec: CredentialSpec,
        children: Vec<portolano_core::Node>,
    ) -> portolano_core::Node {
        let mut node = folder(name, children);
        if let Some(folder) = node.as_folder_mut() {
            folder.credential = spec;
        }
        node
    }

    fn by_name(name: &str) -> CredentialSpec {
        CredentialSpec::ByName {
            name: name.to_owned(),
        }
    }

    #[test]
    fn inherited_credential_lists_the_node_and_the_folders_up_to_the_spec() {
        let admin = credential("Admin", "admin", None);
        let admin_id = admin.id;
        let doc = document(vec![
            admin,
            folder_with(
                "Clienti",
                by_id(admin_id),
                vec![folder("ACME", vec![connection("web01", "ssh")])],
            ),
        ]);

        let trace = resolve_trace(&doc, id_of(&doc, "/Clienti/ACME/web01")).unwrap();

        let paths: Vec<&str> = trace.steps.iter().map(|step| step.path.as_str()).collect();
        assert_eq!(
            paths,
            vec!["/Clienti/ACME/web01", "/Clienti/ACME", "/Clienti"]
        );
        assert_eq!(trace.steps[2].spec, SpecView::ById { target: admin_id });
        assert_eq!(
            trace.end,
            TraceEnd::Credential {
                node: NodeRef {
                    id: admin_id,
                    path: "/Admin".to_owned()
                }
            }
        );
    }

    #[test]
    fn inheritance_up_to_the_root_ends_without_a_credential() {
        let doc = document(vec![folder("Clienti", vec![connection("web01", "ssh")])]);

        let trace = resolve_trace(&doc, id_of(&doc, "/Clienti/web01")).unwrap();

        assert_eq!(trace.steps.len(), 2);
        assert_eq!(trace.end, TraceEnd::Root);
    }

    #[test]
    fn own_spec_ends_the_chain_at_the_node() {
        let doc = document(vec![connection_with("dc01", "rdp", CredentialSpec::Prompt)]);

        let trace = resolve_trace(&doc, id_of(&doc, "/dc01")).unwrap();

        assert_eq!(trace.steps.len(), 1);
        assert_eq!(trace.end, TraceEnd::Prompt);
    }

    #[test]
    fn name_without_a_credential_is_reported_as_not_found() {
        let doc = document(vec![connection_with("web01", "ssh", by_name("Nessuna"))]);

        let trace = resolve_trace(&doc, id_of(&doc, "/web01")).unwrap();

        assert_eq!(
            trace.end,
            TraceEnd::NameNotFound {
                name: "Nessuna".to_owned()
            }
        );
    }

    #[test]
    fn name_of_several_credentials_lists_them() {
        let doc = document(vec![
            folder("A", vec![credential("Admin", "admin", None)]),
            folder("B", vec![credential("Admin", "root", None)]),
            connection_with("web01", "ssh", by_name("Admin")),
        ]);

        let trace = resolve_trace(&doc, id_of(&doc, "/web01")).unwrap();

        let TraceEnd::NameAmbiguous { candidates, .. } = trace.end else {
            panic!("expected an ambiguous name, got {:?}", trace.end);
        };
        let paths: Vec<String> = candidates.into_iter().map(|node| node.path).collect();
        assert_eq!(paths, vec!["/A/Admin", "/B/Admin"]);
    }

    #[test]
    fn reference_to_a_missing_node_is_dangling() {
        let missing = Uuid::new_v4();
        let doc = document(vec![connection_with("web01", "ssh", by_id(missing))]);

        let trace = resolve_trace(&doc, id_of(&doc, "/web01")).unwrap();

        assert_eq!(trace.end, TraceEnd::Dangling { target: missing });
    }

    #[test]
    fn credential_is_its_own_credential() {
        let doc = document(vec![credential("Admin", "admin", None)]);
        let id = id_of(&doc, "/Admin");

        let trace = resolve_trace(&doc, id).unwrap();

        assert!(trace.steps.is_empty());
        assert!(matches!(trace.end, TraceEnd::Credential { node } if node.id == id));
    }

    #[test]
    fn unsupported_node_has_no_credential() {
        let mut node = folder("vnc", vec![]);
        node.kind = NodeKind::Unsupported(portolano_core::model::Unsupported {
            source_type: "RoyalVNCConnection".to_owned(),
            raw: "<RoyalVNCConnection/>".to_owned(),
        });
        let doc = document(vec![node]);

        let result = resolve_trace(&doc, id_of(&doc, "/vnc"));

        assert!(matches!(
            result,
            Err(OpsError::Core(portolano_core::Error::NoCredential { .. }))
        ));
    }
}
