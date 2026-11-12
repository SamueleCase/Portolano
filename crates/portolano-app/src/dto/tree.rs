//! The tree of the main window (SPEC-UI §4.3, §5.4).

use portolano_connectors::Registry;
use portolano_core::{Document, Node, NodeKind, Vault, VaultError};
use portolano_ops::ErrorCategory;
use portolano_ops::inspect::{SecretState, own_secrets_state, summary};
use serde::Serialize;
use uuid::Uuid;

use super::icon::IconView;
use crate::error::IpcError;

/// The kind of a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum NodeKindView {
    /// A folder.
    Folder,
    /// A connection; its protocol is next to it.
    Connection,
    /// A credential.
    Credential,
    /// A node Portolano does not support, shown read-only.
    Unsupported,
}

impl NodeKindView {
    /// The kind of `node`.
    pub fn of(node: &Node) -> Self {
        match node.kind {
            NodeKind::Folder(_) => NodeKindView::Folder,
            NodeKind::Connection(_) => NodeKindView::Connection,
            NodeKind::Credential(_) => NodeKindView::Credential,
            NodeKind::Unsupported(_) => NodeKindView::Unsupported,
        }
    }
}

/// The secrets of a node itself, for the badge (SPEC-UI §1.1: a state,
/// never a value).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum SecretsBadge {
    /// The node holds no secret.
    None,
    /// The vault has all of them.
    Present,
    /// The vault lacks at least one.
    Absent,
    /// The vault is locked: their presence is unknown.
    VaultLocked,
}

/// A node of the tree, with its children.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct TreeNode {
    /// The node.
    pub id: Uuid,
    /// Its name, as text (SPEC-UI §1.3).
    pub name: String,
    /// Its kind.
    pub kind: NodeKindView,
    /// The protocol of a connection.
    pub protocol: Option<String>,
    /// One-line summary: `admin@web01.example.com:22`, `DOMINIO\utente`, …
    pub summary: String,
    /// Color `#RRGGBB`.
    pub color: Option<String>,
    /// Icon; none means the icon of the kind.
    pub icon: Option<IconView>,
    /// Favorite.
    pub favorite: bool,
    /// Tags.
    pub tags: Vec<String>,
    /// Its own secrets.
    pub secrets: SecretsBadge,
    /// All its descendants (the count of a closed folder, ambiguity 21).
    pub descendants: u32,
    /// Its children, in order.
    pub children: Vec<TreeNode>,
}

/// The whole document as a tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct TreeView {
    /// The name of the document.
    pub name: String,
    /// The nodes under the root.
    pub nodes: Vec<TreeNode>,
}

/// The tree of `doc`, with the state of the secrets in `vault`.
///
/// # Errors
/// Errors of the vault other than a locked vault, which makes every badge
/// [`SecretsBadge::VaultLocked`] instead.
pub fn tree_view(
    doc: &Document,
    protocols: &Registry,
    vault: &dyn Vault,
) -> Result<TreeView, IpcError> {
    let mut builder = Builder {
        doc,
        protocols,
        vault,
        locked: false,
    };
    Ok(TreeView {
        name: doc.name.clone(),
        nodes: builder.nodes(&doc.nodes)?,
    })
}

struct Builder<'a> {
    doc: &'a Document,
    protocols: &'a Registry,
    vault: &'a dyn Vault,
    /// The vault answered "locked" once: it is not asked again.
    locked: bool,
}

impl Builder<'_> {
    fn nodes(&mut self, nodes: &[Node]) -> Result<Vec<TreeNode>, IpcError> {
        nodes.iter().map(|node| self.node(node)).collect()
    }

    fn node(&mut self, node: &Node) -> Result<TreeNode, IpcError> {
        let children = match node.as_folder() {
            Some(folder) => self.nodes(&folder.children)?,
            None => Vec::new(),
        };
        let descendants = children.iter().map(|child| child.descendants + 1).sum();
        Ok(TreeNode {
            id: node.id,
            name: node.name.clone(),
            kind: NodeKindView::of(node),
            protocol: node.as_connection().map(|c| c.protocol.clone()),
            summary: summary(self.doc, node, self.protocols),
            color: node.color.clone(),
            icon: node.icon.as_ref().and_then(IconView::of),
            favorite: node.favorite,
            tags: node.tags.clone(),
            secrets: self.badge(node)?,
            descendants,
            children,
        })
    }

    fn badge(&mut self, node: &Node) -> Result<SecretsBadge, IpcError> {
        let state = if self.locked {
            own_secrets_state(self.doc, node, &LockedVault)
        } else {
            own_secrets_state(self.doc, node, self.vault)
        };
        match state {
            Ok(state) => Ok(badge_of(state)),
            Err(error) if ErrorCategory::from(&error) == ErrorCategory::VaultLocked => {
                self.locked = true;
                Ok(SecretsBadge::VaultLocked)
            }
            Err(error) => Err(portolano_ops::OpsError::from(error).into()),
        }
    }
}

/// The badge of a secret state.
pub fn badge_of(state: SecretState) -> SecretsBadge {
    match state {
        SecretState::NotSet => SecretsBadge::None,
        SecretState::Present => SecretsBadge::Present,
        SecretState::Absent => SecretsBadge::Absent,
    }
}

/// Answers every question as a locked vault, once the real one did: asking
/// it again for every node would only repeat the answer.
struct LockedVault;

impl Vault for LockedVault {
    fn backend(&self) -> &'static str {
        "locked"
    }

    fn get(
        &self,
        _: &portolano_core::SecretRef,
    ) -> Result<Option<portolano_core::Secret>, VaultError> {
        Err(locked())
    }

    fn set(
        &self,
        _: &portolano_core::SecretRef,
        _: &portolano_core::Secret,
    ) -> Result<(), VaultError> {
        Err(locked())
    }

    fn delete(&self, _: &portolano_core::SecretRef) -> Result<(), VaultError> {
        Err(locked())
    }
}

fn locked() -> VaultError {
    VaultError::MasterPasswordUnavailable {
        reason: "vault bloccato".to_owned(),
    }
}
