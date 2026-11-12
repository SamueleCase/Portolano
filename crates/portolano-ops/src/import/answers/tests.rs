use portolano_core::vault::MemoryVault;
use pretty_assertions::assert_eq;

use super::*;
use crate::import::ConflictPolicy;
use crate::testutil::*;
use crate::tree_util::document_secret_refs;

const WORKSPACE: &[u8] =
    include_bytes!("../../../../portolano-import/tests/fixtures/workspace.royal.xml");

fn request(input: &[u8]) -> ImportRequest<'_> {
    ImportRequest {
        input,
        file_name: "fixture.rtsx".to_owned(),
        format: None,
        into: None,
        on_conflict: ConflictPolicy::Fail,
        include_trash: false,
        dry_run: false,
    }
}

fn preview(doc: &Document) -> ImportPreview {
    import_requests(doc, &Registry::with_defaults(), request(WORKSPACE)).unwrap()
}

/// An answer with the sentinel for every request.
fn typed(requests: &[SecretRequest]) -> Vec<SecretAnswer> {
    requests
        .iter()
        .map(|req| SecretAnswer {
            node_path: req.node_path.clone(),
            field: req.field.clone(),
            secret: Some(secret(SENTINEL)),
        })
        .collect()
}

fn apply(
    stored: &mut Stored,
    vault: &dyn Vault,
    answers: Vec<SecretAnswer>,
) -> Result<ImportResult> {
    import_with_answers(
        &stored.store,
        &mut stored.doc,
        vault,
        &Registry::with_defaults(),
        request(WORKSPACE),
        answers,
    )
}

#[test]
fn requests_are_listed_without_writing_anything() {
    let stored = stored(vec![]);

    let preview = preview(&stored.doc);

    assert!(!preview.requests.is_empty());
    assert!(preview.result.dry_run);
    assert_eq!(preview.result.secrets_written, 0);
    assert!(stored.reloaded().nodes.is_empty());
}

#[test]
fn answers_in_the_order_of_the_requests_are_stored() {
    let mut stored = stored(vec![]);
    let vault = MemoryVault::default();
    let answers = typed(&preview(&stored.doc).requests);
    let expected = answers.len();

    let result = apply(&mut stored, &vault, answers).unwrap();

    assert_eq!(result.secrets_written, expected);
    for reference in document_secret_refs(&stored.doc) {
        assert_eq!(vault.get(&reference).unwrap().unwrap().expose(), SENTINEL);
    }
}

#[test]
fn skipped_answers_store_no_secret() {
    let mut stored = stored(vec![]);
    let mut answers = typed(&preview(&stored.doc).requests);
    for answer in &mut answers {
        answer.secret = None;
    }

    let result = apply(&mut stored, &MemoryVault::default(), answers).unwrap();

    assert_eq!(result.secrets_written, 0);
    assert!(!stored.reloaded().nodes.is_empty());
}

#[test]
fn answer_for_another_node_stops_the_import_before_writing() {
    let mut stored = stored(vec![]);
    let mut answers = typed(&preview(&stored.doc).requests);
    answers[0].node_path = "/Altro/nodo".to_owned();

    let result = apply(&mut stored, &MemoryVault::default(), answers);

    assert!(matches!(
        result,
        Err(OpsError::ImportAnswersMismatch { .. })
    ));
    assert!(stored.reloaded().nodes.is_empty());
}

#[test]
fn missing_answers_stop_the_import() {
    let mut stored = stored(vec![]);
    let mut answers = typed(&preview(&stored.doc).requests);
    answers.pop();

    let result = apply(&mut stored, &MemoryVault::default(), answers);

    assert!(matches!(
        result,
        Err(OpsError::ImportAnswersMismatch { .. })
    ));
}

#[test]
fn answers_left_over_stop_the_import_before_writing() {
    let mut stored = stored(vec![]);
    let mut answers = typed(&preview(&stored.doc).requests);
    answers.push(SecretAnswer {
        node_path: "/in/più".to_owned(),
        field: SecretField::Password,
        secret: None,
    });

    let result = apply(&mut stored, &MemoryVault::default(), answers);

    assert!(matches!(
        result,
        Err(OpsError::ImportAnswersMismatch { path }) if path == "/in/più"
    ));
    assert!(stored.reloaded().nodes.is_empty());
}
