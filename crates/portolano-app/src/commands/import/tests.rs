use pretty_assertions::assert_eq;

use super::*;
use crate::commands::test_support::{SENTINEL, TestApp};
use crate::dto::SecretText;
use crate::error::ErrorCode;

const WORKSPACE: &[u8] =
    include_bytes!("../../../../portolano-import/tests/fixtures/workspace.royal.xml");

/// An app with the Royal fixture chosen as the file to import.
fn with_import_file() -> TestApp {
    let app = TestApp::with_document();
    let path = app.dir.path().join("infra.rtsx");
    fs::write(&path, WORKSPACE).unwrap();
    app.state.chosen().set(FileKind::Import, &path);
    app
}

fn answers_for(preview: &ImportPreviewView, value: Option<&str>) -> Vec<ImportAnswerInput> {
    preview
        .requests
        .iter()
        .map(|request| {
            serde_json::from_value(serde_json::json!({
                "nodePath": request.node_path,
                "field": request.field,
                "value": value,
            }))
            .unwrap()
        })
        .collect()
}

#[test]
fn preview_lists_the_passwords_to_ask_and_writes_nothing() {
    let app = with_import_file();

    let preview = preview(&app.state, ImportOptionsInput::default()).unwrap();

    assert!(!preview.requests.is_empty());
    assert!(preview.result.dry_run);
    let tree = crate::commands::tree::tree_of(&app.state).unwrap();
    assert!(tree.nodes.is_empty());
}

#[test]
fn import_stores_the_answers_and_remembers_the_report() {
    let app = with_import_file();
    let preview = preview(&app.state, ImportOptionsInput::default()).unwrap();
    let answers = answers_for(&preview, Some(SENTINEL));

    let result = apply(&app.state, ImportOptionsInput::default(), answers).unwrap();

    assert_eq!(result.secrets_written, preview.requests.len());
    assert_eq!(app.state.last_import_report(), Some(result.report));
    assert!(!app.files_text().contains(SENTINEL));
}

#[test]
fn answers_that_do_not_match_the_requests_are_refused() {
    let app = with_import_file();

    let error = apply(&app.state, ImportOptionsInput::default(), Vec::new()).unwrap_err();

    assert_eq!(error.code, ErrorCode::Invalid);
}

#[test]
fn import_without_a_chosen_file_is_refused() {
    let app = TestApp::with_document();

    let error = preview(&app.state, ImportOptionsInput::default()).unwrap_err();

    assert_eq!(error.code, ErrorCode::NotFound);
}

#[test]
fn exported_report_is_json_without_secrets() {
    let app = with_import_file();
    let preview = preview(&app.state, ImportOptionsInput::default()).unwrap();
    let result = apply(
        &app.state,
        ImportOptionsInput::default(),
        answers_for(&preview, Some(SENTINEL)),
    )
    .unwrap();
    let path = app.dir.path().join("report.json");

    write_report(&path, &result.report).unwrap();

    let text = fs::read_to_string(&path).unwrap();
    assert!(serde_json::from_str::<serde_json::Value>(&text).is_ok());
    assert!(!text.contains(SENTINEL));
}

#[test]
fn typed_answer_is_a_secret_text() {
    let answer: ImportAnswerInput = serde_json::from_value(serde_json::json!({
        "nodePath": "Royal/web01",
        "field": { "kind": "password" },
        "value": SENTINEL,
    }))
    .unwrap();

    assert!(!format!("{answer:?}").contains(SENTINEL));
    assert!(answer.value.map(SecretText::into_secret).is_some());
}
