//! Rebuilding of the tree from the flat list of objects (SPEC §8.3.3):
//! parents from `ParentID`, siblings ordered by numeric `PositionNr` and then
//! by file order.

use std::collections::{HashMap, HashSet, VecDeque};

use portolano_core::MAX_DEPTH;
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
    /// Whether each object, by index, is the trash or is in it according
    /// to its `ParentID`s, wherever it ended up in the tree.
    pub(super) in_trash: Vec<bool>,
    /// Objects moved under the root because they were deeper than
    /// [`MAX_DEPTH`] (DEC-021).
    pub(super) too_deep: HashSet<usize>,
    /// Objects moved under the root, with the reason.
    pub(super) warnings: Vec<String>,
}

impl Tree {
    /// Whether the object is skipped as part of the trash.
    pub(super) fn is_skipped_trash(&self, object: usize, include_trash: bool) -> bool {
        !include_trash && self.in_trash[object]
    }
}

/// Builds the tree. Objects whose parent is missing, is not a container or
/// is part of a `ParentID` cycle go under the root, with a warning; so do
/// objects deeper than [`MAX_DEPTH`] levels, the `RoyalDocument` being the
/// first level, since the import root is inserted as one node.
///
/// # Errors
/// [`ImportError::InvalidInput`] if an object has no valid `ID`, two objects
/// share one, or there is not exactly one `RoyalDocument`.
pub(super) fn build(objects: &[RawObject]) -> Result<Tree, ImportError> {
    let ids = parse_ids(objects)?;
    let root = find_document(objects)?;
    let index: HashMap<Uuid, usize> = ids.iter().enumerate().map(|(i, id)| (*id, i)).collect();

    let mut warnings = Vec::new();
    let mut parents = vec![root; objects.len()];
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
        parents[object] = parent;
        children[parent].push(object);
    }
    for siblings in &mut children {
        siblings.sort_by_key(|&object| (position_nr(&objects[object]), object));
    }

    let mut builder = Builder {
        objects,
        children: &children,
        visited: vec![false; objects.len()],
        overflow: VecDeque::new(),
        too_deep: HashSet::new(),
    };
    let mut root_node = builder.subtree(root, 1);
    builder.attach_overflow(&mut root_node, &mut warnings);
    for object in 0..objects.len() {
        if !builder.visited[object] {
            // Not reachable from the root: its ancestors end in a cycle. Only
            // that cycle member is detached; the rest keeps its parent.
            let member = cycle_member(object, &parents);
            warnings.push(format!(
                "{}: ParentID ciclico, inserito nella radice",
                describe(&objects[member])
            ));
            root_node.children.push(builder.subtree(member, 2));
            builder.attach_overflow(&mut root_node, &mut warnings);
        }
    }
    Ok(Tree {
        root: root_node,
        ids,
        in_trash: trash_membership(objects, &parents, root),
        too_deep: builder.too_deep,
        warnings,
    })
}

/// Builds subtrees no deeper than [`MAX_DEPTH`]: the recursion stays
/// shallow however deep the document is.
struct Builder<'a> {
    objects: &'a [RawObject],
    children: &'a [Vec<usize>],
    visited: Vec<bool>,
    /// Objects found beyond the maximum depth, to attach to the root.
    overflow: VecDeque<usize>,
    too_deep: HashSet<usize>,
}

impl Builder<'_> {
    /// The subtree of `object`, placed at `level`.
    fn subtree(&mut self, object: usize, level: usize) -> TreeNode {
        self.visited[object] = true;
        let mut node = TreeNode {
            object,
            children: Vec::new(),
        };
        for &child in &self.children[object] {
            if self.visited[child] {
                continue;
            }
            if level < MAX_DEPTH {
                node.children.push(self.subtree(child, level + 1));
            } else {
                self.visited[child] = true;
                self.overflow.push_back(child);
            }
        }
        node
    }

    /// Attaches the objects beyond the maximum depth to the root, as its
    /// children at level 2; their own deep descendants follow in turn.
    fn attach_overflow(&mut self, root: &mut TreeNode, warnings: &mut Vec<String>) {
        while let Some(object) = self.overflow.pop_front() {
            warnings.push(format!(
                "{}: oltre la profondità massima di {MAX_DEPTH} livelli, inserito nella radice",
                describe(&self.objects[object])
            ));
            self.too_deep.insert(object);
            root.children.push(self.subtree(object, 2));
        }
    }
}

/// Whether each object is the trash or descends from it through its
/// parents, computed once per object even on long chains.
fn trash_membership(objects: &[RawObject], parents: &[usize], root: usize) -> Vec<bool> {
    let mut known: Vec<Option<bool>> = vec![None; objects.len()];
    known[root] = Some(false);
    for start in 0..objects.len() {
        let mut path = Vec::new();
        let mut on_path = HashSet::new();
        let mut current = start;
        let in_trash = loop {
            if let Some(in_trash) = known[current] {
                break in_trash;
            }
            path.push(current);
            if objects[current].type_name == types::TRASH {
                break true;
            }
            if !on_path.insert(current) {
                break false;
            }
            current = parents[current];
        };
        for object in path {
            known[object] = Some(in_trash);
        }
    }
    known
        .into_iter()
        .map(|in_trash| in_trash == Some(true))
        .collect()
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

/// The first ancestor of `object` met twice when climbing the parents: a
/// member of the cycle that keeps `object` away from the root.
fn cycle_member(object: usize, parents: &[usize]) -> usize {
    // A set sized by the climb, not by the document: every object climbed
    // is attached right after, so the climbs stay linear overall.
    let mut seen = HashSet::new();
    let mut current = object;
    while seen.insert(current) {
        current = parents[current];
    }
    current
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
