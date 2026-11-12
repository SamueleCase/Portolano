//! The import wizard (SPEC-UI §4.3 "Import", §5.4): the file is chosen in a
//! native dialog, the preview lists the passwords to ask, the import gets
//! the answers in the same order.

use std::fs;
use std::path::Path;

use portolano_ops::import::{ImportRequest, ImportResult, import_requests, import_with_answers};
use tauri::{AppHandle, Runtime};

use super::{blocking, changed};
use crate::dialogs;
use crate::dto::{ChosenFile, ImportAnswerInput, ImportOptionsInput, ImportPreviewView, file_name};
use crate::error::IpcError;
use crate::state::{AppState, FileKind};

/// Asks for the file to import with the native dialog; `None` if the user
/// cancels.
#[tauri::command]
#[specta::specta]
pub async fn choose_import_file<R: Runtime>(
    app: AppHandle<R>,
) -> Result<Option<ChosenFile>, IpcError> {
    blocking(app, |app, state| {
        let path = dialogs::import(app).blocking_pick_file();
        Ok(dialogs::remember(state, FileKind::Import, path))
    })
    .await
}

/// What importing the chosen file with `options` would do, and the
/// passwords it would ask; nothing is written.
#[tauri::command]
#[specta::specta]
pub async fn import_preview<R: Runtime>(
    app: AppHandle<R>,
    options: ImportOptionsInput,
) -> Result<ImportPreviewView, IpcError> {
    blocking(app, move |_, state| preview(state, options)).await
}

/// Imports the chosen file with `options`, answering the password requests
/// of the preview with `answers`, in their order.
#[tauri::command]
#[specta::specta]
pub async fn import_apply<R: Runtime>(
    app: AppHandle<R>,
    options: ImportOptionsInput,
    answers: Vec<ImportAnswerInput>,
) -> Result<ImportResult, IpcError> {
    blocking(app, move |app, state| {
        changed(app, state, apply(state, options, answers))
    })
    .await
}

/// Saves the report of the last import as JSON where the user chooses with
/// the native dialog; `None` if the user cancels.
#[tauri::command]
#[specta::specta]
pub async fn save_import_report<R: Runtime>(
    app: AppHandle<R>,
) -> Result<Option<ChosenFile>, IpcError> {
    blocking(app, |app, state| {
        let report = state
            .last_import_report()
            .ok_or_else(|| IpcError::not_found("nessun import da esportare".to_owned()))?;
        let Some(path) = dialogs::local_path(dialogs::report(app).blocking_save_file()) else {
            return Ok(None);
        };
        write_report(&path, &report)?;
        Ok(Some(ChosenFile::of(&path)))
    })
    .await
}

pub(crate) fn preview(
    state: &AppState,
    options: ImportOptionsInput,
) -> Result<ImportPreviewView, IpcError> {
    let (path, input) = chosen_input(state)?;
    state.with_document(|open| {
        let preview = import_requests(
            open.document(),
            state.importers(),
            request(&path, &input, options),
        )?;
        Ok(ImportPreviewView {
            result: preview.result,
            requests: preview.requests.into_iter().map(Into::into).collect(),
        })
    })
}

pub(crate) fn apply(
    state: &AppState,
    options: ImportOptionsInput,
    answers: Vec<ImportAnswerInput>,
) -> Result<ImportResult, IpcError> {
    let (path, input) = chosen_input(state)?;
    let vault = state.vault()?;
    let result = state.with_document(|open| {
        let (store, doc) = open.parts();
        Ok(import_with_answers(
            store,
            doc,
            vault.as_ref(),
            state.importers(),
            request(&path, &input, options),
            answers.into_iter().map(Into::into).collect(),
        )?)
    })?;
    state.set_last_import_report(result.report.clone());
    Ok(result)
}

/// The chosen file and its content.
fn chosen_input(state: &AppState) -> Result<(std::path::PathBuf, Vec<u8>), IpcError> {
    let path = state
        .chosen()
        .get(FileKind::Import)
        .ok_or_else(|| IpcError::not_found("nessun file da importare scelto".to_owned()))?;
    let input = fs::read(&path).map_err(|error| IpcError::io(&path, &error))?;
    Ok((path, input))
}

fn request<'a>(path: &Path, input: &'a [u8], options: ImportOptionsInput) -> ImportRequest<'a> {
    ImportRequest {
        input,
        file_name: file_name(path),
        format: None,
        into: options.into,
        on_conflict: options.on_conflict.into(),
        include_trash: options.include_trash,
        dry_run: false,
    }
}

/// Writes `report` as indented JSON; it holds no secret (D3).
pub(crate) fn write_report(
    path: &Path,
    report: &portolano_import::ImportReport,
) -> Result<(), IpcError> {
    let text = serde_json::to_string_pretty(report)
        .map_err(|error| IpcError::internal(&format!("report non serializzabile: {error}")))?;
    fs::write(path, text + "\n").map_err(|error| IpcError::io(path, &error))
}

#[cfg(test)]
mod tests;
