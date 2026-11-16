//! The move of the editor of the app in the same save as its other changes
//! (DEC-040.2).

use portolano_core::vault::MemoryVault;
use pretty_assertions::assert_eq;

use super::*;

fn edit_and_move(
    stored: &mut Stored,
    path: &str,
    edit: NodeEdit,
    parent: Option<Uuid>,
) -> Result<EditOutcome> {
    let id = id_of(&stored.doc, path);
    edit_node_staged(
        &stored.store,
        &mut stored.doc,
        &MemoryVault::default(),
        &Registry::with_defaults(),
        id,
        edit,
        EditorChanges {
            parent: Some(parent),
            ..EditorChanges::default()
        },
    )
}

fn names(doc: &Document, parent: Option<Uuid>) -> Vec<String> {
    doc.children(parent)
        .unwrap()
        .iter()
        .map(|node| node.name.clone())
        .collect()
}

fn rename(name: &str) -> NodeEdit {
    NodeEdit {
        name: Some(name.to_owned()),
        ..NodeEdit::default()
    }
}

#[test]
fn rename_and_move_check_the_new_name_in_the_new_folder() {
    let mut stored = stored(vec![
        connection("web01", "ssh"),
        folder("ACME", vec![connection("web01", "ssh")]),
    ]);
    let acme = id_of(&stored.doc, "/ACME");

    let outcome = edit_and_move(&mut stored, "/web01", rename("web02"), Some(acme)).unwrap();

    assert_eq!(outcome.path, "/ACME/web02");
    assert_eq!(names(&stored.reloaded(), Some(acme)), ["web01", "web02"]);
    assert_eq!(names(&stored.reloaded(), None), ["ACME"]);
}

#[test]
fn moving_into_a_folder_with_the_same_name_is_refused() {
    let mut stored = stored(vec![
        connection("web01", "ssh"),
        folder("ACME", vec![connection("web01", "ssh")]),
    ]);
    let acme = id_of(&stored.doc, "/ACME");

    let result = edit_and_move(&mut stored, "/web01", NodeEdit::default(), Some(acme));

    assert!(
        matches!(result, Err(OpsError::NameTaken { .. })),
        "{result:?}"
    );
    assert_eq!(names(&stored.doc, None), ["web01", "ACME"]);
}

#[test]
fn a_refused_change_moves_nothing() {
    let mut stored = stored(vec![connection("web01", "ssh"), folder("ACME", Vec::new())]);
    let acme = id_of(&stored.doc, "/ACME");
    let invalid = NodeEdit {
        settings_set: vec![("compression".to_owned(), serde_json::json!("sì"))],
        ..NodeEdit::default()
    };

    let result = edit_and_move(&mut stored, "/web01", invalid, Some(acme));

    assert!(
        matches!(result, Err(OpsError::InvalidSettings { .. })),
        "{result:?}"
    );
    assert_eq!(names(&stored.doc, None), ["web01", "ACME"]);
    assert_eq!(names(&stored.reloaded(), Some(acme)), Vec::<String>::new());
}

#[test]
fn a_node_moves_to_the_root() {
    let mut stored = stored(vec![folder("ACME", vec![connection("web01", "ssh")])]);

    let outcome = edit_and_move(&mut stored, "/ACME/web01", NodeEdit::default(), None).unwrap();

    assert_eq!(outcome.path, "/web01");
    assert_eq!(names(&stored.reloaded(), None), ["ACME", "web01"]);
}

#[test]
fn a_folder_cannot_move_inside_itself() {
    let mut stored = stored(vec![folder("ACME", vec![folder("Produzione", Vec::new())])]);
    let inner = id_of(&stored.doc, "/ACME/Produzione");

    let result = edit_and_move(&mut stored, "/ACME", NodeEdit::default(), Some(inner));

    assert!(result.is_err());
    assert_eq!(names(&stored.reloaded(), None), ["ACME"]);
}
