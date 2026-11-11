//! The maximum depth of the tree (DEC-021).

use pretty_assertions::assert_eq;

use super::*;
use crate::testutil::folder_chain;

const DEEP_FOLDER: Uuid = Uuid::from_u128(0xd0);
const DEEP_LEAF: Uuid = Uuid::from_u128(0xd1);

/// A folder holding one connection: a subtree two levels deep.
fn two_level_subtree() -> Node {
    folder(DEEP_FOLDER, "sub", vec![connection(DEEP_LEAF, "leaf")])
}

fn is_too_deep(result: Result<()>) -> bool {
    matches!(result, Err(Error::TooDeep { max: MAX_DEPTH }))
}

#[test]
fn insert_at_the_maximum_depth_is_accepted() {
    let (mut document, ids) = folder_chain(MAX_DEPTH - 1);

    let result = document.insert(ids.last().copied(), None, connection(NEW, "last"));

    assert!(result.is_ok());
}

#[test]
fn insert_below_the_maximum_depth_is_rejected_and_changes_nothing() {
    let (mut document, ids) = folder_chain(MAX_DEPTH);
    let before = document.clone();

    let result = document.insert(ids.last().copied(), None, connection(NEW, "too-deep"));

    assert!(is_too_deep(result));
    assert_eq!(document, before);
}

#[test]
fn insert_counts_the_depth_of_the_inserted_subtree() {
    let (mut document, ids) = folder_chain(MAX_DEPTH - 1);
    let before = document.clone();

    let result = document.insert(ids.last().copied(), None, two_level_subtree());

    assert!(is_too_deep(result));
    assert_eq!(document, before);
}

#[test]
fn move_counts_the_depth_of_the_moved_subtree_and_changes_nothing() {
    let (mut document, ids) = folder_chain(MAX_DEPTH - 1);
    document.insert(None, None, two_level_subtree()).unwrap();
    let before = document.clone();

    let result = document.move_node(DEEP_FOLDER, ids.last().copied(), None);

    assert!(is_too_deep(result));
    assert_eq!(document, before);
}

#[test]
fn move_to_the_maximum_depth_is_accepted() {
    let (mut document, ids) = folder_chain(MAX_DEPTH - 1);
    document
        .insert(None, None, connection(NEW, "moved"))
        .unwrap();

    let result = document.move_node(NEW, ids.last().copied(), None);

    assert!(result.is_ok());
}

#[test]
fn moving_a_deep_subtree_higher_is_accepted() {
    let (mut document, ids) = folder_chain(MAX_DEPTH);

    let result = document.move_node(ids[1], None, None);

    assert!(result.is_ok());
}
