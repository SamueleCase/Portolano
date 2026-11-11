//! Tree operations on a [`Document`] (SPEC §4.1).

// Stub (WP0): the parameters are used once WP1 implements the bodies.
#![expect(unused_variables)]

use uuid::Uuid;

use crate::error::Result;
use crate::model::{Document, Node};
use crate::secret::SecretRef;

impl Document {
    /// The node with the given id, at any depth.
    pub fn find(&self, id: Uuid) -> Option<&Node> {
        todo!()
    }

    /// The mutable node with the given id, at any depth.
    pub fn find_mut(&mut self, id: Uuid) -> Option<&mut Node> {
        todo!()
    }

    /// The parent of a node: `None` if the node does not exist, `Some(None)`
    /// if it is a child of the root, `Some(Some(parent))` otherwise.
    pub fn parent_of(&self, id: Uuid) -> Option<Option<Uuid>> {
        todo!()
    }

    /// The ancestors of a node, from its parent up to the outermost folder.
    /// Empty for a child of the root or a missing node.
    pub fn ancestors(&self, id: Uuid) -> Vec<Uuid> {
        todo!()
    }

    /// The children of a folder, or of the root when `parent` is `None`.
    ///
    /// # Errors
    /// [`Error::NodeNotFound`](crate::Error::NodeNotFound) if the parent does
    /// not exist, [`Error::NotAFolder`](crate::Error::NotAFolder) if it is
    /// not a folder.
    pub fn children(&self, parent: Option<Uuid>) -> Result<&[Node]> {
        todo!()
    }

    /// Inserts a node (with its subtree) under `parent` (`None` = root) at
    /// `position`. `None` appends; a position past the end is clamped to the
    /// end.
    ///
    /// # Errors
    /// [`Error::NodeNotFound`](crate::Error::NodeNotFound) or
    /// [`Error::NotAFolder`](crate::Error::NotAFolder) for a bad parent,
    /// [`Error::DuplicateId`](crate::Error::DuplicateId) if any id of the
    /// inserted subtree already exists in the document.
    pub fn insert(
        &mut self,
        parent: Option<Uuid>,
        position: Option<usize>,
        node: Node,
    ) -> Result<()> {
        todo!()
    }

    /// Detaches a node with its subtree and returns it.
    ///
    /// # Errors
    /// [`Error::NodeNotFound`](crate::Error::NodeNotFound) if the node does
    /// not exist.
    pub fn remove(&mut self, id: Uuid) -> Result<Node> {
        todo!()
    }

    /// Moves a node under `new_parent` (`None` = root). `position` is the
    /// index in the final list, so reordering within the same parent works as
    /// expected; `None` appends. Updates `modified` of the moved node.
    ///
    /// # Errors
    /// [`Error::InvalidMove`](crate::Error::InvalidMove) when moving a node
    /// into itself or a descendant;
    /// [`Error::NodeNotFound`](crate::Error::NodeNotFound) or
    /// [`Error::NotAFolder`](crate::Error::NotAFolder) for a bad node or parent.
    pub fn move_node(
        &mut self,
        id: Uuid,
        new_parent: Option<Uuid>,
        position: Option<usize>,
    ) -> Result<()> {
        todo!()
    }

    /// Pre-order traversal of the whole tree, yielding `(depth, node)`;
    /// children of the root have depth 0.
    pub fn walk(&self) -> impl Iterator<Item = (usize, &Node)> {
        // Stub (WP0): `todo!()` alone has type `!`, which is not an iterator.
        std::iter::from_fn(|| todo!())
    }

    /// Every [`SecretRef`] in the subtree rooted at `id` (the node included).
    /// Empty if the node does not exist.
    pub fn secret_refs_in(&self, id: Uuid) -> Vec<SecretRef> {
        todo!()
    }
}
