use pretty_assertions::assert_eq;
use serde_json::json;

use super::*;
use crate::model::{CustomProperty, PropertyValue};
use crate::testutil::{connection, credential, document, folder, ids, with_spec};

mod depth;

const A: Uuid = Uuid::from_u128(0xa0);
const A1: Uuid = Uuid::from_u128(0xa1);
const A2: Uuid = Uuid::from_u128(0xa2);
const A3: Uuid = Uuid::from_u128(0xa3);
const B: Uuid = Uuid::from_u128(0xb0);
const B1: Uuid = Uuid::from_u128(0xb1);
const B11: Uuid = Uuid::from_u128(0xb11);
const C: Uuid = Uuid::from_u128(0xc0);
const NEW: Uuid = Uuid::from_u128(0xe0);
const MISSING: Uuid = Uuid::from_u128(0xf0);

/// `A{a1, a2, a3}`, `B{B1{b11}}`, `c` at the root.
fn sample() -> Document {
    document(vec![
        folder(
            A,
            "A",
            vec![
                connection(A1, "a1"),
                connection(A2, "a2"),
                connection(A3, "a3"),
            ],
        ),
        folder(B, "B", vec![folder(B1, "B1", vec![connection(B11, "b11")])]),
        connection(C, "c"),
    ])
}

fn child_ids(document: &Document, parent: Option<Uuid>) -> Vec<Uuid> {
    ids(document.children(parent).unwrap())
}

#[test]
fn find_returns_a_nested_node() {
    let document = sample();

    let found = document.find(B11).map(|node| node.name.as_str());

    assert_eq!(found, Some("b11"));
}

#[test]
fn find_mut_returns_a_nested_node() {
    let mut document = sample();

    document.find_mut(B11).unwrap().name = "renamed".to_owned();

    assert_eq!(document.find(B11).unwrap().name, "renamed");
}

#[test]
fn find_missing_node_returns_none() {
    let document = sample();

    assert!(document.find(MISSING).is_none());
}

#[test]
fn parent_of_a_root_child_is_the_root() {
    let document = sample();

    assert_eq!(document.parent_of(C), Some(None));
}

#[test]
fn parent_of_a_nested_node_is_its_folder() {
    let document = sample();

    assert_eq!(document.parent_of(B11), Some(Some(B1)));
}

#[test]
fn parent_of_a_missing_node_is_none() {
    let document = sample();

    assert_eq!(document.parent_of(MISSING), None);
}

#[test]
fn ancestors_go_from_the_parent_to_the_outermost_folder() {
    let document = sample();

    assert_eq!(document.ancestors(B11), vec![B1, B]);
}

#[test]
fn children_of_the_root_are_the_top_level_nodes() {
    let document = sample();

    assert_eq!(child_ids(&document, None), vec![A, B, C]);
}

#[test]
fn children_of_a_connection_are_rejected() {
    let document = sample();

    let result = document.children(Some(C));

    assert!(matches!(result, Err(Error::NotAFolder { id }) if id == C));
}

#[test]
fn children_of_a_missing_node_are_rejected() {
    let document = sample();

    let result = document.children(Some(MISSING));

    assert!(matches!(result, Err(Error::NodeNotFound { id }) if id == MISSING));
}

#[test]
fn walk_is_pre_order_with_depth() {
    let document = sample();

    let visited: Vec<(usize, Uuid)> = document
        .walk()
        .map(|(depth, node)| (depth, node.id))
        .collect();

    assert_eq!(
        visited,
        vec![
            (0, A),
            (1, A1),
            (1, A2),
            (1, A3),
            (0, B),
            (1, B1),
            (2, B11),
            (0, C)
        ]
    );
}

#[test]
fn insert_without_position_appends() {
    let mut document = sample();

    document
        .insert(Some(A), None, connection(NEW, "new"))
        .unwrap();

    assert_eq!(child_ids(&document, Some(A)), vec![A1, A2, A3, NEW]);
}

#[test]
fn insert_at_a_position_places_the_node_there() {
    let mut document = sample();

    document
        .insert(Some(A), Some(1), connection(NEW, "new"))
        .unwrap();

    assert_eq!(child_ids(&document, Some(A)), vec![A1, NEW, A2, A3]);
}

#[test]
fn insert_past_the_end_appends() {
    let mut document = sample();

    document
        .insert(None, Some(99), connection(NEW, "new"))
        .unwrap();

    assert_eq!(child_ids(&document, None), vec![A, B, C, NEW]);
}

#[test]
fn insert_under_a_connection_is_rejected() {
    let mut document = sample();

    let result = document.insert(Some(C), None, connection(NEW, "new"));

    assert!(matches!(result, Err(Error::NotAFolder { id }) if id == C));
}

#[test]
fn insert_under_a_missing_parent_is_rejected() {
    let mut document = sample();

    let result = document.insert(Some(MISSING), None, connection(NEW, "new"));

    assert!(matches!(result, Err(Error::NodeNotFound { id }) if id == MISSING));
}

#[test]
fn insert_of_an_id_already_in_the_document_is_rejected() {
    let mut document = sample();

    let result = document.insert(None, None, connection(B11, "copy"));

    assert!(matches!(result, Err(Error::DuplicateId { id }) if id == B11));
    assert_eq!(child_ids(&document, None), vec![A, B, C]);
}

#[test]
fn insert_of_a_subtree_holding_an_existing_id_is_rejected() {
    let mut document = sample();
    let subtree = folder(NEW, "new", vec![connection(A2, "copy")]);

    let result = document.insert(None, None, subtree);

    assert!(matches!(result, Err(Error::DuplicateId { id }) if id == A2));
}

#[test]
fn insert_of_a_subtree_repeating_its_own_id_is_rejected() {
    let mut document = sample();
    let subtree = folder(NEW, "new", vec![connection(NEW, "same id")]);

    let result = document.insert(None, None, subtree);

    assert!(matches!(result, Err(Error::DuplicateId { id }) if id == NEW));
}

#[test]
fn remove_detaches_the_whole_subtree() {
    let mut document = sample();

    let removed = document.remove(B).unwrap();

    assert_eq!(removed.id, B);
    assert_eq!(child_ids(&document, None), vec![A, C]);
    assert!(document.find(B11).is_none());
}

#[test]
fn remove_of_a_missing_node_is_rejected() {
    let mut document = sample();

    let result = document.remove(MISSING);

    assert!(matches!(result, Err(Error::NodeNotFound { id }) if id == MISSING));
}

#[test]
fn move_forward_within_the_same_parent_uses_the_final_index() {
    let mut document = sample();

    document.move_node(A1, Some(A), Some(2)).unwrap();

    assert_eq!(child_ids(&document, Some(A)), vec![A2, A3, A1]);
}

#[test]
fn move_backward_within_the_same_parent_uses_the_final_index() {
    let mut document = sample();

    document.move_node(A3, Some(A), Some(0)).unwrap();

    assert_eq!(child_ids(&document, Some(A)), vec![A3, A1, A2]);
}

#[test]
fn move_to_another_parent_detaches_from_the_old_one() {
    let mut document = sample();

    document.move_node(A2, Some(B1), Some(0)).unwrap();

    assert_eq!(child_ids(&document, Some(A)), vec![A1, A3]);
    assert_eq!(child_ids(&document, Some(B1)), vec![A2, B11]);
}

#[test]
fn move_to_the_root_appends_by_default() {
    let mut document = sample();

    document.move_node(B11, None, None).unwrap();

    assert_eq!(child_ids(&document, None), vec![A, B, C, B11]);
    assert!(child_ids(&document, Some(B1)).is_empty());
}

#[test]
fn move_to_the_root_at_an_explicit_position() {
    let mut document = sample();

    document.move_node(A2, None, Some(1)).unwrap();

    assert_eq!(child_ids(&document, None), vec![A, A2, B, C]);
}

#[test]
fn move_of_a_missing_node_is_rejected() {
    let mut document = sample();

    let result = document.move_node(MISSING, None, None);

    assert!(matches!(result, Err(Error::NodeNotFound { id }) if id == MISSING));
}

#[test]
fn move_under_a_missing_parent_is_rejected_and_leaves_the_tree_unchanged() {
    let mut document = sample();
    let before = document.clone();

    let result = document.move_node(A1, Some(MISSING), None);

    assert!(matches!(result, Err(Error::NodeNotFound { id }) if id == MISSING));
    assert_eq!(document, before);
}

#[test]
fn move_into_itself_is_rejected() {
    let mut document = sample();

    let result = document.move_node(B, Some(B), None);

    assert!(
        matches!(result, Err(Error::InvalidMove { id, new_parent }) if id == B && new_parent == B)
    );
}

#[test]
fn move_into_own_descendant_is_rejected_and_leaves_the_tree_unchanged() {
    let mut document = sample();
    let before = document.clone();

    let result = document.move_node(B, Some(B1), None);

    assert!(
        matches!(result, Err(Error::InvalidMove { id, new_parent }) if id == B && new_parent == B1)
    );
    assert_eq!(document, before);
}

#[test]
fn move_under_a_connection_is_rejected_and_leaves_the_tree_unchanged() {
    let mut document = sample();
    let before = document.clone();

    let result = document.move_node(A1, Some(C), None);

    assert!(matches!(result, Err(Error::NotAFolder { id }) if id == C));
    assert_eq!(document, before);
}

#[test]
fn move_updates_modified_of_the_moved_node() {
    let mut document = sample();
    let long_ago = "2020-01-01T00:00:00Z".parse().unwrap();
    document.find_mut(A1).unwrap().modified = long_ago;

    document.move_node(A1, None, None).unwrap();

    assert!(document.find(A1).unwrap().modified > long_ago);
}

#[test]
fn secret_refs_in_collects_credentials_inline_specs_and_properties() {
    let password = SecretRef::new();
    let key = SecretRef::new();
    let passphrase = SecretRef::new();
    let inline = SecretRef::new();
    let property = SecretRef::new();
    let mut stored = credential(A1, "stored");
    let fields = stored.as_credential_mut().unwrap();
    fields.password = Some(password.clone());
    fields.private_key = Some(PrivateKey::Vault {
        secret: key.clone(),
    });
    fields.passphrase = Some(passphrase.clone());
    let mut server = with_spec(
        connection(A2, "server"),
        CredentialSpec::Inline {
            username: Some("operator".to_owned()),
            domain: None,
            password: Some(inline.clone()),
        },
    );
    server.custom_properties.push(CustomProperty {
        name: "token".to_owned(),
        value: PropertyValue::Secret {
            secret: property.clone(),
        },
    });
    let document = document(vec![folder(A, "A", vec![stored, server])]);

    let refs = document.secret_refs_in(A);

    assert_eq!(refs, vec![password, key, passphrase, property, inline]);
}

#[test]
fn secret_refs_in_finds_inline_specs_inside_settings() {
    let gateway_password = SecretRef::new();
    let mut server = connection(A1, "server");
    server.as_connection_mut().unwrap().settings.insert(
        "gateway".to_owned(),
        json!({
            "host": "gateway.example.com",
            "credential": { "mode": "inline", "username": "operator", "password": gateway_password },
        }),
    );
    let document = document(vec![server]);

    let refs = document.secret_refs_in(A1);

    assert_eq!(refs, vec![gateway_password]);
}

#[test]
fn secret_refs_in_a_missing_node_is_empty() {
    let document = sample();

    assert!(document.secret_refs_in(MISSING).is_empty());
}
