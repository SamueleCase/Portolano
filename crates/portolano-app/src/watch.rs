//! Watching the open document file (SPEC-UI §4.4): `notify` on its
//! directory, since saves replace the file by renaming another one onto
//! it; changes are gathered for 400 ms, then `Store::changed_on_disk` tells
//! the saves of the app from those of others (the CLI, an editor, git).

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::Duration;

use notify_debouncer_full::notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_full::{DebounceEventResult, Debouncer, RecommendedCache, new_debouncer};

use crate::error::IpcError;
use crate::state::AppState;

/// How long changes are gathered before the file is checked.
const DEBOUNCE: Duration = Duration::from_millis(400);

/// What changed in the document file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileChange {
    /// Another process changed it.
    ModifiedExternally,
    /// It no longer exists.
    Missing,
}

/// Watches one document file while it lives.
pub struct DocumentWatcher {
    _debouncer: Debouncer<RecommendedWatcher, RecommendedCache>,
}

impl DocumentWatcher {
    /// Calls `on_event` on a thread of the watcher each time something
    /// happens to `document` or to its directory, at most once per
    /// [`DEBOUNCE`].
    ///
    /// # Errors
    /// The directory cannot be watched.
    pub fn start(document: &Path, on_event: impl Fn() + Send + 'static) -> Result<Self, IpcError> {
        let directory = document
            .parent()
            .filter(|dir| !dir.as_os_str().is_empty())
            .map_or_else(|| PathBuf::from("."), Path::to_owned);
        let file_name: Option<OsString> = document.file_name().map(ToOwned::to_owned);
        let watched = directory.clone();
        let mut debouncer =
            new_debouncer(
                DEBOUNCE,
                None,
                move |result: DebounceEventResult| match result {
                    Ok(events) => {
                        let concerned = events.iter().flat_map(|event| &event.paths).any(|path| {
                            *path == watched || path.file_name() == file_name.as_deref()
                        });
                        if concerned {
                            on_event();
                        }
                    }
                    Err(errors) => {
                        for error in errors {
                            tracing::warn!(%error, "osservazione del documento");
                        }
                    }
                },
            )
            .map_err(|error| watch_error(&directory, &error))?;
        debouncer
            .watch(&directory, RecursiveMode::NonRecursive)
            .map_err(|error| watch_error(&directory, &error))?;
        Ok(Self {
            _debouncer: debouncer,
        })
    }
}

fn watch_error(directory: &Path, error: &notify_debouncer_full::notify::Error) -> IpcError {
    IpcError::internal(&format!(
        "impossibile osservare la cartella '{}': {error}",
        directory.display()
    ))
}

/// What happened to the document open in `state`, if anything that the
/// frontend must know: a change made by the app itself is not one.
pub fn check_document(state: &AppState) -> Option<FileChange> {
    let checked = state.with_document(|open| Ok(open.store().changed_on_disk()));
    match checked {
        Ok(Ok(true)) => Some(FileChange::ModifiedExternally),
        Ok(Ok(false)) | Err(_) => None,
        Ok(Err(portolano_core::Error::DocumentMissing { .. })) => Some(FileChange::Missing),
        Ok(Err(error)) => {
            tracing::warn!(%error, "controllo del documento non riuscito");
            None
        }
    }
}

#[cfg(test)]
mod tests;
