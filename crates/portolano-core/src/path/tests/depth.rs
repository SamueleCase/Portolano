//! `ensure_parent_path` and the maximum depth of the tree (DEC-021).

use pretty_assertions::assert_eq;

use super::*;
use crate::model::MAX_DEPTH;
use crate::testutil::folder_chain;

/// `Clienti` (level 1), then `count` new folders, then the node to add.
fn path_below_clienti(count: usize) -> String {
    let mut path = "Clienti/".to_owned();
    path.push_str(&"nuova/".repeat(count));
    path.push_str("web09");
    path
}

#[test]
fn ensure_parent_path_up_to_the_maximum_depth_creates_the_folders() {
    let mut document = sample();

    let (parent, _) = document
        .ensure_parent_path(&path_below_clienti(MAX_DEPTH - 2))
        .unwrap();

    assert_eq!(document.child_level(parent), MAX_DEPTH);
}

#[test]
fn ensure_parent_path_beyond_the_maximum_depth_is_rejected_and_creates_nothing() {
    let mut document = sample();
    let before = document.clone();

    let result = document.ensure_parent_path(&path_below_clienti(MAX_DEPTH - 1));

    assert!(matches!(result, Err(Error::TooDeep { max: MAX_DEPTH })));
    assert_eq!(document, before);
}

#[test]
fn ensure_parent_path_under_an_existing_folder_at_the_maximum_depth_is_rejected() {
    let (mut document, _) = folder_chain(MAX_DEPTH);
    let names: Vec<String> = (1..=MAX_DEPTH)
        .map(|level| format!("level-{level}"))
        .collect();

    let result = document.ensure_parent_path(&format!("{}/web09", names.join("/")));

    assert!(matches!(result, Err(Error::TooDeep { max: MAX_DEPTH })));
}
