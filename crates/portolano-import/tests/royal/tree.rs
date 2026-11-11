//! Tree rebuilding, root folder, ids, source info and trash (SPEC §8.3.3–4).

use portolano_import::{ImportError, ImportOptions, ItemStatus, NonInteractive};
use pretty_assertions::assert_eq;

use crate::testutil::*;

fn folder(n: u32, parent: u32, name: &str, position: Option<&str>) -> String {
    let mut fields = vec![("Name", name)];
    if let Some(position) = position {
        fields.push(("PositionNr", position));
    }
    object("RoyalFolder", n, parent, &fields)
}

#[test]
fn siblings_are_ordered_by_numeric_position() {
    let xml = document(&[
        folder(2, 1, "dieci", Some("10")),
        folder(3, 1, "otto", Some("8")),
        folder(4, 1, "due", Some("2")),
    ]);

    let outcome = import(&xml);

    assert_eq!(child_names(&outcome.root), vec!["due", "otto", "dieci"]);
}

#[test]
fn equal_or_missing_positions_keep_file_order() {
    let xml = document(&[
        folder(2, 1, "uno", Some("1")),
        folder(3, 1, "senza-a", None),
        folder(4, 1, "zero", Some("0")),
        folder(5, 1, "senza-b", None),
    ]);

    let outcome = import(&xml);

    assert_eq!(
        child_names(&outcome.root),
        vec!["senza-a", "zero", "senza-b", "uno"]
    );
}

#[test]
fn children_follow_parent_id_at_any_depth() {
    let xml = document(&[
        folder(4, 3, "nipote", None),
        folder(3, 2, "figlio", None),
        folder(2, 1, "padre", None),
    ]);

    let outcome = import(&xml);

    let parent = find(&outcome.root, uuid(2));
    let child = find(parent, uuid(3));
    assert_eq!(child_names(child), vec!["nipote"]);
}

#[test]
fn an_orphan_goes_under_the_root_with_a_warning() {
    let xml = document(&[folder(2, 99, "orfana", None)]);

    let outcome = import(&xml);

    assert_eq!(child_names(&outcome.root), vec!["orfana"]);
    assert_eq!(
        outcome.report.warnings,
        vec![
            "oggetto `orfana` (RoyalFolder): genitore non trovato nel documento, inserito nella radice"
        ]
    );
}

#[test]
fn a_child_of_a_non_container_goes_under_the_root() {
    let xml = document(&[
        object(
            "RoyalSSHConnection",
            2,
            1,
            &[("Name", "ssh"), ("URI", "a.example.com")],
        ),
        folder(3, 2, "dentro-ssh", None),
    ]);

    let outcome = import(&xml);

    assert_eq!(child_names(&outcome.root), vec!["ssh", "dentro-ssh"]);
    assert_eq!(outcome.report.warnings.len(), 1);
}

#[test]
fn a_parent_id_cycle_is_broken_under_the_root() {
    let xml = document(&[folder(2, 3, "a", None), folder(3, 2, "b", None)]);

    let outcome = import(&xml);

    assert_eq!(child_names(&outcome.root), vec!["a"]);
    assert_eq!(child_names(find(&outcome.root, uuid(2))), vec!["b"]);
    assert_eq!(outcome.report.warnings.len(), 1);
}

#[test]
fn node_ids_are_the_royal_ids() {
    let xml = document(&[folder(2, 1, "cartella", None)]);

    let outcome = import(&xml);

    assert_eq!(outcome.root.id, uuid(1));
    assert_eq!(outcome.root.as_folder().unwrap().children[0].id, uuid(2));
}

#[test]
fn reimporting_gives_the_same_ids() {
    let xml = document(&[folder(2, 1, "cartella", None)]);

    let first = import(&xml);
    let second = import(&xml);

    assert_eq!(first.root.id, second.root.id);
    assert_eq!(
        first.root.as_folder().unwrap().children[0].id,
        second.root.as_folder().unwrap().children[0].id
    );
}

#[test]
fn source_info_records_type_id_and_file_name_only() {
    let xml = document(&[folder(2, 1, "cartella", None)]);
    let opts = ImportOptions {
        source_file_name: "/Users/someone/docs/lavoro.rtsz".to_owned(),
        include_trash: false,
    };

    let outcome = import_bytes(xml.as_bytes(), &opts, &mut NonInteractive).unwrap();

    let source = find(&outcome.root, uuid(2)).source.clone().unwrap();
    assert_eq!(source.system, "royal");
    assert_eq!(source.source_type, "RoyalFolder");
    assert_eq!(source.source_id, Some(id(2)));
    assert_eq!(source.file.as_deref(), Some("lavoro.rtsz"));
    assert_eq!(source.imported_at, outcome.report.imported_at);
}

#[test]
fn the_royal_document_is_the_root_folder() {
    let xml = document(&[]);

    let outcome = import(&xml);

    assert!(outcome.root.is_folder());
    assert_eq!(outcome.root.name, DOCUMENT_NAME);
    assert_eq!(outcome.report.document_name, DOCUMENT_NAME);
}

#[test]
fn document_fields_go_to_extra_except_the_password_hash() {
    let xml = format!(
        "<RTSZDocument><RoyalDocument><ID>{}</ID><Name>Doc</Name>\
         <DocumentType>Workspace</DocumentType><PasswordHash>{FAKE_CIPHERTEXT}</PasswordHash>\
         </RoyalDocument></RTSZDocument>",
        id(1)
    );

    let outcome = import(&xml);

    assert_eq!(outcome.root.extra["royal.DocumentType"], "Workspace");
    assert!(!outcome.root.extra.contains_key("royal.PasswordHash"));
    assert_eq!(
        outcome.report.warnings,
        vec!["documento protetto da password"]
    );
}

#[test]
fn the_trash_and_its_content_are_skipped() {
    let xml = document(&[
        object("RoyalTrash", 2, 1, &[("Name", "Trash")]),
        folder(3, 2, "buttata", None),
        object("RoyalSSHConnection", 4, 3, &[("Name", "vecchia")]),
    ]);

    let outcome = import(&xml);

    assert!(child_names(&outcome.root).is_empty());
    for n in [2, 3, 4] {
        assert_eq!(status(&outcome, n), ItemStatus::Skipped);
    }
    assert_eq!(item(&outcome, 4).unwrap().path, "Doc/Trash/buttata/vecchia");
}

#[test]
fn include_trash_imports_it_into_a_dedicated_folder() {
    let xml = document(&[
        object("RoyalTrash", 2, 1, &[("Name", "Trash")]),
        folder(3, 2, "buttata", None),
    ]);
    let opts = ImportOptions {
        include_trash: true,
        ..options()
    };

    let outcome = import_bytes(xml.as_bytes(), &opts, &mut NonInteractive).unwrap();

    let trash = find(&outcome.root, uuid(2));
    assert_eq!(trash.name, "Cestino (Royal)");
    assert_eq!(child_names(trash), vec!["buttata"]);
    assert_eq!(status(&outcome, 3), ItemStatus::Imported);
}

#[test]
fn an_object_without_id_is_rejected() {
    let xml = document(&["<RoyalFolder><Name>x</Name></RoyalFolder>".to_owned()]);

    let error = import_bytes(xml.as_bytes(), &options(), &mut NonInteractive).unwrap_err();

    assert!(matches!(error, ImportError::InvalidInput { .. }));
}

#[test]
fn duplicate_ids_are_rejected() {
    let xml = document(&[folder(2, 1, "a", None), folder(2, 1, "b", None)]);

    let error = import_bytes(xml.as_bytes(), &options(), &mut NonInteractive).unwrap_err();

    assert!(matches!(error, ImportError::InvalidInput { .. }));
}

#[test]
fn a_document_without_royal_document_is_rejected() {
    let xml = format!("<RTSZDocument>{}</RTSZDocument>", folder(2, 1, "a", None));

    let error = import_bytes(xml.as_bytes(), &options(), &mut NonInteractive).unwrap_err();

    assert!(matches!(error, ImportError::InvalidInput { .. }));
}
