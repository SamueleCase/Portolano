//! Tree operations on a [`Document`] (SPEC §4.1).
//!
//! The code is recursive: documents are assumed to be less than 1000 levels
//! deep (SPEC §4.1).

use std::collections::HashSet;
use std::slice;

use serde::Deserialize;
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::model::{CredentialSpec, Document, Node, NodeKind, PrivateKey, PropertyValue};
use crate::secret::SecretRef;
use crate::validate::for_each_object;

impl Document {
    /// The node with the given id, at any depth.
    pub fn find(&self, id: Uuid) -> Option<&Node> {
        self.walk().map(|(_, node)| node).find(|node| node.id == id)
    }

    /// The mutable node with the given id, at any depth.
    pub fn find_mut(&mut self, id: Uuid) -> Option<&mut Node> {
        find_in_mut(&mut self.nodes, id)
    }

    /// The parent of a node: `None` if the node does not exist, `Some(None)`
    /// if it is a child of the root, `Some(Some(parent))` otherwise.
    pub fn parent_of(&self, id: Uuid) -> Option<Option<Uuid>> {
        if self.nodes.iter().any(|node| node.id == id) {
            return Some(None);
        }
        self.walk()
            .map(|(_, node)| node)
            .find(|node| {
                node.as_folder()
                    .is_some_and(|folder| folder.children.iter().any(|child| child.id == id))
            })
            .map(|parent| Some(parent.id))
    }

    /// The ancestors of a node, from its parent up to the outermost folder.
    /// Empty for a child of the root or a missing node.
    pub fn ancestors(&self, id: Uuid) -> Vec<Uuid> {
        let mut ancestors = Vec::new();
        let mut current = id;
        while let Some(Some(parent)) = self.parent_of(current) {
            ancestors.push(parent);
            current = parent;
        }
        ancestors
    }

    /// The children of a folder, or of the root when `parent` is `None`.
    ///
    /// # Errors
    /// [`Error::NodeNotFound`](crate::Error::NodeNotFound) if the parent does
    /// not exist, [`Error::NotAFolder`](crate::Error::NotAFolder) if it is
    /// not a folder.
    pub fn children(&self, parent: Option<Uuid>) -> Result<&[Node]> {
        let Some(id) = parent else {
            return Ok(&self.nodes);
        };
        let node = self.find(id).ok_or(Error::NodeNotFound { id })?;
        let folder = node.as_folder().ok_or(Error::NotAFolder { id })?;
        Ok(&folder.children)
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
        let mut known: HashSet<Uuid> = self.walk().map(|(_, node)| node.id).collect();
        // Checking the subtree against itself too keeps ids unique even when
        // the inserted subtree repeats one of its own ids.
        if let Some((_, duplicate)) =
            walk_nodes(slice::from_ref(&node)).find(|(_, inserted)| !known.insert(inserted.id))
        {
            return Err(Error::DuplicateId { id: duplicate.id });
        }
        self.place(parent, position, node)
    }

    /// Detaches a node with its subtree and returns it.
    ///
    /// # Errors
    /// [`Error::NodeNotFound`](crate::Error::NodeNotFound) if the node does
    /// not exist.
    pub fn remove(&mut self, id: Uuid) -> Result<Node> {
        let parent = self.parent_of(id).ok_or(Error::NodeNotFound { id })?;
        let siblings = self.children_mut(parent)?;
        let index = siblings
            .iter()
            .position(|node| node.id == id)
            .ok_or(Error::NodeNotFound { id })?;
        Ok(siblings.remove(index))
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
        if self.find(id).is_none() {
            return Err(Error::NodeNotFound { id });
        }
        // Every check happens before the node is detached, so a rejected move
        // leaves the document untouched.
        if let Some(target) = new_parent {
            if target == id || self.ancestors(target).contains(&id) {
                return Err(Error::InvalidMove {
                    id,
                    new_parent: target,
                });
            }
            self.children(new_parent)?;
        }
        // Once the node is detached, an index into the remaining siblings is
        // exactly its index in the final list.
        let mut node = self.remove(id)?;
        node.touch();
        self.place(new_parent, position, node)
    }

    /// Pre-order traversal of the whole tree, yielding `(depth, node)`;
    /// children of the root have depth 0.
    pub fn walk(&self) -> impl Iterator<Item = (usize, &Node)> {
        walk_nodes(&self.nodes)
    }

    /// Every [`SecretRef`] in the subtree rooted at `id` (the node included).
    /// Empty if the node does not exist.
    ///
    /// Besides the typed fields, the protocol `settings` are searched for
    /// objects in the form of an inline [`CredentialSpec`] (DEC-014).
    pub fn secret_refs_in(&self, id: Uuid) -> Vec<SecretRef> {
        let Some(root) = self.find(id) else {
            return Vec::new();
        };
        let mut refs = Vec::new();
        for (_, node) in walk_nodes(slice::from_ref(root)) {
            collect_secret_refs(node, &mut refs);
        }
        refs
    }

    /// Inserts `node` without checking its ids, for nodes already known to be
    /// unique (a moved node, or one checked by [`Document::insert`]).
    fn place(&mut self, parent: Option<Uuid>, position: Option<usize>, node: Node) -> Result<()> {
        let siblings = self.children_mut(parent)?;
        let index = position.map_or(siblings.len(), |position| position.min(siblings.len()));
        siblings.insert(index, node);
        Ok(())
    }

    fn children_mut(&mut self, parent: Option<Uuid>) -> Result<&mut Vec<Node>> {
        let Some(id) = parent else {
            return Ok(&mut self.nodes);
        };
        let node = self.find_mut(id).ok_or(Error::NodeNotFound { id })?;
        let folder = node.as_folder_mut().ok_or(Error::NotAFolder { id })?;
        Ok(&mut folder.children)
    }
}

/// Pre-order traversal of `nodes` and their descendants, yielding
/// `(depth, node)`; the nodes of `nodes` have depth 0.
pub(crate) fn walk_nodes(nodes: &[Node]) -> impl Iterator<Item = (usize, &Node)> {
    Walk {
        levels: vec![nodes.iter()],
    }
}

/// Iterative pre-order traversal: one slice iterator per open level, so the
/// depth is the number of open levels minus one.
struct Walk<'a> {
    levels: Vec<slice::Iter<'a, Node>>,
}

impl<'a> Iterator for Walk<'a> {
    type Item = (usize, &'a Node);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let depth = self.levels.len().checked_sub(1)?;
            let level = self.levels.last_mut()?;
            match level.next() {
                Some(node) => {
                    if let Some(folder) = node.as_folder() {
                        self.levels.push(folder.children.iter());
                    }
                    return Some((depth, node));
                }
                None => {
                    self.levels.pop();
                }
            }
        }
    }
}

fn find_in_mut(nodes: &mut [Node], id: Uuid) -> Option<&mut Node> {
    nodes.iter_mut().find_map(|node| {
        if node.id == id {
            Some(node)
        } else {
            node.as_folder_mut()
                .and_then(|folder| find_in_mut(&mut folder.children, id))
        }
    })
}

/// Appends the secret references held by `node` itself, children excluded,
/// to `refs`.
pub(crate) fn collect_secret_refs(node: &Node, refs: &mut Vec<SecretRef>) {
    refs.extend(
        node.custom_properties
            .iter()
            .filter_map(|property| match &property.value {
                PropertyValue::Secret { secret } => Some(secret.clone()),
                PropertyValue::Text { .. } => None,
            }),
    );
    match &node.kind {
        NodeKind::Folder(folder) => refs.extend(inline_password(&folder.credential)),
        NodeKind::Connection(connection) => {
            refs.extend(inline_password(&connection.credential));
            for value in connection.settings.values() {
                for_each_object(value, &mut |object| {
                    if object.get("mode").and_then(serde_json::Value::as_str) == Some("inline") {
                        // A malformed object is not a credential spec, so it
                        // holds no reference to collect.
                        if let Ok(spec) = CredentialSpec::deserialize(object) {
                            refs.extend(inline_password(&spec));
                        }
                    }
                });
            }
        }
        NodeKind::Credential(credential) => {
            refs.extend(credential.password.iter().cloned());
            if let Some(PrivateKey::Vault { secret }) = &credential.private_key {
                refs.push(secret.clone());
            }
            refs.extend(credential.passphrase.iter().cloned());
        }
        NodeKind::Unsupported(_) => {}
    }
}

fn inline_password(spec: &CredentialSpec) -> Option<SecretRef> {
    match spec {
        CredentialSpec::Inline { password, .. } => password.clone(),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
