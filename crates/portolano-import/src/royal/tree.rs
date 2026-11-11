//! Rebuilding of the tree from the flat list of objects (SPEC §8.3.3):
//! parents from `ParentID`, siblings ordered by numeric `PositionNr` and then
//! by file order.

use std::collections::HashMap;

use uuid::Uuid;

use super::xml::RawObject;
use super::{UNNAMED, defaults, types};
use crate::ImportError;

/// An object and its children, in display order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct TreeNode {
    /// Index of the object in the parsed list.
    pub(super) object: usize,
    pub(super) children: Vec<TreeNode>,
}

/// The tree of a document.
#[derive(Debug)]
pub(super) struct Tree {
    /// The `RoyalDocument` and everything under it.
    pub(super) root: TreeNode,
    /// The `ID` of each object, by index.
    pub(super) ids: Vec<Uuid>,
    /// Objects moved under the root, with the reason.
    pub(super) warnings: Vec<String>,
}

/// Builds the tree. Objects whose parent is missing, is not a container or
/// is part of a `ParentID` cycle go under the root, with a warning.
///
/// # Errors
/// [`ImportError::InvalidInput`] if an object has no valid `ID`, two objects
/// share one, or there is not exactly one `RoyalDocument`.
pub(super) fn build(objects: &[RawObject]) -> Result<Tree, ImportError> {
    let ids = parse_ids(objects)?;
    let root = find_document(objects)?;
    let index: HashMap<Uuid, usize> = ids.iter().enumerate().map(|(i, id)| (*id, i)).collect();

    let mut warnings = Vec::new();
    let mut children = vec![Vec::new(); objects.len()];
    for (object, raw) in objects.iter().enumerate().filter(|(i, _)| *i != root) {
        let parent = match find_parent(objects, &index, object) {
            Ok(parent) => parent,
            Err(problem) => {
                warnings.push(format!(
                    "{}: {problem}, inserito nella radice",
                    describe(raw)
                ));
                root
            }
        };
        children[parent].push(object);
    }
    for siblings in &mut children {
        siblings.sort_by_key(|&object| (position_nr(&objects[object]), object));
    }

    let mut visited = vec![false; objects.len()];
    let mut root_node = subtree(root, &children, &mut visited);
    for object in 0..objects.len() {
        if !visited[object] {
            warnings.push(format!(
                "{}: ParentID ciclico, inserito nella radice",
                describe(&objects[object])
            ));
            root_node
                .children
                .push(subtree(object, &children, &mut visited));
        }
    }
    Ok(Tree {
        root: root_node,
        ids,
        warnings,
    })
}

fn parse_ids(objects: &[RawObject]) -> Result<Vec<Uuid>, ImportError> {
    let mut seen: HashMap<Uuid, usize> = HashMap::new();
    let mut ids = Vec::with_capacity(objects.len());
    for (index, object) in objects.iter().enumerate() {
        let ordinal = index + 1;
        let type_name = &object.type_name;
        let value = object
            .value("ID")
            .ok_or_else(|| ImportError::InvalidInput {
                reason: format!("l'oggetto n. {ordinal} ({type_name}) non ha un ID"),
            })?;
        let id = Uuid::parse_str(value).map_err(|_| ImportError::InvalidInput {
            reason: format!("l'oggetto n. {ordinal} ({type_name}) ha un ID non valido"),
        })?;
        if let Some(first) = seen.insert(id, ordinal) {
            return Err(ImportError::InvalidInput {
                reason: format!("gli oggetti n. {first} e n. {ordinal} hanno lo stesso ID"),
            });
        }
        ids.push(id);
    }
    Ok(ids)
}

fn find_document(objects: &[RawObject]) -> Result<usize, ImportError> {
    let mut documents = objects
        .iter()
        .enumerate()
        .filter(|(_, object)| object.type_name == types::DOCUMENT)
        .map(|(index, _)| index);
    match (documents.next(), documents.next()) {
        (Some(document), None) => Ok(document),
        (None, _) => Err(ImportError::InvalidInput {
            reason: format!("manca l'oggetto {}", types::DOCUMENT),
        }),
        (Some(_), Some(_)) => Err(ImportError::InvalidInput {
            reason: format!("più di un oggetto {}", types::DOCUMENT),
        }),
    }
}

/// The parent of `object`, or why it cannot be used.
fn find_parent(
    objects: &[RawObject],
    index: &HashMap<Uuid, usize>,
    object: usize,
) -> Result<usize, &'static str> {
    let parent_id = objects[object]
        .value("ParentID")
        .ok_or("ParentID assente")?;
    let parent = Uuid::parse_str(parent_id)
        .ok()
        .and_then(|id| index.get(&id).copied())
        .ok_or("genitore non trovato nel documento")?;
    if parent == object {
        Err("l'oggetto è genitore di sé stesso")
    } else if !types::is_container(&objects[parent].type_name) {
        Err("il genitore non è una cartella")
    } else {
        Ok(parent)
    }
}

fn subtree(object: usize, children: &[Vec<usize>], visited: &mut [bool]) -> TreeNode {
    visited[object] = true;
    let mut node = TreeNode {
        object,
        children: Vec::new(),
    };
    for &child in &children[object] {
        if !visited[child] {
            node.children.push(subtree(child, children, visited));
        }
    }
    node
}

/// `PositionNr` is a string to be compared as a number (`"10"` after `"8"`).
fn position_nr(object: &RawObject) -> i64 {
    object
        .value("PositionNr")
        .and_then(|value| value.parse().ok())
        .unwrap_or(defaults::POSITION_NR)
}

fn describe(object: &RawObject) -> String {
    let name = object.value("Name").unwrap_or(UNNAMED);
    format!("oggetto `{name}` ({})", object.type_name)
}
