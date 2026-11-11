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
fn each_fixture_imports_with_the_counts_of_the_importer() {
    for name in FIXTURES {
        let workspace = Workspace::new();
        let report = workspace.path("report.json");
        import(&workspace, name, &["--report", report.to_str().unwrap()]).success();
        let json: Value = serde_json::from_str(&fs::read_to_string(&report).unwrap()).unwrap();
        let expected = serde_json::to_value(expected_counts(name)).unwrap();
        assert_eq!(json["counts"], expected, "{name}");
    }
}

#[test]
fn minimal_fixture_imports_with_the_counts_of_its_objects() {
    // Counted by hand in the fixture: one document, five folders, one RDP
    // and one SSH connection, all importable.
    let workspace = Workspace::new();
    let report = workspace.path("report.json");
    import(
        &workspace,
        "minimal.royal.xml",
        &["--report", report.to_str().unwrap()],
    )
    .success();
    let json: Value = serde_json::from_str(&fs::read_to_string(&report).unwrap()).unwrap();
    let all_imported = |count: usize| {
        serde_json::json!({
            "source": count, "imported": count, "partial": 0, "unsupported": 0, "skipped": 0
        })
    };
    assert_eq!(
        json["counts"],
        serde_json::json!({
            "RoyalDocument": all_imported(1),
            "RoyalFolder": all_imported(5),
            "RoyalRDSConnection": all_imported(1),
            "RoyalSSHConnection": all_imported(1),
        })
    );
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

/// Number of folders at the root of the document.
fn root_folders(workspace: &Workspace) -> usize {
    let entries = workspace.ok(&["list", "--json", "--type", "folder"]);
    serde_json::from_str::<Value>(&entries)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| entry["depth"] == 0)
        .count()
}

#[test]
fn second_import_without_a_policy_lists_the_conflicts() {
    let workspace = Workspace::new();
    import(&workspace, "minimal.royal.xml", &[]).success();
    import(&workspace, "minimal.royal.xml", &[])
        .code(1)
        .stderr(contains("8 nodi dell'import esistono già"))
        .stderr(contains("--on-conflict"));
}

#[test]
fn second_import_with_update_replaces_in_place() {
    let workspace = Workspace::new();
    import(&workspace, "minimal.royal.xml", &[]).success();
    import(
        &workspace,
        "minimal.royal.xml",
        &["--on-conflict", "update"],
    )
    .success()
    .stdout(contains("0 nodi inseriti, 8 aggiornati"));
    assert_eq!(root_folders(&workspace), 1);
}

#[test]
fn second_import_with_new_ids_adds_a_copy() {
    let workspace = Workspace::new();
    import(&workspace, "minimal.royal.xml", &[]).success();
    import(
        &workspace,
        "minimal.royal.xml",
        &["--on-conflict", "new-ids"],
    )
    .success()
    .stdout(contains("8 nodi inseriti"));
    assert_eq!(root_folders(&workspace), 2);
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

/// A Royal document whose names, host and user carry terminal escape
/// sequences as XML character references (DEC-024.3).
const ESCAPES_DOCUMENT: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<RTSZDocument>
  <RoyalDocument>
    <ID>e0000000-0000-4000-8000-000000000001</ID>
    <Name>Doc&#x1b;]0;titolo&#x7;</Name>
  </RoyalDocument>
  <RoyalSSHConnection>
    <ID>e0000000-0000-4000-8000-000000000002</ID>
    <ParentID>e0000000-0000-4000-8000-000000000001</ParentID>
    <Name>web&#x1b;[2J01</Name>
    <URI>web01.example.com&#x1b;[1m</URI>
    <CredentialMode>2</CredentialMode>
    <CredentialUsername>adm&#xd;in</CredentialUsername>
  </RoyalSSHConnection>
</RTSZDocument>
"#;

const ESCAPE: char = '\u{1b}';

fn import_escapes(workspace: &Workspace) -> assert_cmd::assert::Assert {
    let file = workspace.path("escapes.rtsx");
    fs::write(&file, ESCAPES_DOCUMENT).unwrap();
    workspace.run(&["import", file.to_str().unwrap()])
}

#[test]
fn control_characters_of_an_import_are_shown_escaped_in_the_report() {
    let workspace = Workspace::new();
    let output = all_output(&import_escapes(&workspace).success());
    assert!(output.contains("Doc\\u{1b}]0;titolo\\u{7}"), "{output}");
    assert!(!output.contains(ESCAPE));
}

#[test]
fn control_characters_of_an_imported_name_are_shown_escaped_by_tree_and_list() {
    let workspace = Workspace::new();
    import_escapes(&workspace).success();
    for command in [&["tree"][..], &["list"][..]] {
        let output = workspace.ok(command);
        assert!(output.contains("web\\u{1b}[2J01"), "{output}");
        assert!(output.contains("web01.example.com\\u{1b}[1m"), "{output}");
        assert!(!output.contains(ESCAPE), "{command:?}");
        assert!(!output.contains('\r'), "{command:?}");
    }
}

#[test]
fn control_characters_are_shown_escaped_by_show_validate_and_connect_print() {
    let workspace = Workspace::new();
    import_escapes(&workspace).success();
    let node = "@e0000000-0000-4000-8000-000000000002";
    let shown = workspace.ok(&["show", "--resolved", node]);
    assert!(shown.contains("adm\\u{d}in"), "{shown}");
    let validated = all_output(&workspace.run(&["validate"]));
    assert!(validated.contains("[control_characters]"), "{validated}");
    let printed = all_output(&workspace.run(&["connect", "--print", node]));
    for output in [shown, validated, printed] {
        assert!(!output.contains(ESCAPE), "{output}");
        assert!(!output.contains('\r'), "{output}");
    }
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
