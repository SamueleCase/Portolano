//! Commands about the document file (SPEC-UI §4.3 "Documento").

use std::path::{Path, PathBuf};

use portolano_ops::document::{init_document, open_document as open_file};
use serde::Deserialize;
use tauri::{AppHandle, Runtime};

use super::blocking;
use crate::dto::{AppStatus, ChosenFile, DocumentInfo, DocumentLocation, VaultKind};
use crate::error::IpcError;
use crate::events::{DocumentChanged, DocumentMissing, DocumentModifiedExternally, emit};
use crate::state::{AppState, FileKind, OpenDocument};
use crate::watch::{DocumentWatcher, FileChange, check_document};
use crate::{dialogs, menu};

/// Which document to open.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, specta::Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum DocumentSource {
    /// The file chosen with `choose_document` (or `choose_new_document` to
    /// create one).
    Chosen,
    /// The default document of the configuration.
    Default,
    /// A recent document, as `recent_documents` lists it.
    Recent {
        /// Its path.
        path: String,
    },
}

/// The documents, the vault backend and the preferences of the interface.
#[tauri::command]
#[specta::specta]
pub async fn app_status<R: Runtime>(app: AppHandle<R>) -> Result<AppStatus, IpcError> {
    blocking(app, |_, state| status(state)).await
}

/// Asks for a document to open with the native dialog; `None` if the user
/// cancels.
#[tauri::command]
#[specta::specta]
pub async fn choose_document<R: Runtime>(
    app: AppHandle<R>,
) -> Result<Option<ChosenFile>, IpcError> {
    blocking(app, |app, state| {
        let path = dialogs::document(app).blocking_pick_file();
        Ok(dialogs::remember(state, FileKind::Document, path))
    })
    .await
}

/// Asks where to create a new document with the native dialog; `None` if
/// the user cancels.
#[tauri::command]
#[specta::specta]
pub async fn choose_new_document<R: Runtime>(
    app: AppHandle<R>,
) -> Result<Option<ChosenFile>, IpcError> {
    blocking(app, |app, state| {
        let path = dialogs::document(app)
            .set_file_name(portolano_core::config::DOCUMENT_FILE_NAME)
            .blocking_save_file();
        Ok(dialogs::remember(state, FileKind::NewDocument, path))
    })
    .await
}

/// Opens a document; another open document is closed first.
#[tauri::command]
#[specta::specta]
pub async fn open_document<R: Runtime>(
    app: AppHandle<R>,
    source: DocumentSource,
) -> Result<DocumentInfo, IpcError> {
    blocking(app, move |app, state| {
        let path = source_path(state, &source, FileKind::Document)?;
        let info = open(state, &path)?;
        after_open(app, state, &path);
        Ok(info)
    })
    .await
}

/// Creates an empty document named `name` and opens it.
#[tauri::command]
#[specta::specta]
pub async fn create_document<R: Runtime>(
    app: AppHandle<R>,
    source: DocumentSource,
    name: String,
) -> Result<DocumentInfo, IpcError> {
    blocking(app, move |app, state| {
        let path = source_path(state, &source, FileKind::NewDocument)?;
        let info = create(state, &path, &name)?;
        after_open(app, state, &path);
        Ok(info)
    })
    .await
}

/// The documents opened last, most recent first.
#[tauri::command]
#[specta::specta]
pub async fn recent_documents<R: Runtime>(
    app: AppHandle<R>,
) -> Result<Vec<DocumentLocation>, IpcError> {
    blocking(app, |_, state| Ok(recents(state))).await
}

/// Reads the open document from its file again, dropping what the app had
/// in memory (after "Ricarica").
#[tauri::command]
#[specta::specta]
pub async fn reload_document<R: Runtime>(app: AppHandle<R>) -> Result<DocumentInfo, IpcError> {
    blocking(app, |app, state| {
        let info = reload(state)?;
        emit(
            app,
            DocumentChanged {
                file_name: info.file_name.clone(),
            },
        );
        Ok(info)
    })
    .await
}

/// Closes the open document; the deletions that could still be undone are
/// finished.
#[tauri::command]
#[specta::specta]
pub async fn close_document<R: Runtime>(app: AppHandle<R>) -> Result<(), IpcError> {
    blocking(app, |_, state| {
        close(state);
        Ok(())
    })
    .await
}

/// The status of the app.
pub(crate) fn status(state: &AppState) -> Result<AppStatus, IpcError> {
    let locations = state.locations()?;
    let preferences = state.ui_preferences();
    let document = state
        .with_document(|open| Ok(DocumentInfo::new(&open.document().name, open.path())))
        .ok();
    Ok(AppStatus {
        default_document: DocumentLocation::of(&locations.document),
        vault: VaultKind::from(&locations.vault),
        document,
        last_document: preferences.last_document().map(DocumentLocation::of),
        preferences: preferences.interface,
    })
}

/// The file `source` names. The frontend never gives a path of its own: a
/// recent document must be one of the list.
pub(crate) fn source_path(
    state: &AppState,
    source: &DocumentSource,
    chosen: FileKind,
) -> Result<PathBuf, IpcError> {
    match source {
        DocumentSource::Chosen => state
            .chosen()
            .get(chosen)
            .ok_or_else(|| IpcError::not_found("nessun file scelto".to_owned())),
        DocumentSource::Default => Ok(state.locations()?.document),
        DocumentSource::Recent { path } => state
            .ui_preferences()
            .recent_documents
            .into_iter()
            .find(|recent| recent.display().to_string() == *path)
            .ok_or_else(|| IpcError::not_found(format!("'{path}' non è fra i documenti recenti"))),
    }
}

/// Opens the document at `path`, closing the previous one.
pub(crate) fn open(state: &AppState, path: &Path) -> Result<DocumentInfo, IpcError> {
    let (store, document) = open_file(path)?;
    replace(state, OpenDocument::new(path.to_owned(), store, document))
}

/// Creates an empty document named `name` at `path` and opens it.
pub(crate) fn create(state: &AppState, path: &Path, name: &str) -> Result<DocumentInfo, IpcError> {
    let (store, document) = init_document(path, name, false)?;
    replace(state, OpenDocument::new(path.to_owned(), store, document))
}

fn replace(state: &AppState, open: OpenDocument) -> Result<DocumentInfo, IpcError> {
    let info = DocumentInfo::new(&open.document().name, open.path());
    let path = open.path().to_owned();
    close(state);
    state.replace_document(Some(open));
    // The document is open even if the list of recent documents cannot be
    // saved: it only matters at the next start.
    if let Err(error) =
        state.update_ui_preferences(|preferences| preferences.remember_document(&path))
    {
        tracing::warn!("{}", error.message);
    }
    Ok(info)
}

/// Reads the open document again.
pub(crate) fn reload(state: &AppState) -> Result<DocumentInfo, IpcError> {
    state.with_document(|open| {
        open.reload().map_err(portolano_ops::OpsError::from)?;
        Ok(DocumentInfo::new(&open.document().name, open.path()))
    })
}

/// Closes the open document, if any.
pub(crate) fn close(state: &AppState) {
    state.purge_pending_deletes();
    state.set_watcher(None);
    state.replace_document(None);
}

/// The recent documents and whether they still exist.
pub(crate) fn recents(state: &AppState) -> Vec<DocumentLocation> {
    state
        .ui_preferences()
        .recent_documents
        .iter()
        .map(|path| DocumentLocation::of(path))
        .collect()
}

/// Watches the file just opened and lists it in the menu.
fn after_open<R: Runtime>(app: &AppHandle<R>, state: &AppState, path: &Path) {
    let handle = app.clone();
    match DocumentWatcher::start(path, move || notify_change(&handle)) {
        Ok(watcher) => state.set_watcher(Some(watcher)),
        // The app works without it: changes made by others are then found
        // when saving (`ModifiedExternally`).
        Err(error) => tracing::warn!("{}", error.message),
    }
    // Only the menu installed at startup is updated (a test app has none).
    if app.menu().is_some()
        && let Err(error) = menu::install(app, &state.ui_preferences().recent_documents)
    {
        tracing::warn!(%error, "menu non aggiornato");
    }
}

/// Tells the frontend what happened to the open document.
fn notify_change<R: Runtime>(app: &AppHandle<R>) {
    use tauri::Manager;
    let state = app.state::<AppState>();
    let Some(path) = state.document_path() else {
        return;
    };
    let file_name = crate::dto::file_name(&path);
    match check_document(&state) {
        Some(FileChange::ModifiedExternally) => emit(app, DocumentModifiedExternally { file_name }),
        Some(FileChange::Missing) => emit(app, DocumentMissing { file_name }),
        None => {}
    }
}

#[cfg(test)]
mod tests;
