//! Read-only views of the document: the entries of `tree` and `list`, the
//! one-line summaries and the effective credential of `show --resolved`
//! (SPEC §10.2). Secrets appear only as present or absent.

use portolano_connectors::Registry;
use portolano_core::{
    CredentialOrigin, Document, Node, NodeKind, PrivateKey, SecretRef, Vault, VaultError,
    escape_path_segment,
};
use serde::Serialize;
use uuid::Uuid;

use crate::Result;
use crate::tree_util::node_path;

/// The kind of a node, with the protocol of a connection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NodeType {
    /// A folder.
    Folder,
    /// A connection.
    Connection {
        /// The protocol id, such as `"ssh"`.
        protocol: String,
    },
    /// A credential.
    Credential,
    /// A node Portolano does not support.
    Unsupported {
        /// The type in the source system.
        source_type: String,
    },
}

impl NodeType {
    /// The type of `node`.
    pub fn of(node: &Node) -> NodeType {
        match &node.kind {
            NodeKind::Folder(_) => NodeType::Folder,
            NodeKind::Connection(connection) => NodeType::Connection {
                protocol: connection.protocol.clone(),
            },
            NodeKind::Credential(_) => NodeType::Credential,
            NodeKind::Unsupported(unsupported) => NodeType::Unsupported {
                source_type: unsupported.source_type.clone(),
            },
        }
    }

    /// A short identifier: `folder`, `credential`, `unsupported`, or the
    /// protocol of a connection (the values of `list --type`).
    pub fn label(&self) -> &str {
        match self {
            NodeType::Folder => "folder",
            NodeType::Connection { protocol } => protocol,
            NodeType::Credential => "credential",
            NodeType::Unsupported { .. } => "unsupported",
        }
    }
}

/// A node named by its id and its path, as the views list them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct NodeRef {
    /// The node.
    pub id: Uuid,
    /// Its path.
    pub path: String,
}

impl NodeRef {
    /// Node `id` of `doc`, with its path.
    pub fn of(doc: &Document, id: Uuid) -> NodeRef {
        NodeRef {
            id,
            path: node_path(doc, id),
        }
    }
}

/// Which nodes `list --type` keeps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeFilter {
    /// Folders.
    Folder,
    /// Connections of any protocol.
    Connection,
    /// Credentials.
    Credential,
    /// Unsupported nodes.
    Unsupported,
    /// Connections of one protocol.
    Protocol(String),
}

impl TypeFilter {
    fn matches(&self, node_type: &NodeType) -> bool {
        match (self, node_type) {
            (TypeFilter::Folder, NodeType::Folder)
            | (TypeFilter::Connection, NodeType::Connection { .. })
            | (TypeFilter::Credential, NodeType::Credential)
            | (TypeFilter::Unsupported, NodeType::Unsupported { .. }) => true,
            (TypeFilter::Protocol(wanted), NodeType::Connection { protocol }) => wanted == protocol,
            _ => false,
        }
    }
}

/// A node as shown by `tree` and `list`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Entry {
    /// The node id.
    pub id: Uuid,
    /// Depth below the starting point: 0 for its children.
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub depth: usize,
    /// The node name.
    pub name: String,
    /// The node path.
    pub path: String,
    /// The node type.
    #[serde(flatten)]
    pub node_type: NodeType,
    /// One-line summary, see [`summary`].
    pub summary: String,
}

/// The descendants of `start` (`None` = the whole document), pre-order.
///
/// `max_depth` keeps only the first levels (1 = the children only);
/// `filter` keeps only the matching nodes, still visiting every level.
///
/// # Errors
/// [`portolano_core::Error::NodeNotFound`] for a missing starting point.
pub fn entries(
    doc: &Document,
    start: Option<Uuid>,
    protocols: &Registry,
    max_depth: Option<usize>,
    filter: Option<&TypeFilter>,
) -> Result<Vec<Entry>> {
    let mut found = Vec::new();
    let children = match start {
        None => doc.nodes.as_slice(),
        Some(id) => doc
            .find(id)
            .ok_or(portolano_core::Error::NodeNotFound { id })?
            .as_folder()
            // A leaf has no children: it lists nothing.
            .map_or(&[][..], |folder| folder.children.as_slice()),
    };
    let visit = Visit {
        doc,
        protocols,
        max_depth,
        filter,
    };
    // Paths are built while visiting: asking the document for the path of
    // every node would walk the tree once per node.
    let base = start.and_then(|id| doc.path_of(id)).unwrap_or_default();
    visit.collect(children, &base, 0, &mut found);
    Ok(found)
}

struct Visit<'a> {
    doc: &'a Document,
    protocols: &'a Registry,
    max_depth: Option<usize>,
    filter: Option<&'a TypeFilter>,
}

impl Visit<'_> {
    fn collect(&self, nodes: &[Node], parent_path: &str, depth: usize, found: &mut Vec<Entry>) {
        if self.max_depth.is_some_and(|max| depth >= max) {
            return;
        }
        for node in nodes {
            let path = format!("{parent_path}/{}", escape_path_segment(&node.name));
            let node_type = NodeType::of(node);
            if self.filter.is_none_or(|filter| filter.matches(&node_type)) {
                found.push(Entry {
                    id: node.id,
                    depth,
                    name: node.name.clone(),
                    path: path.clone(),
                    node_type,
                    summary: summary(self.doc, node, self.protocols),
                });
            }
            if let Some(folder) = node.as_folder() {
                self.collect(&folder.children, &path, depth + 1, found);
            }
        }
    }
}

/// One-line summary of a node: the protocol's summary for a connection
/// (`admin@web01.example.com:22`), `DOMINIO\utente` for a credential,
/// `non supportato: <tipo>` for an unsupported node, nothing for a folder.
pub fn summary(doc: &Document, node: &Node, protocols: &Registry) -> String {
    match &node.kind {
        NodeKind::Folder(_) => String::new(),
        NodeKind::Connection(connection) => match protocols.get(&connection.protocol) {
            Some(protocol) => protocol.summary(doc, node),
            None => format!(
                "{} (protocollo sconosciuto: {})",
                connection.host, connection.protocol
            ),
        },
        NodeKind::Credential(credential) => {
            login_name(credential.username.as_deref(), credential.domain.as_deref())
        }
        NodeKind::Unsupported(unsupported) => {
            format!("non supportato: {}", unsupported.source_type)
        }
    }
}

/// `DOMINIO\utente`, `utente`, or nothing.
fn login_name(username: Option<&str>, domain: Option<&str>) -> String {
    match (domain, username) {
        (Some(domain), Some(user)) => format!("{domain}\\{user}"),
        (None, Some(user)) => user.to_owned(),
        (Some(domain), None) => format!("{domain}\\"),
        (None, None) => String::new(),
    }
}

/// Whether a secret is set in the document and present in the vault.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "snake_case")]
pub enum SecretState {
    /// The document holds no reference.
    NotSet,
    /// The document holds a reference and the vault has the secret.
    Present,
    /// The document holds a reference but the vault has no entry.
    Absent,
}

impl SecretState {
    /// The state of an optional reference in `vault`.
    ///
    /// # Errors
    /// Errors of the vault.
    pub fn of(reference: Option<&SecretRef>, vault: &dyn Vault) -> Result<SecretState, VaultError> {
        Ok(match reference {
            None => SecretState::NotSet,
            Some(reference) if vault.contains(reference)? => SecretState::Present,
            Some(_) => SecretState::Absent,
        })
    }
}

/// Where the effective credential comes from, with paths.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OriginView {
    /// No credential.
    None,
    /// Asked at connection time.
    Prompt,
    /// Inline data of a node.
    Inline {
        /// The node holding the data.
        node: Uuid,
        /// Its path.
        path: String,
    },
    /// A credential node.
    Credential {
        /// The credential node.
        node: Uuid,
        /// Its path.
        path: String,
    },
}

/// The private key of an effective credential.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum KeyView {
    /// A key file.
    File {
        /// Its path.
        path: String,
    },
    /// A key in the vault.
    Vault {
        /// Whether the vault has it.
        state: SecretState,
    },
}

/// The effective credential of a node, without secrets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct CredentialView {
    /// Where it comes from.
    pub origin: OriginView,
    /// User name.
    pub username: Option<String>,
    /// Windows domain.
    pub domain: Option<String>,
    /// The password.
    pub password: SecretState,
    /// The private key.
    pub private_key: Option<KeyView>,
    /// The key passphrase.
    pub passphrase: SecretState,
}

/// The effective credential of node `id` (SPEC §4.3), with the presence of
/// its secrets in `vault`.
///
/// # Errors
/// Resolution errors of the core (missing or ambiguous credential, a node
/// without credentials), errors of the vault.
pub fn resolved_credential(doc: &Document, id: Uuid, vault: &dyn Vault) -> Result<CredentialView> {
    let resolved = doc.resolve_credential(id)?;
    let origin = match resolved.origin {
        CredentialOrigin::None => OriginView::None,
        CredentialOrigin::Prompt => OriginView::Prompt,
        CredentialOrigin::Inline { node } => OriginView::Inline {
            node,
            path: node_path(doc, node),
        },
        CredentialOrigin::Credential { node } => OriginView::Credential {
            node,
            path: node_path(doc, node),
        },
    };
    let private_key = match &resolved.private_key {
        None => None,
        Some(PrivateKey::File { path }) => Some(KeyView::File { path: path.clone() }),
        Some(PrivateKey::Vault { secret }) => Some(KeyView::Vault {
            state: SecretState::of(Some(secret), vault)?,
        }),
    };
    Ok(CredentialView {
        origin,
        password: SecretState::of(resolved.password.as_ref(), vault)?,
        passphrase: SecretState::of(resolved.passphrase.as_ref(), vault)?,
        username: resolved.username,
        domain: resolved.domain,
        private_key,
    })
}

#[cfg(test)]
mod tests;
