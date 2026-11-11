//! `references_to` and `detach_credential_references`.

use pretty_assertions::assert_eq;
use serde_json::json;

use super::*;

#[test]
fn references_to_finds_a_ref_nested_in_settings() {
    let document = sample();

    assert_eq!(document.references_to(BASTION), vec![WEB]);
}

#[test]
fn references_to_finds_a_by_id_credential_spec_but_not_its_inheritors() {
    let document = sample();

    assert_eq!(document.references_to(ADMIN), vec![CLIENTS]);
}

#[test]
fn references_to_does_not_credit_a_folder_with_its_childrens_references() {
    let document = sample();

    let referrers = document.references_to(BASTION);

    assert!(!referrers.contains(&CLIENTS));
}

#[test]
fn references_to_counts_by_name_specs_that_resolve_to_the_credential() {
    let mut document = sample();
    set_spec(&mut document, DB, by_name("admin acme"));

    assert_eq!(document.references_to(ADMIN), vec![CLIENTS, DB]);
}

#[test]
fn references_to_ignores_by_name_specs_that_resolve_elsewhere() {
    let mut document = sample();
    document
        .insert(None, None, credential(OTHER, "admin acme"))
        .unwrap();
    set_spec(&mut document, DB, by_name("admin acme"));

    assert_eq!(document.references_to(ADMIN), vec![CLIENTS]);
    assert_eq!(document.references_to(OTHER), vec![DB]);
}

fn spec_of(document: &Document, id: Uuid) -> CredentialSpec {
    credential_spec(document.find(id).unwrap()).unwrap().clone()
}

#[test]
fn references_to_counts_by_name_specs_inside_settings() {
    let mut document = sample();
    document.nodes.push(with_setting(
        connection(OTHER, "rdp"),
        "gateway",
        json!({ "credential": { "mode": "by_name", "name": "Admin ACME" } }),
    ));

    assert_eq!(document.references_to(ADMIN), vec![CLIENTS, OTHER]);
}

#[test]
fn detach_turns_a_by_id_spec_into_prompt_and_touches_the_node() {
    let mut document = sample();
    let long_ago = "2020-01-01T00:00:00Z".parse().unwrap();
    document.find_mut(CLIENTS).unwrap().modified = long_ago;

    let detached = document.detach_credential_references(ADMIN);

    assert_eq!(detached, vec![CLIENTS]);
    assert_eq!(spec_of(&document, CLIENTS), CredentialSpec::Prompt);
    assert!(document.find(CLIENTS).unwrap().modified > long_ago);
}

#[test]
fn detach_turns_a_by_name_spec_that_resolves_to_the_credential_into_prompt() {
    let mut document = sample();
    set_spec(&mut document, DB, by_name("admin acme"));

    let detached = document.detach_credential_references(ADMIN);

    assert_eq!(detached, vec![CLIENTS, DB]);
    assert_eq!(spec_of(&document, DB), CredentialSpec::Prompt);
}

#[test]
fn detach_leaves_by_name_specs_that_resolve_elsewhere() {
    let mut document = sample();
    document
        .insert(None, None, credential(OTHER, "admin acme"))
        .unwrap();
    set_spec(&mut document, DB, by_name("admin acme"));

    let detached = document.detach_credential_references(ADMIN);

    assert_eq!(detached, vec![CLIENTS]);
    assert_eq!(spec_of(&document, DB), by_name("admin acme"));
}

#[test]
fn detach_replaces_specs_inside_settings_and_keeps_the_other_settings() {
    let mut document = sample();
    let gateway = with_setting(
        connection(OTHER, "rdp"),
        "gateway",
        json!({
            "host": "gateway.example.com",
            "credential": { "mode": "by_id", "$ref": ADMIN },
        }),
    );
    document.nodes.push(gateway);

    let detached = document.detach_credential_references(ADMIN);

    assert_eq!(detached, vec![CLIENTS, OTHER]);
    let settings = &document
        .find(OTHER)
        .unwrap()
        .as_connection()
        .unwrap()
        .settings;
    assert_eq!(
        settings["gateway"],
        json!({ "host": "gateway.example.com", "credential": { "mode": "prompt" } })
    );
}

#[test]
fn detach_without_references_changes_nothing() {
    let mut document = sample();
    document.nodes.push(with_setting(
        connection(OTHER, "rdp"),
        "resolution",
        json!({ "mode": "fit" }),
    ));
    let before = document.clone();

    let detached = document.detach_credential_references(BASTION);

    assert!(detached.is_empty());
    assert_eq!(document, before);
}
