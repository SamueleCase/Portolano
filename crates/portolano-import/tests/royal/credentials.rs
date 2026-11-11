//! Credentials of connections and folders, `RoyalCredential` and secrets
//! (SPEC §8.3.5).

use portolano_core::{CredentialSpec, PrivateKey};
use portolano_import::{ImportError, ItemStatus, SecretField, SecretPrompter, SecretRequest};
use pretty_assertions::assert_eq;

use crate::testutil::*;

const SKIPPED_PASSWORD: &str =
    "password Royal cifrata non importata: impostala con `portolano secret set`";

fn spec_of(xml: &str) -> CredentialSpec {
    connection(&import(xml).root, 2).credential.clone()
}

#[test]
fn absent_mode_inherits() {
    assert_eq!(spec_of(&ssh_with(&[])), CredentialSpec::Inherit);
}

#[test]
fn credential_from_parent_inherits_whatever_the_mode() {
    let xml = ssh_with(&[("CredentialFromParent", "True"), ("CredentialMode", "1")]);

    assert_eq!(spec_of(&xml), CredentialSpec::Inherit);
}

#[test]
fn mode_0_is_none_and_mode_1_is_prompt() {
    assert_eq!(
        spec_of(&ssh_with(&[("CredentialMode", "0")])),
        CredentialSpec::None
    );
    assert_eq!(
        spec_of(&ssh_with(&[("CredentialMode", "1")])),
        CredentialSpec::Prompt
    );
}

#[test]
fn mode_2_is_inline_with_domain_split() {
    let xml = ssh_with(&[("CredentialMode", "2"), ("CredentialUsername", "ACME\\ops")]);

    let expected = CredentialSpec::Inline {
        username: Some("ops".to_owned()),
        domain: Some("ACME".to_owned()),
        password: None,
    };
    assert_eq!(spec_of(&xml), expected);
}

#[test]
fn mode_3_references_the_credential_by_id() {
    let xml = document(&[
        object("RoyalCredential", 3, 1, &[("Name", "admin")]),
        object(
            "RoyalSSHConnection",
            2,
            1,
            &[
                ("URI", "web01.example.com"),
                ("CredentialMode", "3"),
                ("CredentialId", &id(3)),
            ],
        ),
    ]);

    let outcome = import(&xml);

    let spec = &connection(&outcome.root, 2).credential;
    assert_eq!(spec, &CredentialSpec::ById { id: uuid(3) });
    assert_eq!(status(&outcome, 2), ItemStatus::Imported);
}

#[test]
fn mode_3_to_a_credential_outside_the_document_is_partial() {
    let xml = ssh_with(&[("CredentialMode", "3"), ("CredentialId", &id(77))]);

    let outcome = import(&xml);

    assert_eq!(
        connection(&outcome.root, 2).credential,
        CredentialSpec::ById { id: uuid(77) }
    );
    assert_eq!(status(&outcome, 2), ItemStatus::Partial);
    assert_eq!(
        reasons(&outcome, 2),
        vec!["riferimento a credenziale esterna al documento"]
    );
}

#[test]
fn mode_3_to_a_credential_in_the_trash_is_partial() {
    let xml = document(&[
        object("RoyalTrash", 4, 1, &[]),
        object("RoyalCredential", 3, 4, &[("Name", "buttata")]),
        object(
            "RoyalSSHConnection",
            2,
            1,
            &[
                ("URI", "web01.example.com"),
                ("CredentialMode", "3"),
                ("CredentialId", &id(3)),
            ],
        ),
    ]);

    let outcome = import(&xml);

    assert_eq!(
        reasons(&outcome, 2),
        vec!["riferimento a una credenziale non importata"]
    );
}

#[test]
fn mode_4_references_the_credential_by_name() {
    let xml = ssh_with(&[
        ("CredentialMode", "4"),
        ("CredentialName", "Deploy"),
        ("CredentialId", &id(9)),
    ]);

    let outcome = import(&xml);

    let node = find(&outcome.root, uuid(2));
    assert_eq!(
        node.as_connection().unwrap().credential,
        CredentialSpec::ByName {
            name: "Deploy".to_owned()
        }
    );
    assert_eq!(node.extra["royal.CredentialId"], id(9).as_str());
}

#[test]
fn an_unknown_mode_inherits_and_is_partial() {
    let outcome = import(&ssh_with(&[("CredentialMode", "7")]));

    assert_eq!(
        connection(&outcome.root, 2).credential,
        CredentialSpec::Inherit
    );
    assert_eq!(status(&outcome, 2), ItemStatus::Partial);
}

#[test]
fn folders_follow_the_same_rules() {
    let xml = document(&[object("RoyalFolder", 2, 1, &[("CredentialMode", "1")])]);

    let outcome = import(&xml);

    let folder = find(&outcome.root, uuid(2)).as_folder().unwrap();
    assert_eq!(folder.credential, CredentialSpec::Prompt);
}

#[test]
fn an_inline_password_is_requested_with_path_and_user() {
    let xml = ssh_with(&[
        ("CredentialMode", "2"),
        ("CredentialUsername", "ops"),
        ("CredentialPassword", FAKE_CIPHERTEXT),
    ]);

    let (outcome, prompter) = import_answering(&xml, "typed-by-the-user");

    let expected = SecretRequest {
        node_path: "Doc/web01".to_owned(),
        username: Some("ops".to_owned()),
        field: SecretField::Password,
    };
    assert_eq!(prompter.requests, vec![expected]);
    let CredentialSpec::Inline { password, .. } = &connection(&outcome.root, 2).credential else {
        panic!("expected an inline credential");
    };
    let (reference, secret) = &outcome.secrets[0];
    assert_eq!(password.as_ref(), Some(reference));
    assert_eq!(secret.expose(), "typed-by-the-user");
    assert_eq!(status(&outcome, 2), ItemStatus::Imported);
}

#[test]
fn a_skipped_password_makes_the_node_partial() {
    let xml = ssh_with(&[
        ("CredentialMode", "2"),
        ("CredentialPassword", FAKE_CIPHERTEXT),
    ]);

    let mut prompter = ScriptedPrompter::skipping();

    let outcome = import_bytes(xml.as_bytes(), &options(), &mut prompter).unwrap();

    assert_eq!(prompter.requests.len(), 1);
    assert!(outcome.secrets.is_empty());
    assert_eq!(status(&outcome, 2), ItemStatus::Partial);
    assert_eq!(reasons(&outcome, 2), vec![SKIPPED_PASSWORD]);
}

#[test]
fn the_password_of_an_inactive_mode_is_not_requested() {
    let xml = ssh_with(&[
        ("CredentialMode", "1"),
        ("CredentialPassword", FAKE_CIPHERTEXT),
    ]);

    let (outcome, prompter) = import_answering(&xml, "unused");

    assert!(prompter.requests.is_empty());
    assert_eq!(status(&outcome, 2), ItemStatus::Imported);
}

#[test]
fn a_cancelled_prompt_aborts_the_import() {
    struct Cancelling;
    impl SecretPrompter for Cancelling {
        fn ask(
            &mut self,
            _req: &SecretRequest,
        ) -> Result<Option<portolano_core::Secret>, ImportError> {
            Err(ImportError::Cancelled)
        }
    }
    let xml = ssh_with(&[
        ("CredentialMode", "2"),
        ("CredentialPassword", FAKE_CIPHERTEXT),
    ]);

    let error = import_bytes(xml.as_bytes(), &options(), &mut Cancelling).unwrap_err();

    assert!(matches!(error, ImportError::Cancelled));
}

#[test]
fn a_credential_splits_domain_backslash_user() {
    let outcome = import(&credential_with(&[("UserName", "ACME\\administrator")]));

    let credential = find(&outcome.root, uuid(2)).as_credential().unwrap();
    assert_eq!(credential.username.as_deref(), Some("administrator"));
    assert_eq!(credential.domain.as_deref(), Some("ACME"));
}

#[test]
fn a_credential_prefers_the_domain_field_and_keeps_user_at_domain() {
    let outcome = import(&credential_with(&[
        ("UserName", "mario@acme.example.com"),
        ("Domain", "ACME"),
    ]));

    let credential = find(&outcome.root, uuid(2)).as_credential().unwrap();
    assert_eq!(
        credential.username.as_deref(),
        Some("mario@acme.example.com")
    );
    assert_eq!(credential.domain.as_deref(), Some("ACME"));
}

#[test]
fn a_credential_password_is_stored_as_a_vault_reference() {
    let xml = credential_with(&[("UserName", "root"), ("Password", FAKE_CIPHERTEXT)]);

    let (outcome, _) = import_answering(&xml, "typed-by-the-user");

    let credential = find(&outcome.root, uuid(2)).as_credential().unwrap();
    assert_eq!(credential.password.as_ref(), Some(&outcome.secrets[0].0));
}

#[test]
fn a_key_path_becomes_a_key_file() {
    let outcome = import(&credential_with(&[
        ("PrivateKeyPath", "~/.ssh/id_ed25519"),
        ("PrivateKeyMode", "0"),
    ]));

    let credential = find(&outcome.root, uuid(2)).as_credential().unwrap();
    assert_eq!(
        credential.private_key,
        Some(PrivateKey::File {
            path: "~/.ssh/id_ed25519".to_owned()
        })
    );
}

#[test]
fn an_embedded_key_and_its_passphrase_are_requested() {
    let xml = credential_with(&[
        ("UserName", "svc"),
        ("PrivateKeyMode", "1"),
        ("PrivateKeyContent", FAKE_CIPHERTEXT),
        ("Passphrase", FAKE_CIPHERTEXT),
    ]);

    let (outcome, prompter) = import_answering(&xml, "typed-by-the-user");

    let fields: Vec<SecretField> = prompter.requests.iter().map(|r| r.field.clone()).collect();
    assert_eq!(
        fields,
        vec![SecretField::PrivateKey, SecretField::Passphrase]
    );
    let credential = find(&outcome.root, uuid(2)).as_credential().unwrap();
    assert_eq!(
        credential.private_key,
        Some(PrivateKey::Vault {
            secret: outcome.secrets[0].0.clone()
        })
    );
    assert_eq!(credential.passphrase.as_ref(), Some(&outcome.secrets[1].0));
}

#[test]
fn a_provided_embedded_key_is_still_partial_and_keeps_the_mode() {
    let xml = credential_with(&[
        ("PrivateKeyMode", "1"),
        ("PrivateKeyContent", FAKE_CIPHERTEXT),
        ("Passphrase", FAKE_CIPHERTEXT),
    ]);

    let (outcome, _) = import_answering(&xml, "typed-by-the-user");

    assert_eq!(status(&outcome, 2), ItemStatus::Partial);
    assert_eq!(
        reasons(&outcome, 2),
        vec![
            "chiave privata incorporata: campi Royal non verificati",
            "passphrase: campi Royal non verificati"
        ]
    );
    assert_eq!(
        find(&outcome.root, uuid(2)).extra["royal.PrivateKeyMode"],
        "1"
    );
}

#[test]
fn a_skipped_embedded_key_makes_the_credential_partial() {
    let xml = credential_with(&[
        ("PrivateKeyMode", "1"),
        ("PrivateKeyContent", FAKE_CIPHERTEXT),
    ]);

    let outcome = import(&xml);

    assert_eq!(
        reasons(&outcome, 2),
        vec!["chiave privata Royal cifrata non importata: impostala con `portolano secret set`"]
    );
}
