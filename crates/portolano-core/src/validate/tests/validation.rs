//! The issues reported by `validate`.

use pretty_assertions::assert_eq;
use serde_json::json;

use super::*;
use crate::model::Unsupported;
use crate::secret::SecretRef;

fn codes(issues: &[Issue]) -> Vec<(&'static str, Option<Uuid>)> {
    issues
        .iter()
        .map(|issue| (issue.code, issue.node))
        .collect()
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

#[test]
fn validate_reports_an_unsupported_node_as_info_with_its_source_type() {
    let mut legacy = connection(OTHER, "vnc-legacy");
    legacy.kind = NodeKind::Unsupported(Unsupported {
        source_type: "RoyalVNCConnection".to_owned(),
        raw: "<RoyalVNCConnection />".to_owned(),
    });
    let mut document = sample();
    document.nodes.push(legacy);

    let issues = document.validate();

    assert_eq!(codes(&issues), vec![("unsupported_node", Some(OTHER))]);
    assert_eq!(issues[0].severity, Severity::Info);
    assert!(issues[0].message.contains("RoyalVNCConnection"));
}

fn credential_with_password(id: Uuid, name: &str, password: &SecretRef) -> Node {
    let mut node = credential(id, name);
    node.as_credential_mut().unwrap().password = Some(password.clone());
    node
}

#[test]
fn validate_reports_a_secret_ref_shared_by_two_nodes() {
    let shared = SecretRef::new();
    let mut document = sample();
    document
        .nodes
        .push(credential_with_password(OTHER, "first", &shared));
    document
        .nodes
        .push(credential_with_password(SECOND, "second", &shared));

    let issues = document.validate();

    assert_eq!(codes(&issues), vec![("duplicate_secret_ref", Some(OTHER))]);
    assert!(issues[0].message.contains(shared.as_str()));
    assert!(issues[0].message.contains("/second"));
}

#[test]
fn validate_reports_a_secret_ref_used_twice_in_the_same_node() {
    let shared = SecretRef::new();
    let mut node = credential_with_password(OTHER, "twice", &shared);
    node.as_credential_mut().unwrap().passphrase = Some(shared);
    let mut document = sample();
    document.nodes.push(node);

    assert_eq!(
        codes(&document.validate()),
        vec![("duplicate_secret_ref", Some(OTHER))]
    );
}

#[test]
fn validate_reports_a_secret_ref_repeated_inside_settings() {
    let shared = SecretRef::new();
    let mut node = with_spec(
        connection(OTHER, "rdp"),
        CredentialSpec::Inline {
            username: None,
            domain: None,
            password: Some(shared.clone()),
        },
    );
    node.as_connection_mut().unwrap().settings.insert(
        "gateway".to_owned(),
        json!({ "credential": { "mode": "inline", "password": shared } }),
    );
    let mut document = sample();
    document.nodes.push(node);

    assert_eq!(
        codes(&document.validate()),
        vec![("duplicate_secret_ref", Some(OTHER))]
    );
}
