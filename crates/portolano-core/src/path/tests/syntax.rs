//! Path form: escapes, leading slash, root and malformed paths.

use pretty_assertions::assert_eq;

use super::*;

#[test]
fn path_of_starts_with_a_slash_and_omits_the_document_name() {
    let document = sample();

    assert_eq!(
        document.path_of(WEB01).as_deref(),
        Some("/Clienti/ACME/web01")
    );
}

#[test]
fn path_of_escapes_slashes_and_backslashes() {
    let document = sample();

    assert_eq!(document.path_of(SLASHED).as_deref(), Some("/Clienti/a\\/b"));
    assert_eq!(
        document.path_of(BACKSLASHED).as_deref(),
        Some("/Clienti/back\\\\slash")
    );
}

#[test]
fn path_of_a_missing_node_is_none() {
    let document = sample();

    assert_eq!(document.path_of(MISSING), None);
}

#[test]
fn resolve_path_accepts_paths_with_and_without_leading_slash() {
    let document = sample();

    assert_eq!(document.resolve_path("/Clienti/ACME/web01").unwrap(), WEB01);
    assert_eq!(document.resolve_path("Clienti/ACME/web01").unwrap(), WEB01);
}

#[test]
fn resolve_path_unescapes_names() {
    let document = sample();

    assert_eq!(document.resolve_path("Clienti/a\\/b").unwrap(), SLASHED);
    assert_eq!(
        document.resolve_path("Clienti/back\\\\slash").unwrap(),
        BACKSLASHED
    );
}

#[test]
fn resolve_path_inverts_path_of() {
    let document = sample();

    // The two sibling folders named "Linux" share a path, so it is ambiguous.
    let unambiguous = document
        .walk()
        .map(|(_, node)| node.id)
        .filter(|id| ![LINUX_FIRST, LINUX_SECOND].contains(id));

    for id in unambiguous {
        let path = document.path_of(id).unwrap();
        assert_eq!(document.resolve_path(&path).unwrap(), id, "{path}");
    }
}

#[test]
fn resolve_path_of_the_root_is_rejected() {
    let document = sample();

    let reason = invalid_reason(document.resolve_path("/"));

    assert_eq!(reason, "la radice non è un nodo");
}

#[test]
fn resolve_path_rejects_a_trailing_slash() {
    let document = sample();

    invalid_reason(document.resolve_path("Clienti/"));
}

#[test]
fn resolve_path_rejects_an_empty_segment() {
    let document = sample();

    invalid_reason(document.resolve_path("Clienti//ACME"));
}

#[test]
fn resolve_path_rejects_an_unknown_escape() {
    let document = sample();

    invalid_reason(document.resolve_path("Clienti\\n"));
}

#[test]
fn resolve_path_rejects_a_trailing_backslash() {
    let document = sample();

    invalid_reason(document.resolve_path("Clienti\\"));
}

#[test]
fn resolve_path_rejects_an_empty_path() {
    let document = sample();

    invalid_reason(document.resolve_path(""));
}

#[test]
fn resolve_location_of_the_slash_is_the_root() {
    let document = sample();

    assert_eq!(document.resolve_location("/").unwrap(), None);
}

#[test]
fn resolve_parent_path_unescapes_the_last_name() {
    let document = sample();

    let resolved = document.resolve_parent_path("Clienti/x\\/y").unwrap();

    assert_eq!(resolved, (Some(CLIENTI), "x/y".to_owned()));
}

#[test]
fn resolve_parent_path_of_the_root_is_rejected() {
    let document = sample();

    invalid_reason(document.resolve_parent_path("/"));
}

#[test]
fn resolve_parent_path_ending_with_an_id_is_rejected() {
    let document = sample();

    invalid_reason(document.resolve_parent_path("Clienti/@abcdef01"));
}
