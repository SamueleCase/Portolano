use portolano_core::Vault;
use pretty_assertions::assert_eq;
use serde_json::json;

use super::*;
use crate::commands::test_support::{SENTINEL, TestApp, admin_with_sentinel, ssh};
use crate::commands::tree::{details, tree_of};
use crate::error::ErrorCode;

fn edit_of(json: serde_json::Value) -> NodeEditInput {
    serde_json::from_value(json).unwrap()
}

fn names(app: &TestApp) -> Vec<String> {
    tree_of(&app.state)
        .unwrap()
        .nodes
        .into_iter()
        .map(|node| node.name)
        .collect()
}

#[test]
fn folder_created_with_a_credential_passes_it_to_its_children() {
    let app = TestApp::with_document();
    let admin = app.add(admin_with_sentinel());

    let prod = app.add(json!({
        "parent": null,
        "name": "Prod",
        "kind": { "type": "folder", "credential": { "mode": "byId", "id": admin } },
    }));
    let web01 = app.add(ssh("web01", Some(prod)));

    let detail = details(&app.state, web01).unwrap();
    assert!(matches!(
        detail.trace.unwrap().end,
        portolano_ops::trace::TraceEnd::Credential { node } if node.id == admin
    ));
}

#[test]
fn name_taken_is_reported_on_the_name_field() {
    let app = TestApp::with_document();
    app.add(ssh("web01", None));

    let input: NewNodeInput = serde_json::from_value(ssh("web01", None)).unwrap();
    let error = create(&app.state, input).unwrap_err();

    assert_eq!(error.code, ErrorCode::Invalid);
    assert_eq!(
        error.details,
        Some(crate::error::IpcErrorDetails::Invalid {
            field: "name".to_owned()
        })
    );
}

#[test]
fn staged_password_replaces_the_old_one_on_save() {
    let app = TestApp::with_document();
    let admin = app.add(admin_with_sentinel());
    let vault = app.state.vault().unwrap();
    let old = app
        .state
        .with_document(|open| Ok(open.document().secret_refs_in(admin)))
        .unwrap();

    change(
        &app.state,
        admin,
        edit_of(
            json!({ "secrets": { "password": { "action": "set", "value": "sentinel-nuova" } } }),
        ),
    )
    .unwrap();

    assert!(!vault.contains(&old[0]).unwrap());
    let new = app
        .state
        .with_document(|open| Ok(open.document().secret_refs_in(admin)))
        .unwrap();
    assert_eq!(
        vault.get(&new[0]).unwrap().unwrap().expose(),
        "sentinel-nuova"
    );
}

#[test]
fn edit_replaces_tags_and_text_properties() {
    let app = TestApp::with_document();
    let web01 = app.add(ssh("web01", None));
    change(
        &app.state,
        web01,
        edit_of(json!({ "details": { "tags": ["a", "b"], "properties": [{ "name": "rack", "value": "R1" }] } })),
    )
    .unwrap();

    change(
        &app.state,
        web01,
        edit_of(json!({ "details": { "tags": ["b", "c"], "properties": [] } })),
    )
    .unwrap();

    let detail = details(&app.state, web01).unwrap();
    assert_eq!(detail.tags, vec!["b", "c"]);
    assert!(detail.properties.is_empty());
}

#[test]
fn inline_choice_on_an_inline_credential_keeps_its_password() {
    let app = TestApp::with_document();
    let web01 = app.add(json!({
        "parent": null,
        "name": "web01",
        "kind": {
            "type": "connection", "protocol": "ssh", "host": "web01.example.com", "port": null,
            "credential": { "mode": "inline", "username": "ops", "domain": null }, "settings": {},
        },
        "secrets": { "password": SENTINEL },
    }));

    change(
        &app.state,
        web01,
        edit_of(json!({ "credential": { "mode": "inline", "username": "root", "domain": null } })),
    )
    .unwrap();

    let detail = details(&app.state, web01).unwrap();
    assert_eq!(detail.secrets.len(), 1);
    assert_eq!(
        detail.credential_spec,
        Some(portolano_ops::trace::SpecView::Inline {
            username: Some("root".to_owned()),
            domain: None
        })
    );
}

#[test]
fn deleted_node_comes_back_with_undo() {
    let app = TestApp::with_document();
    app.add(ssh("web01", None));
    let web02 = app.add(ssh("web02", None));

    let deleted = remove(&app.state, web02, DeleteOptionsInput::default()).unwrap();
    assert_eq!(names(&app), vec!["web01"]);
    let restored = undo(&app.state, deleted.undo_id).unwrap();

    assert_eq!(restored.path, "/web02");
    assert_eq!(names(&app), vec!["web01", "web02"]);
}

#[test]
fn undo_after_the_deletion_is_finished_is_refused() {
    let app = TestApp::with_document();
    let web01 = app.add(ssh("web01", None));
    let deleted = remove(&app.state, web01, DeleteOptionsInput::default()).unwrap();
    let pending = app.state.pending_deletes().take(deleted.undo_id).unwrap();
    app.state.purge(pending);

    let error = undo(&app.state, deleted.undo_id).unwrap_err();

    assert_eq!(error.code, ErrorCode::NotFound);
}

#[test]
fn finished_deletion_removes_the_secrets_from_the_vault() {
    let app = TestApp::with_document();
    let admin = app.add(admin_with_sentinel());
    let reference = app
        .state
        .with_document(|open| Ok(open.document().secret_refs_in(admin)))
        .unwrap();
    let vault = app.state.vault().unwrap();
    let deleted = remove(&app.state, admin, DeleteOptionsInput::default()).unwrap();
    assert!(vault.contains(&reference[0]).unwrap());

    let pending = app.state.pending_deletes().take(deleted.undo_id).unwrap();
    app.state.purge(pending);

    assert!(!vault.contains(&reference[0]).unwrap());
}

#[test]
fn closing_the_document_finishes_the_pending_deletions() {
    let app = TestApp::with_document();
    let admin = app.add(admin_with_sentinel());
    let reference = app
        .state
        .with_document(|open| Ok(open.document().secret_refs_in(admin)))
        .unwrap();
    remove(&app.state, admin, DeleteOptionsInput::default()).unwrap();

    crate::commands::document::close(&app.state);

    assert!(!app.state.vault().unwrap().contains(&reference[0]).unwrap());
}

#[test]
fn preview_counts_the_connections_that_inherit_a_deleted_credential() {
    let app = TestApp::with_document();
    let admin = app.add(admin_with_sentinel());
    let prod = app.add(json!({
        "parent": null,
        "name": "Prod",
        "kind": { "type": "folder", "credential": { "mode": "byId", "id": admin } },
    }));
    let web01 = app.add(ssh("web01", Some(prod)));

    let view = app
        .state
        .with_document(|open| preview_of(open.document(), admin))
        .unwrap();

    assert_eq!(view.preview.references.len(), 1);
    assert_eq!(
        view.inherited
            .iter()
            .map(|node| node.id)
            .collect::<Vec<_>>(),
        vec![web01]
    );
}

#[test]
fn referenced_credential_is_not_deleted_without_force() {
    let app = TestApp::with_document();
    let admin = app.add(admin_with_sentinel());
    app.add(json!({
        "parent": null,
        "name": "Prod",
        "kind": { "type": "folder", "credential": { "mode": "byId", "id": admin } },
    }));

    let error = remove(&app.state, admin, DeleteOptionsInput::default()).unwrap_err();

    assert_eq!(error.code, ErrorCode::Referenced);
}

#[test]
fn undo_refused_by_a_taken_name_stays_undoable() {
    let app = TestApp::with_document();
    let web01 = app.add(ssh("web01", None));
    let deleted = remove(&app.state, web01, DeleteOptionsInput::default()).unwrap();
    let other = app.add(ssh("web01", None));

    let error = undo(&app.state, deleted.undo_id).unwrap_err();
    change(&app.state, other, edit_of(json!({ "name": "web02" }))).unwrap();
    let restored = undo(&app.state, deleted.undo_id).unwrap();

    assert_eq!(error.code, ErrorCode::Invalid);
    assert_eq!(restored.path, "/web01");
}

#[test]
fn undo_refused_by_a_change_on_disk_stays_undoable_after_a_reload() {
    let app = TestApp::with_document();
    let web01 = app.add(ssh("web01", None));
    let deleted = remove(&app.state, web01, DeleteOptionsInput::default()).unwrap();
    let store = portolano_core::Store::open(app.document_path()).unwrap();
    let mut doc = store.load().unwrap();
    doc.name = "Cambiato dalla CLI".to_owned();
    store.save(&mut doc).unwrap();

    let error = undo(&app.state, deleted.undo_id).unwrap_err();
    crate::commands::document::reload(&app.state).unwrap();
    let restored = undo(&app.state, deleted.undo_id).unwrap();

    assert_eq!(error.code, ErrorCode::ModifiedExternally);
    assert_eq!(restored.path, "/web01");
}
