use pretty_assertions::assert_eq;

use super::*;
use crate::model::Unsupported;
use crate::testutil::{connection, credential, document, folder, with_spec};

const ADMIN: Uuid = Uuid::from_u128(0xad);
const TOP: Uuid = Uuid::from_u128(0x10);
const MIDDLE: Uuid = Uuid::from_u128(0x20);
const BOTTOM: Uuid = Uuid::from_u128(0x30);
const LEAF: Uuid = Uuid::from_u128(0x40);
const NO_CREDENTIAL: Uuid = Uuid::from_u128(0x50);
const UNDER_NONE: Uuid = Uuid::from_u128(0x60);
const AT_ROOT: Uuid = Uuid::from_u128(0x70);
const MISSING: Uuid = Uuid::from_u128(0xf0);

fn admin(password: &SecretRef) -> Node {
    let mut node = credential(ADMIN, "Admin ACME");
    let fields = node.as_credential_mut().unwrap();
    fields.username = Some("admin".to_owned());
    fields.domain = Some("ACME".to_owned());
    fields.password = Some(password.clone());
    fields.private_key = Some(PrivateKey::File {
        path: "~/.ssh/id_test".to_owned(),
    });
    node
}

/// ```text
/// Admin ACME (credential)
/// Top (by_id Admin ACME)
///   Middle / Bottom / leaf     (all inherit)
///   NoCredential (none)
///     under-none               (inherit)
/// at-root                      (inherit)
/// ```
fn sample(password: &SecretRef) -> Document {
    document(vec![
        admin(password),
        with_spec(
            folder(
                TOP,
                "Top",
                vec![
                    folder(
                        MIDDLE,
                        "Middle",
                        vec![folder(BOTTOM, "Bottom", vec![connection(LEAF, "leaf")])],
                    ),
                    with_spec(
                        folder(
                            NO_CREDENTIAL,
                            "NoCredential",
                            vec![connection(UNDER_NONE, "under-none")],
                        ),
                        CredentialSpec::None,
                    ),
                ],
            ),
            CredentialSpec::ById { id: ADMIN },
        ),
        connection(AT_ROOT, "at-root"),
    ])
}

fn admin_resolved(password: &SecretRef) -> ResolvedCredential {
    ResolvedCredential {
        origin: CredentialOrigin::Credential { node: ADMIN },
        username: Some("admin".to_owned()),
        domain: Some("ACME".to_owned()),
        password: Some(password.clone()),
        private_key: Some(PrivateKey::File {
            path: "~/.ssh/id_test".to_owned(),
        }),
        passphrase: None,
    }
}

fn origin(result: Result<ResolvedCredential>) -> CredentialOrigin {
    result.unwrap().origin
}

#[test]
fn inherit_chain_over_three_levels_reaches_the_folder_credential() {
    let password = SecretRef::new();
    let document = sample(&password);

    let resolved = document.resolve_credential(LEAF).unwrap();

    assert_eq!(resolved, admin_resolved(&password));
}

#[test]
fn inherit_up_to_the_root_has_no_credential() {
    let document = sample(&SecretRef::new());

    assert_eq!(
        origin(document.resolve_credential(AT_ROOT)),
        CredentialOrigin::None
    );
}

#[test]
fn none_stops_the_inheritance_chain() {
    let document = sample(&SecretRef::new());

    assert_eq!(
        origin(document.resolve_credential(UNDER_NONE)),
        CredentialOrigin::None
    );
}

#[test]
fn prompt_asks_at_connection_time() {
    let document = sample(&SecretRef::new());

    let resolved = document.resolve_spec(LEAF, &CredentialSpec::Prompt);

    assert_eq!(origin(resolved), CredentialOrigin::Prompt);
}

#[test]
fn inline_copies_the_fields_of_the_context_node() {
    let password = SecretRef::new();
    let document = sample(&SecretRef::new());
    let spec = CredentialSpec::Inline {
        username: Some("operator".to_owned()),
        domain: Some("LAB".to_owned()),
        password: Some(password.clone()),
    };

    let resolved = document.resolve_spec(LEAF, &spec).unwrap();

    assert_eq!(
        resolved,
        ResolvedCredential {
            origin: CredentialOrigin::Inline { node: LEAF },
            username: Some("operator".to_owned()),
            domain: Some("LAB".to_owned()),
            password: Some(password),
            private_key: None,
            passphrase: None,
        }
    );
}

#[test]
fn inherited_inline_spec_names_the_folder_that_holds_it() {
    let mut document = sample(&SecretRef::new());
    document
        .find_mut(MIDDLE)
        .unwrap()
        .as_folder_mut()
        .unwrap()
        .credential = CredentialSpec::Inline {
        username: Some("operator".to_owned()),
        domain: None,
        password: None,
    };

    let resolved = document.resolve_credential(LEAF);

    assert_eq!(origin(resolved), CredentialOrigin::Inline { node: MIDDLE });
}

#[test]
fn by_id_returns_the_fields_of_the_credential() {
    let password = SecretRef::new();
    let document = sample(&password);

    let resolved = document
        .resolve_spec(AT_ROOT, &CredentialSpec::ById { id: ADMIN })
        .unwrap();

    assert_eq!(resolved, admin_resolved(&password));
}

#[test]
fn by_id_to_a_missing_node_is_a_dangling_reference() {
    let document = sample(&SecretRef::new());

    let result = document.resolve_spec(AT_ROOT, &CredentialSpec::ById { id: MISSING });

    assert!(matches!(
        result,
        Err(Error::DanglingReference { from, target }) if from == AT_ROOT && target == MISSING
    ));
}

#[test]
fn inherited_dangling_reference_names_the_folder_that_holds_it() {
    let mut document = sample(&SecretRef::new());
    document
        .find_mut(TOP)
        .unwrap()
        .as_folder_mut()
        .unwrap()
        .credential = CredentialSpec::ById { id: MISSING };

    let result = document.resolve_credential(LEAF);

    assert!(matches!(
        result,
        Err(Error::DanglingReference { from, target }) if from == TOP && target == MISSING
    ));
}

#[test]
fn by_id_to_a_connection_is_not_a_credential() {
    let document = sample(&SecretRef::new());

    let result = document.resolve_spec(LEAF, &CredentialSpec::ById { id: AT_ROOT });

    assert!(matches!(
        result,
        Err(Error::NotACredential { from, target }) if from == LEAF && target == AT_ROOT
    ));
}

#[test]
fn by_name_with_a_unique_name_returns_the_credential() {
    let password = SecretRef::new();
    let document = sample(&password);
    let spec = CredentialSpec::ByName {
        name: "Admin ACME".to_owned(),
    };

    let resolved = document.resolve_spec(LEAF, &spec).unwrap();

    assert_eq!(resolved, admin_resolved(&password));
}

#[test]
fn by_name_falls_back_to_a_case_insensitive_match() {
    let document = sample(&SecretRef::new());
    let spec = CredentialSpec::ByName {
        name: "admin acme".to_owned(),
    };

    let resolved = document.resolve_spec(LEAF, &spec);

    assert_eq!(
        origin(resolved),
        CredentialOrigin::Credential { node: ADMIN }
    );
}

#[test]
fn by_name_prefers_the_exact_match_over_case_insensitive_ones() {
    let exact = Uuid::from_u128(0xe0);
    let mut document = sample(&SecretRef::new());
    document
        .insert(None, None, credential(exact, "admin acme"))
        .unwrap();
    let spec = CredentialSpec::ByName {
        name: "admin acme".to_owned(),
    };

    let resolved = document.resolve_spec(LEAF, &spec);

    assert_eq!(
        origin(resolved),
        CredentialOrigin::Credential { node: exact }
    );
}

#[test]
fn by_name_with_a_repeated_name_is_ambiguous() {
    let other = Uuid::from_u128(0xe1);
    let mut document = sample(&SecretRef::new());
    document
        .insert(Some(TOP), None, credential(other, "Admin ACME"))
        .unwrap();
    let spec = CredentialSpec::ByName {
        name: "Admin ACME".to_owned(),
    };

    let result = document.resolve_spec(LEAF, &spec);

    let Err(Error::Ambiguous { query, candidates }) = result else {
        panic!("expected Ambiguous, got {result:?}");
    };
    assert_eq!(query, "Admin ACME");
    assert_eq!(
        candidates,
        vec![
            (ADMIN, "/Admin ACME".to_owned()),
            (other, "/Top/Admin ACME".to_owned())
        ]
    );
}

#[test]
fn by_name_with_an_unknown_name_is_not_found() {
    let document = sample(&SecretRef::new());
    let spec = CredentialSpec::ByName {
        name: "Nobody".to_owned(),
    };

    let result = document.resolve_spec(LEAF, &spec);

    assert!(matches!(result, Err(Error::NotFound { query }) if query == "Nobody"));
}

#[test]
fn by_name_ignores_nodes_that_are_not_credentials() {
    let document = sample(&SecretRef::new());
    let spec = CredentialSpec::ByName {
        name: "at-root".to_owned(),
    };

    let result = document.resolve_spec(LEAF, &spec);

    assert!(matches!(result, Err(Error::NotFound { .. })));
}

#[test]
fn resolve_credential_of_a_credential_returns_its_own_fields() {
    let password = SecretRef::new();
    let document = sample(&password);

    let resolved = document.resolve_credential(ADMIN).unwrap();

    assert_eq!(resolved, admin_resolved(&password));
}

#[test]
fn resolve_credential_of_a_folder_uses_its_own_spec() {
    let document = sample(&SecretRef::new());

    let resolved = document.resolve_credential(TOP);

    assert_eq!(
        origin(resolved),
        CredentialOrigin::Credential { node: ADMIN }
    );
}

#[test]
fn resolve_credential_of_an_unsupported_node_is_rejected() {
    let unsupported_id = Uuid::from_u128(0xe2);
    let mut unsupported = folder(unsupported_id, "vnc-legacy", vec![]);
    unsupported.kind = NodeKind::Unsupported(Unsupported {
        source_type: "RoyalVNCConnection".to_owned(),
        raw: "<RoyalVNCConnection />".to_owned(),
    });
    let document = document(vec![unsupported]);

    let result = document.resolve_credential(unsupported_id);

    assert!(matches!(result, Err(Error::NoCredential { id }) if id == unsupported_id));
}

#[test]
fn resolve_credential_of_a_missing_node_is_rejected() {
    let document = sample(&SecretRef::new());

    let result = document.resolve_credential(MISSING);

    assert!(matches!(result, Err(Error::NodeNotFound { id }) if id == MISSING));
}

#[test]
fn inherit_from_a_missing_context_is_rejected() {
    let document = sample(&SecretRef::new());

    let result = document.resolve_spec(MISSING, &CredentialSpec::Inherit);

    assert!(matches!(result, Err(Error::NodeNotFound { id }) if id == MISSING));
}
