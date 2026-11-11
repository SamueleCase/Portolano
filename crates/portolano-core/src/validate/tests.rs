use pretty_assertions::assert_eq;
use serde_json::json;

use super::*;
use crate::testutil::{connection, credential, document, folder, with_spec};

const ADMIN: Uuid = Uuid::from_u128(0xad);
const BASTION: Uuid = Uuid::from_u128(0xba);
const CLIENTS: Uuid = Uuid::from_u128(0xc1);
const WEB: Uuid = Uuid::from_u128(0x3e);
const DB: Uuid = Uuid::from_u128(0xdb);
const OTHER: Uuid = Uuid::from_u128(0xe0);
const MISSING: Uuid = Uuid::from_u128(0xf0);

fn by_id(id: Uuid) -> CredentialSpec {
    CredentialSpec::ById { id }
}

fn by_name(name: &str) -> CredentialSpec {
    CredentialSpec::ByName {
        name: name.to_owned(),
    }
}

fn with_jump(mut node: Node, jump: Value) -> Node {
    node.as_connection_mut()
        .unwrap()
        .settings
        .insert("jump_hosts".to_owned(), json!([jump]));
    node
}

/// A valid document: `Clients` uses `Admin ACME` by id, `web` jumps through
/// `bastion`, `db` inherits.
fn sample() -> Document {
    document(vec![
        credential(ADMIN, "Admin ACME"),
        connection(BASTION, "bastion"),
        with_spec(
            folder(
                CLIENTS,
                "Clients",
                vec![
                    with_jump(connection(WEB, "web"), json!({ "$ref": BASTION })),
                    connection(DB, "db"),
                ],
            ),
            by_id(ADMIN),
        ),
    ])
}

fn set_spec(document: &mut Document, id: Uuid, spec: CredentialSpec) {
    let node = document.find_mut(id).unwrap();
    *node = with_spec(node.clone(), spec);
}

fn codes(issues: &[Issue]) -> Vec<(&'static str, Option<Uuid>)> {
    issues
        .iter()
        .map(|issue| (issue.code, issue.node))
        .collect()
}

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

#[test]
fn validate_accepts_a_consistent_document() {
    let document = sample();

    assert_eq!(document.validate(), vec![]);
}

#[test]
fn validate_reports_duplicate_ids_once_with_severity_error() {
    let mut document = sample();
    document.nodes.push(connection(WEB, "copy"));

    let issues = document.validate();

    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].code, "duplicate_id");
    assert_eq!(issues[0].node, Some(WEB));
    assert_eq!(issues[0].severity, Severity::Error);
}

#[test]
fn validate_reports_a_dangling_ref_in_settings() {
    let mut document = sample();
    document.nodes.push(with_jump(
        connection(OTHER, "orphan"),
        json!({ "$ref": MISSING }),
    ));

    assert_eq!(
        codes(&document.validate()),
        vec![("dangling_ref", Some(OTHER))]
    );
}

#[test]
fn validate_reports_a_ref_that_is_not_an_id_as_dangling() {
    let mut document = sample();
    document.nodes.push(with_jump(
        connection(OTHER, "orphan"),
        json!({ "$ref": "not-an-id" }),
    ));

    assert_eq!(
        codes(&document.validate()),
        vec![("dangling_ref", Some(OTHER))]
    );
}

#[test]
fn validate_reports_a_dangling_credential_reference() {
    let mut document = sample();
    set_spec(&mut document, DB, by_id(MISSING));

    assert_eq!(
        codes(&document.validate()),
        vec![("dangling_ref", Some(DB))]
    );
}

#[test]
fn validate_reports_a_credential_reference_to_another_kind_of_node() {
    let mut document = sample();
    set_spec(&mut document, DB, by_id(BASTION));

    assert_eq!(
        codes(&document.validate()),
        vec![("credential_ref_not_credential", Some(DB))]
    );
}

#[test]
fn validate_reports_an_unresolved_by_name() {
    let mut document = sample();
    set_spec(&mut document, DB, by_name("Nobody"));

    assert_eq!(
        codes(&document.validate()),
        vec![("by_name_unresolved", Some(DB))]
    );
}

#[test]
fn validate_reports_an_ambiguous_by_name() {
    let mut document = sample();
    document
        .insert(None, None, credential(OTHER, "Admin ACME"))
        .unwrap();
    set_spec(&mut document, DB, by_name("Admin ACME"));

    assert_eq!(
        codes(&document.validate()),
        vec![("by_name_ambiguous", Some(DB))]
    );
}

#[test]
fn validate_reports_a_blank_host() {
    let mut document = sample();
    document
        .find_mut(DB)
        .unwrap()
        .as_connection_mut()
        .unwrap()
        .host = "  ".to_owned();

    assert_eq!(codes(&document.validate()), vec![("empty_host", Some(DB))]);
}
