//! Imports of the synthetic fixtures (SPEC §8.3.6), plain and gzip, and the
//! check that no ciphertext survives anywhere in the outcome (D3).

use std::collections::BTreeMap;

use portolano_core::{Node, NodeKind};
use portolano_import::{ImportOutcome, ItemStatus, NonInteractive, TypeCounts};
use pretty_assertions::assert_eq;

use crate::testutil::*;

const FIXTURES: [&str; 3] = [
    "minimal.royal.xml",
    "workspace.royal.xml",
    "full-synthetic.royal.xml",
];

/// Elements of the fixtures that hold encrypted values.
/// `Data` and `Item` hold values nested two and three levels under a
/// `Password` element.
const ENCRYPTED_ELEMENTS: [&str; 9] = [
    "Password",
    "CredentialPassword",
    "PasswordHash",
    "Passphrase",
    "PrivateKeyContent",
    "CustomProtectedField1",
    "SSHKeyPassword",
    "Data",
    "Item",
];

fn import_fixture(name: &str) -> ImportOutcome {
    import_bytes(&fixture(name), &options(), &mut NonInteractive).unwrap()
}

/// Attributes of the fixtures that hold encrypted values.
const ENCRYPTED_ATTRIBUTES: [&str; 2] = ["value", "Passphrase"];

/// The encrypted values written in a fixture, as element text (also nested
/// in other fields) or as attribute values.
fn ciphertexts(name: &str) -> Vec<String> {
    let text = String::from_utf8(fixture(name)).unwrap();
    let elements = ENCRYPTED_ELEMENTS.iter().flat_map(|element| {
        let open = format!("<{element}>");
        let close = format!("</{element}>");
        text.split(&open)
            .skip(1)
            .filter_map(|rest| rest.split_once(&close).map(|(value, _)| value.to_owned()))
            .collect::<Vec<_>>()
    });
    let attributes = ENCRYPTED_ATTRIBUTES.iter().flat_map(|attribute| {
        text.split(&format!(" {attribute}=\""))
            .skip(1)
            .filter_map(|rest| rest.split_once('"').map(|(value, _)| value.to_owned()))
            .collect::<Vec<_>>()
    });
    // An element wrapping others yields markup, not a value: the value is
    // collected from the innermost element.
    elements
        .chain(attributes)
        .filter(|value| !value.contains('<'))
        .collect()
}

/// Everything an outcome can expose: the tree, the report and the debug
/// form of the whole outcome (secrets, icons).
fn exposed_text(outcome: &ImportOutcome) -> String {
    format!(
        "{}\n{}\n{:?}",
        serde_json::to_string(&outcome.root).unwrap(),
        serde_json::to_string(&outcome.report).unwrap(),
        outcome
    )
}

/// Pre-order (id, name, kind) of a tree: what must not depend on the
/// container or the time of the import.
fn shape(node: &Node) -> Vec<(uuid::Uuid, String, &'static str)> {
    let kind = match &node.kind {
        NodeKind::Folder(_) => "folder",
        NodeKind::Connection(_) => "connection",
        NodeKind::Credential(_) => "credential",
        NodeKind::Unsupported(_) => "unsupported",
    };
    let mut nodes = vec![(node.id, node.name.clone(), kind)];
    if let Some(folder) = node.as_folder() {
        nodes.extend(folder.children.iter().flat_map(shape));
    }
    nodes
}

fn counts(
    source: usize,
    imported: usize,
    partial: usize,
    unsupported: usize,
    skipped: usize,
) -> TypeCounts {
    TypeCounts {
        source,
        imported,
        partial,
        unsupported,
        skipped,
    }
}

#[test]
fn every_fixture_has_encrypted_values_to_check() {
    assert!(ciphertexts("minimal.royal.xml").is_empty());
    assert_eq!(ciphertexts("workspace.royal.xml").len(), 7);
    assert_eq!(ciphertexts("full-synthetic.royal.xml").len(), 12);
}

#[test]
fn no_ciphertext_appears_in_the_outcome_when_secrets_are_skipped() {
    for name in FIXTURES {
        let exposed = exposed_text(&import_fixture(name));

        for ciphertext in ciphertexts(name) {
            assert!(
                !exposed.contains(&ciphertext),
                "ciphertext leaked from {name}"
            );
        }
    }
}

#[test]
fn no_ciphertext_appears_in_the_outcome_when_secrets_are_typed() {
    for name in FIXTURES {
        let mut prompter = ScriptedPrompter::answering("typed-by-the-user");
        let outcome = import_bytes(&fixture(name), &options(), &mut prompter).unwrap();

        let exposed = exposed_text(&outcome);
        for ciphertext in ciphertexts(name) {
            assert!(
                !exposed.contains(&ciphertext),
                "ciphertext leaked from {name}"
            );
        }
        for (_, secret) in &outcome.secrets {
            assert_eq!(secret.expose(), "typed-by-the-user");
        }
    }
}

#[test]
fn gzip_fixtures_import_like_the_plain_ones() {
    for name in FIXTURES {
        let plain = import_fixture(name);
        let compressed =
            import_bytes(&gzip(&fixture(name)), &options(), &mut NonInteractive).unwrap();

        assert_eq!(shape(&compressed.root), shape(&plain.root), "{name}");
        assert_eq!(compressed.report.counts, plain.report.counts, "{name}");
    }
}

#[test]
fn report_statuses_add_up_to_the_source_objects() {
    for name in FIXTURES {
        let outcome = import_fixture(name);
        let objects = String::from_utf8(fixture(name))
            .unwrap()
            .matches("<ID>")
            .count();

        let total: usize = outcome.report.counts.values().map(|c| c.source).sum();
        let statuses: usize = outcome
            .report
            .counts
            .values()
            .map(|c| c.imported + c.partial + c.unsupported + c.skipped)
            .sum();
        assert_eq!((total, statuses), (objects, objects), "{name}");
    }
}

#[test]
fn report_items_list_only_objects_not_fully_imported() {
    for name in FIXTURES {
        let outcome = import_fixture(name);

        assert!(
            outcome
                .report
                .items
                .iter()
                .all(|item| item.status != ItemStatus::Imported && !item.reasons.is_empty())
        );
    }
}

#[test]
fn minimal_imports_everything() {
    let outcome = import_fixture("minimal.royal.xml");

    let expected: BTreeMap<String, TypeCounts> = [
        ("RoyalDocument", counts(1, 1, 0, 0, 0)),
        ("RoyalFolder", counts(5, 5, 0, 0, 0)),
        ("RoyalRDSConnection", counts(1, 1, 0, 0, 0)),
        ("RoyalSSHConnection", counts(1, 1, 0, 0, 0)),
    ]
    .map(|(name, counts)| (name.to_owned(), counts))
    .into();
    assert_eq!(outcome.report.counts, expected);
    assert!(outcome.report.items.is_empty() && outcome.report.warnings.is_empty());
    assert_eq!(
        child_names(&outcome.root),
        vec!["Clienti", "Interni", "Archivio"]
    );
}

#[test]
fn workspace_counts_and_order() {
    let outcome = import_fixture("workspace.royal.xml");

    let expected: BTreeMap<String, TypeCounts> = [
        ("RoyalCredential", counts(3, 0, 3, 0, 0)),
        ("RoyalDocument", counts(1, 1, 0, 0, 0)),
        ("RoyalFolder", counts(4, 3, 0, 0, 1)),
        ("RoyalRDSConnection", counts(5, 3, 1, 0, 1)),
        ("RoyalSSHConnection", counts(4, 2, 1, 0, 1)),
        ("RoyalTrash", counts(1, 0, 0, 0, 1)),
    ]
    .map(|(name, counts)| (name.to_owned(), counts))
    .into();
    assert_eq!(outcome.report.counts, expected);
    assert_eq!(
        outcome.report.warnings,
        vec!["documento protetto da password"]
    );
    assert_eq!(
        child_names(&outcome.root),
        vec!["Credenziali", "Server Linux", "Windows"]
    );
    let linux = outcome.root.as_folder().unwrap().children[1].clone();
    assert_eq!(child_names(&linux), vec!["bastion", "db01", "web01"]);
}

#[test]
fn workspace_prompts_for_the_secrets_outside_the_trash_only() {
    let mut prompter = ScriptedPrompter::answering("typed-by-the-user");

    let outcome = import_bytes(&fixture("workspace.royal.xml"), &options(), &mut prompter).unwrap();

    let paths: Vec<&str> = prompter
        .requests
        .iter()
        .map(|r| r.node_path.as_str())
        .collect();
    assert_eq!(
        paths,
        vec![
            "Workspace di prova/Credenziali/Admin ACME",
            "Workspace di prova/Credenziali/Root Linux",
            "Workspace di prova/Credenziali/Deploy",
            "Workspace di prova/Server Linux/bastion",
            "Workspace di prova/Windows/app01",
        ]
    );
    assert_eq!(outcome.secrets.len(), 5);
    assert!(
        outcome
            .report
            .items
            .iter()
            .all(|i| i.status == ItemStatus::Skipped)
    );
}

#[test]
fn full_synthetic_counts() {
    let outcome = import_fixture("full-synthetic.royal.xml");

    let expected: BTreeMap<String, TypeCounts> = [
        ("RoyalCredential", counts(2, 1, 1, 0, 0)),
        ("RoyalDocument", counts(1, 1, 0, 0, 0)),
        ("RoyalDynamicFolder", counts(1, 0, 0, 1, 0)),
        ("RoyalFileTransferConnection", counts(1, 0, 0, 1, 0)),
        ("RoyalFolder", counts(3, 1, 2, 0, 0)),
        ("RoyalHologramConnection", counts(1, 0, 0, 1, 0)),
        ("RoyalRDSConnection", counts(4, 0, 4, 0, 0)),
        ("RoyalRDSGateway", counts(1, 0, 0, 1, 0)),
        ("RoyalSSHConnection", counts(4, 0, 2, 2, 0)),
        ("RoyalSecureGateway", counts(1, 1, 0, 0, 0)),
        ("RoyalVNCConnection", counts(1, 0, 0, 1, 0)),
        ("RoyalWebConnection", counts(1, 0, 0, 1, 0)),
    ]
    .map(|(name, counts)| (name.to_owned(), counts))
    .into();
    assert_eq!(outcome.report.counts, expected);
    assert_eq!(outcome.icons.len(), 1);
    assert_eq!(outcome.report.warnings.len(), 1);
}
