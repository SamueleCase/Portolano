//! `import` of the synthetic fixtures of `portolano-import`.

use std::collections::BTreeMap;
use std::fs;

use portolano_import::{ImportOptions, Importer, NonInteractive, TypeCounts, royal::RoyalImporter};
use predicates::str::contains;
use pretty_assertions::assert_eq;
use serde_json::Value;

use crate::testutil::*;

const FIXTURES: [&str; 3] = [
    "minimal.royal.xml",
    "workspace.royal.xml",
    "full-synthetic.royal.xml",
];

/// The counts of the importer run directly on the fixture.
fn expected_counts(name: &str) -> BTreeMap<String, TypeCounts> {
    let options = ImportOptions {
        source_file_name: name.to_owned(),
        include_trash: false,
    };
    RoyalImporter
        .import(
            &fs::read(fixture(name)).unwrap(),
            &options,
            &mut NonInteractive,
        )
        .unwrap()
        .report
        .counts
}

fn import(workspace: &Workspace, name: &str, extra: &[&str]) -> assert_cmd::assert::Assert {
    let file = fixture(name);
    let mut args = vec!["import", file.to_str().unwrap()];
    args.extend_from_slice(extra);
    workspace.run(&args)
}

#[test]
fn each_fixture_imports_with_the_expected_counts() {
    for name in FIXTURES {
        let workspace = Workspace::new();
        let report = workspace.path("report.json");
        import(&workspace, name, &["--report", report.to_str().unwrap()]).success();
        let json: Value = serde_json::from_str(&fs::read_to_string(&report).unwrap()).unwrap();
        let counts: BTreeMap<String, TypeCounts> = expected_counts(name);
        let reported = json["counts"].as_object().unwrap();
        assert_eq!(reported.len(), counts.len(), "{name}");
        for (source_type, expected) in counts {
            let got = &reported[&source_type];
            assert_eq!(got["source"], expected.source, "{name} {source_type}");
            assert_eq!(got["imported"], expected.imported, "{name} {source_type}");
            assert_eq!(got["partial"], expected.partial, "{name} {source_type}");
            assert_eq!(
                got["unsupported"], expected.unsupported,
                "{name} {source_type}"
            );
            assert_eq!(got["skipped"], expected.skipped, "{name} {source_type}");
        }
    }
}

#[test]
fn import_prints_the_report_table() {
    let workspace = Workspace::new();
    import(&workspace, "workspace.royal.xml", &[])
        .success()
        .stdout(contains("TIPO"))
        .stdout(contains("RoyalSSHConnection"))
        .stdout(contains("[parziale]"))
        .stdout(contains("Fatto:"));
}

#[test]
fn second_import_conflicts_unless_a_policy_is_given() {
    let workspace = Workspace::new();
    import(&workspace, "minimal.royal.xml", &[]).success();
    import(&workspace, "minimal.royal.xml", &[])
        .code(1)
        .stderr(contains("--on-conflict"));
    import(
        &workspace,
        "minimal.royal.xml",
        &["--on-conflict", "update"],
    )
    .success()
    .stdout(contains("0 nodi inseriti"));
    import(
        &workspace,
        "minimal.royal.xml",
        &["--on-conflict", "new-ids"],
    )
    .success();
    let tree = workspace.ok(&["list", "--json", "--type", "folder"]);
    let roots = serde_json::from_str::<Value>(&tree)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| entry["depth"] == 0)
        .count();
    assert_eq!(roots, 2);
}

#[test]
fn dry_run_writes_nothing() {
    let workspace = Workspace::new();
    let before = workspace.doc_text();
    import(&workspace, "full-synthetic.royal.xml", &["--dry-run"])
        .success()
        .stdout(contains("Simulazione"));
    assert_eq!(workspace.doc_text(), before);
}

#[test]
fn import_goes_into_the_given_folder() {
    let workspace = Workspace::new();
    workspace.ok(&["add", "folder", "/Import"]);
    import(&workspace, "minimal.royal.xml", &["--into", "/Import"]).success();
    let entries = workspace.ok(&["list", "--json", "/Import"]);
    assert!(
        !serde_json::from_str::<Value>(&entries)
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn prompt_secrets_without_a_terminal_fails() {
    let workspace = Workspace::new();
    import(&workspace, "workspace.royal.xml", &["--prompt-secrets"])
        .code(1)
        .stderr(contains("nessun terminale"));
}

#[test]
fn unrecognized_file_is_refused() {
    let workspace = Workspace::new();
    let notes = workspace.path("notes.txt");
    fs::write(&notes, "just notes").unwrap();
    workspace
        .run(&["import", notes.to_str().unwrap()])
        .code(1)
        .stderr(contains("non riconosciuto"));
}

#[test]
fn unknown_format_is_a_usage_error() {
    let workspace = Workspace::new();
    import(&workspace, "minimal.royal.xml", &["--format", "mremoteng"])
        .code(2)
        .stderr(contains("formato d'importazione sconosciuto"));
}

#[test]
fn import_too_deep_for_the_destination_exits_with_4() {
    let workspace = Workspace::new();
    let path = "/l".repeat(portolano_core::MAX_DEPTH);
    workspace.ok(&["add", "folder", "-p", &path]);
    import(&workspace, "minimal.royal.xml", &["--into", &path])
        .code(4)
        .stderr(contains("scegli una cartella meno profonda"));
}
