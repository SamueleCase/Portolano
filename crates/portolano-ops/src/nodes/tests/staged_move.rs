//! The move of the editor of the app in the same save as its other changes
//! (DEC-040.2).

use portolano_core::vault::MemoryVault;
use portolano_core::Node;
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
fn a_folder_cannot_move_into_its_descendant() {
    let mut stored = stored(vec![folder("ACME", vec![folder("Produzione", Vec::new())])]);
    let inner = id_of(&stored.doc, "/ACME/Produzione");

    let result = edit_and_move(&mut stored, "/ACME", NodeEdit::default(), Some(inner));

    assert!(
        matches!(
            result,
            Err(OpsError::Core(portolano_core::Error::InvalidMove { .. }))
        ),
        "{result:?}"
    );
    assert_eq!(names(&stored.doc, None), ["ACME"]);
    assert_eq!(names(&stored.reloaded(), None), ["ACME"]);
}

#[test]
fn an_impossible_move_is_refused_for_the_move_and_not_for_the_name() {
    // The destination also holds a node with the same name.
    let mut stored = stored(vec![folder(
        "ACME",
        vec![folder("Produzione", vec![folder("ACME", Vec::new())])],
    )]);
    let inner = id_of(&stored.doc, "/ACME/Produzione");

    let result = edit_and_move(&mut stored, "/ACME", NodeEdit::default(), Some(inner));

    assert!(
        matches!(
            result,
            Err(OpsError::Core(portolano_core::Error::InvalidMove { .. }))
        ),
        "{result:?}"
    );
}

#[test]
fn a_destination_that_is_not_a_folder_is_refused() {
    let mut stored = stored(vec![connection("web01", "ssh"), connection("db01", "ssh")]);
    let db01 = id_of(&stored.doc, "/db01");

    let result = edit_and_move(&mut stored, "/web01", NodeEdit::default(), Some(db01));

    assert!(
        matches!(
            result,
            Err(OpsError::Core(portolano_core::Error::NotAFolder { .. }))
        ),
        "{result:?}"
    );
    assert_eq!(names(&stored.doc, None), ["web01", "db01"]);
}

#[test]
fn a_destination_that_does_not_exist_is_refused() {
    let mut stored = stored(vec![connection("web01", "ssh")]);

    let result = edit_and_move(
        &mut stored,
        "/web01",
        NodeEdit::default(),
        Some(Uuid::from_u128(404)),
    );

    assert!(
        matches!(
            result,
            Err(OpsError::Core(portolano_core::Error::NodeNotFound { .. }))
        ),
        "{result:?}"
    );
    assert_eq!(names(&stored.doc, None), ["web01"]);
}

/// Folders nested `levels` deep, the innermost named "fondo".
fn nested_folders(levels: usize) -> Node {
    (1..levels).fold(folder("fondo", Vec::new()), |inner, level| {
        folder(&format!("livello {level}"), vec![inner])
    })
}

#[test]
fn a_move_past_the_maximum_depth_is_refused() {
    let mut stored = stored(vec![
        nested_folders(portolano_core::MAX_DEPTH),
        connection("web01", "ssh"),
    ]);
    let deepest = stored
        .doc
        .walk()
        .find(|(_, node)| node.name == "fondo")
        .map(|(_, node)| node.id)
        .unwrap();

    let result = edit_and_move(&mut stored, "/web01", NodeEdit::default(), Some(deepest));

    assert!(
        matches!(
            result,
            Err(OpsError::Core(portolano_core::Error::TooDeep { .. }))
        ),
        "{result:?}"
    );
    assert_eq!(
        names(&stored.reloaded(), Some(deepest)),
        Vec::<String>::new()
    );
}

#[test]
fn the_current_folder_as_destination_keeps_the_order() {
    let mut stored = stored(vec![folder(
        "ACME",
        vec![
            connection("web01", "ssh"),
            connection("web02", "ssh"),
            connection("web03", "ssh"),
        ],
    )]);
    let acme = id_of(&stored.doc, "/ACME");

    let outcome =
        edit_and_move(&mut stored, "/ACME/web01", NodeEdit::default(), Some(acme)).unwrap();

    assert_eq!(outcome.path, "/ACME/web01");
    assert_eq!(
        names(&stored.reloaded(), Some(acme)),
        ["web01", "web02", "web03"]
    );
}
