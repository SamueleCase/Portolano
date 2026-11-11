//! Moving and reordering nodes (SPEC §10.2 `move`).

use portolano_core::{Document, Store};
use uuid::Uuid;

use crate::commit::Change;
use crate::nodes::check_name;
use crate::tree_util::{location_path, node_path};
use crate::{OpsError, Result};

/// Where a moved node goes among the children of its destination.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    /// After the last child.
    End,
    /// At this index of the final list, from 0; past the end means last.
    Index(usize),
    /// Right before this child of the destination.
    Before(Uuid),
    /// Right after this child of the destination.
    After(Uuid),
}

/// Moves node `id` under `destination` (`None` = root) and saves the
/// document; with the current parent as destination it reorders. Returns the
/// new path of the node.
///
/// # Errors
/// [`portolano_core::Error::InvalidMove`] into itself or a descendant,
/// [`portolano_core::Error::NotAFolder`],
/// [`portolano_core::Error::TooDeep`], [`OpsError::NameTaken`] if another
/// child of the destination has the same name, [`OpsError::NotASibling`]
/// for a `Before`/`After` node that is not a child of the destination,
/// errors of the store.
pub fn move_node(
    store: &Store,
    doc: &mut Document,
    id: Uuid,
    destination: Option<Uuid>,
    placement: Placement,
) -> Result<String> {
    let node = doc
        .find(id)
        .ok_or(portolano_core::Error::NodeNotFound { id })?;
    check_name(doc, destination, &node.name, Some(id))?;
    let position = position(doc, id, destination, placement)?;
    let mut change = Change::from(doc);
    change.doc.move_node(id, destination, position)?;
    change.save_without_secrets(store, doc)?;
    Ok(node_path(doc, id))
}

/// The index in the final list of children, which does not contain the
/// moved node any more.
fn position(
    doc: &Document,
    id: Uuid,
    destination: Option<Uuid>,
    placement: Placement,
) -> Result<Option<usize>> {
    let (sibling, offset) = match placement {
        Placement::End => return Ok(None),
        Placement::Index(index) => return Ok(Some(index)),
        Placement::Before(sibling) => (sibling, 0),
        Placement::After(sibling) => (sibling, 1),
    };
    doc.children(destination)?
        .iter()
        .filter(|child| child.id != id)
        .position(|child| child.id == sibling)
        .map(|index| Some(index + offset))
        .ok_or_else(|| OpsError::NotASibling {
            path: node_path(doc, sibling),
            destination: location_path(doc, destination),
        })
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::testutil::*;

    fn names(doc: &Document, parent: Option<Uuid>) -> Vec<String> {
        doc.children(parent)
            .unwrap()
            .iter()
            .map(|node| node.name.clone())
            .collect()
    }

    fn three() -> Stored {
        stored(vec![
            connection("a", "ssh"),
            connection("b", "ssh"),
            connection("c", "ssh"),
            folder("F", vec![]),
        ])
    }

    fn reorder(stored: &mut Stored, path: &str, placement: Placement) -> Result<String> {
        let id = id_of(&stored.doc, path);
        move_node(&stored.store, &mut stored.doc, id, None, placement)
    }

    #[test]
    fn move_into_a_folder_returns_the_new_path_and_saves() {
        let mut stored = three();
        let id = id_of(&stored.doc, "/a");
        let folder = id_of(&stored.doc, "/F");
        let path = move_node(
            &stored.store,
            &mut stored.doc,
            id,
            Some(folder),
            Placement::End,
        )
        .unwrap();
        assert_eq!(path, "/F/a");
        assert_eq!(names(&stored.reloaded(), Some(folder)), vec!["a"]);
    }

    #[test]
    fn before_places_the_node_before_its_sibling() {
        let mut stored = three();
        let c = id_of(&stored.doc, "/c");
        reorder(&mut stored, "/a", Placement::Before(c)).unwrap();
        assert_eq!(names(&stored.doc, None), vec!["b", "a", "c", "F"]);
    }

    #[test]
    fn after_places_the_node_after_its_sibling() {
        let mut stored = three();
        let a = id_of(&stored.doc, "/a");
        reorder(&mut stored, "/c", Placement::After(a)).unwrap();
        assert_eq!(names(&stored.doc, None), vec!["a", "c", "b", "F"]);
    }

    #[test]
    fn index_is_the_position_in_the_final_list() {
        let mut stored = three();
        reorder(&mut stored, "/F", Placement::Index(0)).unwrap();
        assert_eq!(names(&stored.doc, None), vec!["F", "a", "b", "c"]);
    }

    #[test]
    fn before_a_node_of_another_folder_is_refused() {
        let mut stored = stored(vec![
            connection("a", "ssh"),
            folder("F", vec![connection("x", "ssh")]),
        ]);
        let x = id_of(&stored.doc, "/F/x");
        let result = reorder(&mut stored, "/a", Placement::Before(x));
        assert!(matches!(result, Err(OpsError::NotASibling { .. })));
    }

    #[test]
    fn move_next_to_a_node_with_the_same_name_is_refused() {
        let mut stored = stored(vec![
            connection("web01", "ssh"),
            folder("F", vec![connection("web01", "ssh")]),
        ]);
        let id = id_of(&stored.doc, "/F/web01");
        let result = move_node(&stored.store, &mut stored.doc, id, None, Placement::End);
        assert!(matches!(result, Err(OpsError::NameTaken { .. })));
    }

    #[test]
    fn move_into_a_descendant_is_refused() {
        let mut stored = stored(vec![folder("F", vec![folder("G", vec![])])]);
        let f = id_of(&stored.doc, "/F");
        let g = id_of(&stored.doc, "/F/G");
        let result = move_node(&stored.store, &mut stored.doc, f, Some(g), Placement::End);
        assert!(matches!(
            result,
            Err(OpsError::Core(portolano_core::Error::InvalidMove { .. }))
        ));
    }
}
