//! Path lookup: ids, names, ambiguity, case and parent paths.

use pretty_assertions::assert_eq;

use super::*;

#[test]
fn resolve_path_by_full_uuid() {
    let document = sample();

    let resolved = document.resolve_path(&format!("@{WEB01}")).unwrap();

    assert_eq!(resolved, WEB01);
}

#[test]
fn resolve_path_by_id_prefix() {
    let document = sample();

    assert_eq!(document.resolve_path("@abcdef01").unwrap(), WEB01);
}

#[test]
fn resolve_path_by_id_prefix_ignores_hex_case() {
    let document = sample();

    assert_eq!(document.resolve_path("@ABCDEF01").unwrap(), WEB01);
}

#[test]
fn resolve_path_continues_with_names_after_an_id() {
    let document = sample();

    assert_eq!(document.resolve_path("@acacac/web02").unwrap(), WEB02);
}

#[test]
fn resolve_path_id_after_names_searches_descendants_at_any_depth() {
    let document = sample();

    assert_eq!(document.resolve_path("Clienti/@abcdef02").unwrap(), WEB02);
}

#[test]
fn resolve_path_id_after_names_ignores_nodes_outside_the_subtree() {
    let document = sample();

    let result = document.resolve_path("Prod/@abcdef02");

    assert!(matches!(result, Err(Error::NotFound { query }) if query == "Prod/@abcdef02"));
}

#[test]
fn resolve_path_ambiguous_id_prefix_lists_the_candidates() {
    let document = sample();

    let result = document.resolve_path("@abcdef");

    let Err(Error::Ambiguous { query, candidates }) = result else {
        panic!("expected Ambiguous, got {result:?}");
    };
    assert_eq!(query, "@abcdef");
    assert_eq!(
        candidates,
        vec![
            (WEB01, "/Clienti/ACME/web01".to_owned()),
            (WEB02, "/Clienti/ACME/web02".to_owned())
        ]
    );
}

#[test]
fn resolve_path_short_id_prefix_is_a_name() {
    let mut document = sample();
    document
        .insert(None, None, connection(NAMED_LIKE_AN_ID, "@abcde"))
        .unwrap();

    assert_eq!(document.resolve_path("@abcde").unwrap(), NAMED_LIKE_AN_ID);
}

#[test]
fn resolve_path_ambiguous_names_list_the_candidates() {
    let document = sample();

    let result = document.resolve_path("Linux");

    let Err(Error::Ambiguous { candidates, .. }) = result else {
        panic!("expected Ambiguous, got {result:?}");
    };
    assert_eq!(
        candidates,
        vec![
            (LINUX_FIRST, "/Linux".to_owned()),
            (LINUX_SECOND, "/Linux".to_owned())
        ]
    );
}

#[test]
fn resolve_path_through_duplicate_folder_names_finds_the_only_match() {
    let document = sample();

    assert_eq!(
        document.resolve_path("Linux/only-in-second").unwrap(),
        ONLY_IN_SECOND
    );
}

#[test]
fn resolve_path_falls_back_to_case_insensitive_names() {
    let document = sample();

    assert_eq!(document.resolve_path("clienti/acme/WEB01").unwrap(), WEB01);
}

#[test]
fn resolve_path_prefers_an_exact_match_over_case_insensitive_ones() {
    let document = sample();

    assert_eq!(document.resolve_path("Prod/db").unwrap(), DB_UPPER);
    assert_eq!(document.resolve_path("prod/db").unwrap(), DB_LOWER);
}

#[test]
fn resolve_path_case_insensitive_fallback_can_be_ambiguous() {
    let document = sample();

    let result = document.resolve_path("PROD/db");

    let Err(Error::Ambiguous { candidates, .. }) = result else {
        panic!("expected Ambiguous, got {result:?}");
    };
    assert_eq!(
        candidates.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
        vec![DB_UPPER, DB_LOWER]
    );
}

#[test]
fn resolve_path_unknown_name_is_not_found() {
    let document = sample();

    let result = document.resolve_path("Clienti/nope");

    assert!(matches!(result, Err(Error::NotFound { query }) if query == "Clienti/nope"));
}

#[test]
fn resolve_location_of_a_node_is_its_id() {
    let document = sample();

    assert_eq!(
        document.resolve_location("/Clienti").unwrap(),
        Some(CLIENTI)
    );
}

#[test]
fn resolve_parent_path_splits_parent_and_name() {
    let document = sample();

    let resolved = document.resolve_parent_path("Clienti/ACME/web03").unwrap();

    assert_eq!(resolved, (Some(ACME), "web03".to_owned()));
}

#[test]
fn resolve_parent_path_of_a_top_level_name_is_the_root() {
    let document = sample();

    let resolved = document.resolve_parent_path("/new").unwrap();

    assert_eq!(resolved, (None, "new".to_owned()));
}

#[test]
fn resolve_parent_path_under_a_connection_is_rejected() {
    let document = sample();

    let result = document.resolve_parent_path("Clienti/ACME/web01/x");

    assert!(matches!(result, Err(Error::NotAFolder { id }) if id == WEB01));
}

#[test]
fn resolve_parent_path_with_a_missing_parent_is_not_found() {
    let document = sample();

    let result = document.resolve_parent_path("Clienti/Nuovo/x");

    assert!(matches!(result, Err(Error::NotFound { query }) if query == "/Clienti/Nuovo"));
}

#[test]
fn ensure_parent_path_creates_the_missing_folders() {
    let mut document = sample();

    let (parent, name) = document
        .ensure_parent_path("Clienti/Nuovo/Interno/web09")
        .unwrap();

    let parent = parent.unwrap();
    assert_eq!(name, "web09");
    assert_eq!(
        document.path_of(parent).as_deref(),
        Some("/Clienti/Nuovo/Interno")
    );
    assert!(document.find(parent).unwrap().is_folder());
}

#[test]
fn ensure_parent_path_with_an_existing_parent_creates_nothing() {
    let mut document = sample();
    let before = document.clone();

    let resolved = document.ensure_parent_path("Clienti/ACME/web03").unwrap();

    assert_eq!(resolved, (Some(ACME), "web03".to_owned()));
    assert_eq!(document, before);
}

#[test]
fn ensure_parent_path_creates_below_a_case_insensitive_match() {
    let mut document = sample();

    let (parent, _) = document.ensure_parent_path("clienti/Nuovo/x").unwrap();

    assert_eq!(document.parent_of(parent.unwrap()), Some(Some(CLIENTI)));
}

#[test]
fn ensure_parent_path_under_a_connection_is_rejected_and_creates_nothing() {
    let mut document = sample();
    let before = document.clone();

    let result = document.ensure_parent_path("Clienti/ACME/web01/sub/x");

    assert!(matches!(result, Err(Error::NotAFolder { id }) if id == WEB01));
    assert_eq!(document, before);
}

#[test]
fn ensure_parent_path_with_a_missing_id_is_rejected_and_creates_nothing() {
    let mut document = sample();
    let before = document.clone();

    let result = document.ensure_parent_path("Clienti/Nuovo/@fedcba/x");

    assert!(matches!(result, Err(Error::NotFound { query }) if query == "/Clienti/Nuovo/@fedcba"));
    assert_eq!(document, before);
}

#[test]
fn ensure_parent_path_with_an_ambiguous_parent_is_rejected_and_creates_nothing() {
    let mut document = sample();
    let before = document.clone();

    let result = document.ensure_parent_path("Linux/x");

    assert!(matches!(result, Err(Error::Ambiguous { .. })));
    assert_eq!(document, before);
}
