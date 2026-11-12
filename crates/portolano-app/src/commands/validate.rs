//! The validation of the document (SPEC-UI §4.3, §5.4 "Validazione").

use portolano_core::{Issue, Severity};
use portolano_ops::ErrorCategory;
use portolano_ops::validate::validate as validate_document;
use tauri::{AppHandle, Runtime};

use super::blocking;
use crate::error::IpcError;
use crate::state::AppState;

/// Code of the note that replaces the checks of the vault while it is
/// locked.
pub const VAULT_LOCKED: &str = "vault_locked";

/// Every problem of the document: the core, the protocols and the secrets
/// missing from the vault (DEC-022.4). With the vault locked, a single note
/// replaces the `missing_secret` of every secret (SPEC-UI §5.4).
#[tauri::command]
#[specta::specta]
pub async fn validate<R: Runtime>(app: AppHandle<R>) -> Result<Vec<Issue>, IpcError> {
    blocking(app, |_, state| issues(state)).await
}

pub(crate) fn issues(state: &AppState) -> Result<Vec<Issue>, IpcError> {
    let vault = state.vault()?;
    state.with_document(|open| {
        let doc = open.document();
        match validate_document(doc, state.protocols(), Some(vault.as_ref())) {
            Ok(issues) => Ok(issues),
            Err(error) if error.category() == ErrorCategory::VaultLocked => {
                let mut issues = validate_document(doc, state.protocols(), None)?;
                issues.push(Issue {
                    severity: Severity::Info,
                    node: None,
                    code: VAULT_LOCKED,
                    message: "il vault è bloccato: la presenza dei segreti non è stata controllata"
                        .to_owned(),
                });
                Ok(issues)
            }
            Err(error) => Err(error.into()),
        }
    })
}
