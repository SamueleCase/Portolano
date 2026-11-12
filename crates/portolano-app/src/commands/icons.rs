//! The icons of the editor (SPEC-UI §4.3 "Icone", §4.7): the builtin set,
//! the images of the document, and a new image chosen with the native
//! dialog.

use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use portolano_core::icons::MAX_ICON_BYTES;
use portolano_ops::OpsError;
use tauri::{AppHandle, Runtime};

use super::blocking;
use crate::builtin_icons::BUILTIN_ICONS;
use crate::dialogs;
use crate::dto::{ChosenFile, IconView};
use crate::error::IpcError;
use crate::state::{AppState, FileKind};

/// The names of the builtin icons (DEC-029.4).
#[tauri::command]
#[specta::specta]
pub async fn builtin_icons() -> Result<Vec<String>, IpcError> {
    Ok(BUILTIN_ICONS
        .iter()
        .map(|name| (*name).to_owned())
        .collect())
}

/// The images stored with the open document.
#[tauri::command]
#[specta::specta]
pub async fn custom_icons<R: Runtime>(app: AppHandle<R>) -> Result<Vec<IconView>, IpcError> {
    blocking(app, |_, state| custom(state)).await
}

/// Asks for an image with the native dialog; `None` if the user cancels.
#[tauri::command]
#[specta::specta]
pub async fn choose_icon_file<R: Runtime>(
    app: AppHandle<R>,
) -> Result<Option<ChosenFile>, IpcError> {
    blocking(app, |app, state| {
        let path = dialogs::icon(app).blocking_pick_file();
        Ok(dialogs::remember(state, FileKind::Icon, path))
    })
    .await
}

/// Stores the chosen image with the document, to choose it in the editor.
#[tauri::command]
#[specta::specta]
pub async fn add_icon<R: Runtime>(app: AppHandle<R>) -> Result<IconView, IpcError> {
    blocking(app, |_, state| add(state)).await
}

pub(crate) fn custom(state: &AppState) -> Result<Vec<IconView>, IpcError> {
    let icons = state.icons().ok_or_else(IpcError::no_document)?;
    Ok(icons
        .list()
        .map_err(OpsError::from)?
        .iter()
        .filter_map(IconView::of)
        .collect())
}

pub(crate) fn add(state: &AppState) -> Result<IconView, IpcError> {
    let path = state
        .chosen()
        .get(FileKind::Icon)
        .ok_or_else(|| IpcError::not_found("nessuna immagine scelta".to_owned()))?;
    let icons = state.icons().ok_or_else(IpcError::no_document)?;
    let bytes = read_bounded(&path)?;
    let icon = icons
        .add(&bytes, "application/octet-stream")
        .map_err(OpsError::from)?;
    IconView::of(&icon).ok_or_else(|| IpcError::internal("icona memorizzata non riconosciuta"))
}

/// The bytes of `path`, reading at most one byte more than the largest
/// icon, so that a huge file is refused without being read whole.
fn read_bounded(path: &Path) -> Result<Vec<u8>, IpcError> {
    let io_error = |error| IpcError::io(path, &error);
    let size = fs::metadata(path).map_err(io_error)?.len();
    if size > MAX_ICON_BYTES {
        return Err(OpsError::Core(portolano_core::Error::IconTooLarge {
            size,
            max: MAX_ICON_BYTES,
        })
        .into());
    }
    let mut bytes = Vec::new();
    File::open(path)
        .and_then(|file| file.take(MAX_ICON_BYTES + 1).read_to_end(&mut bytes))
        .map_err(io_error)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests;
