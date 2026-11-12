//! The import of the app in two phases (SPEC-UI §3.6, §5.4): the wizard
//! first learns which secrets the import will ask, without writing anything
//! ([`import_requests`]); it asks them one at a time ("N / M", "Salta",
//! "Salta tutte le restanti"); then it imports with the answers
//! ([`import_with_answers`]), which must match the requests one by one, so
//! that a file changed in between cannot put a password on another node.

use std::collections::VecDeque;

use portolano_core::{Document, Secret, Store, Vault};
use portolano_import::{ImportError, Registry, SecretField, SecretPrompter, SecretRequest};

use super::{ImportRequest, ImportResult, commit, prepare};
use crate::{OpsError, Result};

/// What an import would do, and the secrets it would ask.
#[derive(Debug)]
pub struct ImportPreview {
    /// The secret requests, in the order the importer makes them.
    pub requests: Vec<SecretRequest>,
    /// The result of the import with every request skipped; nothing is
    /// written.
    pub result: ImportResult,
}

/// The answer to one secret request.
#[derive(Debug)]
pub struct SecretAnswer {
    /// The node of the request, as [`SecretRequest::node_path`].
    pub node_path: String,
    /// The secret requested.
    pub field: SecretField,
    /// The secret, or `None` to skip it.
    pub secret: Option<Secret>,
}

/// The first phase: the secret requests of importing `request.input` into
/// `doc`, and what the import would do. Nothing is written, whatever
/// `request.dry_run` says.
///
/// # Errors
/// As [`super::import`], without the errors of writing.
pub fn import_requests(
    doc: &Document,
    importers: &Registry,
    request: ImportRequest<'_>,
) -> Result<ImportPreview> {
    let mut recorder = Recorder::default();
    let (_, finish) = prepare(doc, importers, &request, &mut recorder)?;
    let (mut result, _, _) = finish.into_parts();
    result.dry_run = true;
    Ok(ImportPreview {
        requests: recorder.requests,
        result,
    })
}

/// The second phase: imports `request.input` like [`super::import`],
/// answering the secret requests with `answers`, in order.
///
/// # Errors
/// [`OpsError::ImportAnswersMismatch`] if a request does not match its
/// answer by node and field, or if answers are left over; then nothing is
/// written. Otherwise as [`super::import`].
pub fn import_with_answers(
    store: &Store,
    doc: &mut Document,
    vault: &dyn Vault,
    importers: &Registry,
    request: ImportRequest<'_>,
    answers: Vec<SecretAnswer>,
) -> Result<ImportResult> {
    let mut prompter = Answers {
        answers: answers.into(),
        mismatch: None,
    };
    let prepared = prepare(doc, importers, &request, &mut prompter);
    if let Some(path) = prompter.mismatch {
        return Err(OpsError::ImportAnswersMismatch { path });
    }
    let (change, finish) = prepared?;
    if let Some(extra) = prompter.answers.front() {
        return Err(OpsError::ImportAnswersMismatch {
            path: extra.node_path.clone(),
        });
    }
    commit(store, vault, doc, change, finish)
}

/// Records the requests and skips them all.
#[derive(Default)]
struct Recorder {
    requests: Vec<SecretRequest>,
}

impl SecretPrompter for Recorder {
    fn ask(&mut self, req: &SecretRequest) -> Result<Option<Secret>, ImportError> {
        self.requests.push(req.clone());
        Ok(None)
    }
}

/// Answers each request with the next answer, if it is for the same node
/// and field; otherwise stops the import.
struct Answers {
    answers: VecDeque<SecretAnswer>,
    mismatch: Option<String>,
}

impl SecretPrompter for Answers {
    fn ask(&mut self, req: &SecretRequest) -> Result<Option<Secret>, ImportError> {
        match self.answers.pop_front() {
            Some(answer) if answer.node_path == req.node_path && answer.field == req.field => {
                Ok(answer.secret)
            }
            _ => {
                self.mismatch = Some(req.node_path.clone());
                Err(ImportError::Cancelled)
            }
        }
    }
}

#[cfg(test)]
mod tests;
