//! Encrypted fields recognized by their value, or by a name that the
//! mapping does not know (D3, DEC-019.3, DEC-024.4).

use portolano_core::NodeKind;
use portolano_import::ItemStatus;
use pretty_assertions::assert_eq;

use crate::testutil::*;

/// Base64 of 48 bytes, block-aligned, under a name that says nothing about
/// a secret (DEC-024.4).
const UNNAMED_CIPHERTEXT: &str = "kq7Hn2Lw0zVb5yX1cR8tFj3mP6sD9gA4eK0uN7iB2oQ5vZ8xW1hT4rY6lC3dE9fG";

#[test]
fn ciphertext_under_an_unexpected_name_is_dropped_and_reported() {
    let xml = ssh_with(&[("TokenBlob", UNNAMED_CIPHERTEXT)]);

    let outcome = import(&xml);

    let node = find(&outcome.root, uuid(2));
    assert!(!node.extra.contains_key("royal.TokenBlob"));
    assert_eq!(status(&outcome, 2), ItemStatus::Partial);
    assert_eq!(
        reasons(&outcome, 2),
        vec!["campo cifrato `TokenBlob` non importato"]
    );
    let json = serde_json::to_string(&outcome.root).unwrap();
    assert!(!json.contains(UNNAMED_CIPHERTEXT));
}

#[test]
fn ciphertext_under_an_unexpected_name_is_removed_from_an_unsupported_object() {
    let xml = document(&[object(
        "RoyalVNCConnection",
        2,
        1,
        &[("Name", "vnc"), ("TokenBlob", UNNAMED_CIPHERTEXT)],
    )]);

    let outcome = import(&xml);

    let node = find(&outcome.root, uuid(2));
    let NodeKind::Unsupported(unsupported) = &node.kind else {
        panic!("not unsupported: {node:?}");
    };
    assert!(!unsupported.raw.contains(UNNAMED_CIPHERTEXT));
    assert!(unsupported.raw.contains("<Name>vnc</Name>"));
}

#[test]
fn base64_notes_and_images_are_not_taken_for_ciphertext() {
    let xml = ssh_with(&[("Notes", UNNAMED_CIPHERTEXT)]);

    let outcome = import(&xml);

    assert_eq!(status(&outcome, 2), ItemStatus::Imported);
    let node = find(&outcome.root, uuid(2));
    assert_eq!(node.notes.as_deref(), Some(UNNAMED_CIPHERTEXT));
}

#[test]
fn an_unknown_encrypted_field_is_dropped_and_reported() {
    let xml = credential_with(&[("SSHKeyPassword", FAKE_CIPHERTEXT)]);

    let outcome = import(&xml);

    let node = find(&outcome.root, uuid(2));
    assert!(!node.extra.contains_key("royal.SSHKeyPassword"));
    assert_eq!(
        reasons(&outcome, 2),
        vec!["campo cifrato `SSHKeyPassword` non importato"]
    );
}
