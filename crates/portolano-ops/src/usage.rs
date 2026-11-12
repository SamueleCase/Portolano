//! Who uses a node (SPEC-UI §3.6 `credential_usage`): the nodes that refer
//! to it (DEC-018.1) and, for a credential, the connections that receive it
//! by inheritance from a folder (ambiguity 13 of the mock).

use std::collections::HashSet;

use portolano_core::{CredentialOrigin, Document};
use serde::Serialize;
use uuid::Uuid;

use crate::Result;
use crate::inspect::NodeRef;
use crate::nodes::credential_spec;

/// The users of a node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct CredentialUsage {
    /// Nodes that refer to it by id or by name, in tree order.
    pub direct: Vec<NodeRef>,
    /// Connections that do not refer to it but inherit it from a folder
    /// that does, in tree order.
    pub inherited: Vec<NodeRef>,
}

/// The users of node `id`. Only a credential can be inherited, so for any
/// other node `inherited` is empty.
///
/// # Errors
/// [`portolano_core::Error::NodeNotFound`].
pub fn credential_usage(doc: &Document, id: Uuid) -> Result<CredentialUsage> {
    let node = doc
        .find(id)
        .ok_or(portolano_core::Error::NodeNotFound { id })?;
    let direct_ids = doc.references_to(id);
    let inherited = if node.as_credential().is_some() {
        inheriting_connections(doc, id, &direct_ids.iter().copied().collect())
    } else {
        Vec::new()
    };
    Ok(CredentialUsage {
        direct: direct_ids
            .into_iter()
            .map(|from| NodeRef::of(doc, from))
            .collect(),
        inherited,
    })
}

/// The connections whose own spec is `Inherit` and whose effective
/// credential is `credential`, except those in `direct`.
fn inheriting_connections(
    doc: &Document,
    credential: Uuid,
    direct: &HashSet<Uuid>,
) -> Vec<NodeRef> {
    doc.walk()
        .map(|(_, node)| node)
        .filter(|node| node.as_connection().is_some() && !direct.contains(&node.id))
        .filter(|node| credential_spec(node).is_some_and(|spec| spec.is_inherit()))
        // A chain that does not resolve uses no credential: it is not a user.
        .filter(|node| {
            doc.resolve_credential(node.id).is_ok_and(|resolved| {
                resolved.origin == CredentialOrigin::Credential { node: credential }
            })
        })
        .map(|node| NodeRef::of(doc, node.id))
        .collect()
}

#[cfg(test)]
mod tests {
    use portolano_core::CredentialSpec;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::testutil::*;

    fn paths(nodes: &[NodeRef]) -> Vec<&str> {
        nodes.iter().map(|node| node.path.as_str()).collect()
    }

    /// `/Admin`, and `/Prod` that defaults to it with `web01` (inheriting),
    /// `web02` (by name) and `db01` (its own inline login).
    fn production() -> Document {
        let admin = credential("Admin", "admin", None);
        let admin_id = admin.id;
        let mut prod = folder(
            "Prod",
            vec![
                connection("web01", "ssh"),
                connection_with(
                    "web02",
                    "ssh",
                    CredentialSpec::ByName {
                        name: "Admin".to_owned(),
                    },
                ),
                connection_with(
                    "db01",
                    "ssh",
                    CredentialSpec::Inline {
                        username: Some("dba".to_owned()),
                        domain: None,
                        password: None,
                    },
                ),
            ],
        );
        if let Some(folder) = prod.as_folder_mut() {
            folder.credential = CredentialSpec::ById { id: admin_id };
        }
        document(vec![admin, prod])
    }

    #[test]
    fn references_by_id_and_by_name_are_direct() {
        let doc = production();

        let usage = credential_usage(&doc, id_of(&doc, "/Admin")).unwrap();

        assert_eq!(paths(&usage.direct), vec!["/Prod", "/Prod/web02"]);
    }

    #[test]
    fn connections_that_inherit_the_credential_are_inherited() {
        let doc = production();

        let usage = credential_usage(&doc, id_of(&doc, "/Admin")).unwrap();

        assert_eq!(paths(&usage.inherited), vec!["/Prod/web01"]);
    }

    #[test]
    fn node_that_is_not_a_credential_has_only_direct_users() {
        let bastion = connection("bastion", "ssh");
        let mut web01 = connection("web01", "ssh");
        if let Some(connection) = web01.as_connection_mut() {
            connection.settings.insert(
                "jump_hosts".to_owned(),
                serde_json::json!([{ "$ref": bastion.id }]),
            );
        }
        let doc = document(vec![bastion, web01]);

        let usage = credential_usage(&doc, id_of(&doc, "/bastion")).unwrap();

        assert_eq!(paths(&usage.direct), vec!["/web01"]);
        assert!(usage.inherited.is_empty());
    }

    #[test]
    fn usage_of_a_missing_node_is_not_found() {
        let doc = production();

        let result = credential_usage(&doc, Uuid::new_v4());

        assert!(matches!(
            result,
            Err(crate::OpsError::Core(
                portolano_core::Error::NodeNotFound { .. }
            ))
        ));
    }
}
