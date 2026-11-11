use portolano_core::vault::MemoryVault;
use portolano_core::{CredentialSpec, Unsupported};
use pretty_assertions::assert_eq;

use super::*;
use crate::testutil::*;

fn sample() -> Document {
    let cred = credential("Admin", "admin", None);
    let cred_id = cred.id;
    let mut unsupported = Node::folder("vnc-legacy");
    unsupported.kind = NodeKind::Unsupported(Unsupported {
        source_type: "RoyalVNCConnection".to_owned(),
        raw: "<RoyalVNCConnection/>".to_owned(),
    });
    document(vec![
        folder(
            "Clienti",
            vec![
                connection_with("web01", "ssh", CredentialSpec::ById { id: cred_id }),
                connection("dc01", "rdp"),
                cred,
            ],
        ),
        unsupported,
        connection("legacy", "telnet"),
    ])
}

fn names(entries: &[Entry]) -> Vec<(usize, &str)> {
    entries
        .iter()
        .map(|entry| (entry.depth, entry.name.as_str()))
        .collect()
}

#[test]
fn entries_of_the_whole_document_are_in_pre_order_with_depths() {
    let doc = sample();
    let found = entries(&doc, None, &Registry::with_defaults(), None, None).unwrap();
    assert_eq!(
        names(&found),
        vec![
            (0, "Clienti"),
            (1, "web01"),
            (1, "dc01"),
            (1, "Admin"),
            (0, "vnc-legacy"),
            (0, "legacy"),
        ]
    );
}

#[test]
fn entries_from_a_folder_list_its_descendants_only() {
    let doc = sample();
    let start = id_of(&doc, "/Clienti");
    let found = entries(&doc, Some(start), &Registry::with_defaults(), None, None).unwrap();
    assert_eq!(names(&found), vec![(0, "web01"), (0, "dc01"), (0, "Admin")]);
}

#[test]
fn max_depth_keeps_the_first_levels() {
    let doc = sample();
    let found = entries(&doc, None, &Registry::with_defaults(), Some(1), None).unwrap();
    assert_eq!(
        names(&found),
        vec![(0, "Clienti"), (0, "vnc-legacy"), (0, "legacy")]
    );
}

#[test]
fn protocol_filter_keeps_matching_connections_at_any_depth() {
    let doc = sample();
    let filter = TypeFilter::Protocol("rdp".to_owned());
    let found = entries(&doc, None, &Registry::with_defaults(), None, Some(&filter)).unwrap();
    assert_eq!(names(&found), vec![(1, "dc01")]);
}

#[test]
fn connection_filter_keeps_every_protocol() {
    let doc = sample();
    let found = entries(
        &doc,
        None,
        &Registry::with_defaults(),
        None,
        Some(&TypeFilter::Connection),
    )
    .unwrap();
    assert_eq!(
        names(&found),
        vec![(1, "web01"), (1, "dc01"), (0, "legacy")]
    );
}

#[test]
fn entries_carry_paths_and_summaries() {
    let doc = sample();
    let found = entries(&doc, None, &Registry::with_defaults(), None, None).unwrap();
    let web01 = &found[1];
    assert_eq!(web01.path, "/Clienti/web01");
    assert_eq!(web01.summary, "admin@web01.example.com:22");
    assert_eq!(found[3].summary, "admin");
    assert_eq!(found[4].summary, "non supportato: RoyalVNCConnection");
    assert_eq!(
        found[5].summary,
        "legacy.example.com (protocollo sconosciuto: telnet)"
    );
}

#[test]
fn credential_summary_shows_the_domain() {
    let mut node = credential("Admin ACME", "administrator", None);
    if let Some(credential) = node.as_credential_mut() {
        credential.domain = Some("ACME".to_owned());
    }
    let doc = document(vec![node.clone()]);
    assert_eq!(
        summary(&doc, &node, &Registry::with_defaults()),
        "ACME\\administrator"
    );
}

#[test]
fn resolved_credential_reports_presence_without_the_secret() {
    let (vault, reference) = vault_with(SENTINEL);
    let cred = credential("Admin", "admin", Some(reference));
    let cred_id = cred.id;
    let doc = document(vec![
        cred,
        connection_with("web01", "ssh", CredentialSpec::ById { id: cred_id }),
    ]);
    let view = resolved_credential(&doc, id_of(&doc, "/web01"), &vault).unwrap();
    assert_eq!(
        view.origin,
        OriginView::Credential {
            node: cred_id,
            path: "/Admin".to_owned()
        }
    );
    assert_eq!(view.username.as_deref(), Some("admin"));
    assert_eq!(view.password, SecretState::Present);
    assert_eq!(view.passphrase, SecretState::NotSet);
    assert!(!serde_json::to_string(&view).unwrap().contains(SENTINEL));
}

#[test]
fn resolved_credential_reports_a_secret_missing_from_the_vault() {
    let doc = document(vec![credential("Admin", "admin", Some(SecretRef::new()))]);
    let view = resolved_credential(&doc, id_of(&doc, "/Admin"), &MemoryVault::default()).unwrap();
    assert_eq!(view.password, SecretState::Absent);
}

#[test]
fn entry_paths_are_those_of_the_document_with_escapes() {
    let doc = document(vec![folder(
        "a/b",
        vec![folder("c\\d", vec![connection("web01", "ssh")])],
    )]);
    let start = id_of(&doc, "/a\\/b");
    for start in [None, Some(start)] {
        let found = entries(&doc, start, &Registry::with_defaults(), None, None).unwrap();
        for entry in found {
            assert_eq!(Some(entry.path.clone()), doc.path_of(entry.id));
        }
    }
}
