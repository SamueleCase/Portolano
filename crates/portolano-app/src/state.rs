//! The state shared by the commands (SPEC-UI §4.1).
//!
//! Every lock is taken and released inside one synchronous call: commands
//! run their work on a blocking thread and never hold a guard across an
//! `.await`. A poisoned lock is recovered, as in `LazyVault`: the values it
//! guards are replaced in one assignment (the operations of
//! `portolano-ops` work on a copy and replace the document only on
//! success), so a panic cannot leave them half-written.

mod chosen;
mod open;
mod pending;
mod unlock;

use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError, RwLock};

use portolano_core::{AppConfig, IconStore, VaultSelection};
use portolano_ops::locations::{GlobalOptions, Locations};
use portolano_ops::vault::LazyVault;
use portolano_ops::{OpsError, delete::purge_secrets};

pub use chosen::{ChosenFiles, FileKind};
pub use open::OpenDocument;
pub use pending::{PendingDelete, PendingDeletes, UNDO_WINDOW, UndoId};
pub use unlock::UnlockSlot;

use unlock::SharedSlot;

use crate::error::IpcError;
use crate::prefs::UiPreferences;
use crate::sessions::SessionManager;
use crate::watch::DocumentWatcher;

/// The files of the app besides the documents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    /// `config.toml`, shared with the CLI.
    pub config: PathBuf,
    /// `ui.toml`, the preferences of the interface only (DEC-011).
    pub ui_preferences: PathBuf,
}

/// The vault of the configuration, opened on first use.
struct VaultHandle {
    selection: VaultSelection,
    vault: Arc<LazyVault>,
}

/// The state of the app, managed by Tauri.
pub struct AppState {
    paths: AppPaths,
    open: Mutex<Option<OpenDocument>>,
    vault: Mutex<Option<VaultHandle>>,
    unlock: Arc<UnlockSlot>,
    ui_preferences: RwLock<UiPreferences>,
    protocols: portolano_connectors::Registry,
    importers: portolano_import::Registry,
    watcher: Mutex<Option<DocumentWatcher>>,
    pending_deletes: PendingDeletes,
    chosen: ChosenFiles,
    last_import_report: Mutex<Option<portolano_import::ImportReport>>,
    sessions: SessionManager,
}

impl AppState {
    /// The state of an app without an open document, with its files at
    /// `paths`. Unreadable preferences give way to the defaults, logged:
    /// they only change how the interface looks.
    pub fn new(paths: AppPaths) -> Self {
        let ui_preferences = UiPreferences::load(&paths.ui_preferences).unwrap_or_else(|error| {
            tracing::warn!("{}", error.message);
            UiPreferences::default()
        });
        Self {
            paths,
            open: Mutex::new(None),
            vault: Mutex::new(None),
            unlock: Arc::new(UnlockSlot::default()),
            ui_preferences: RwLock::new(ui_preferences),
            protocols: portolano_connectors::Registry::with_defaults(),
            importers: portolano_import::Registry::with_defaults(),
            watcher: Mutex::new(None),
            pending_deletes: PendingDeletes::default(),
            chosen: ChosenFiles::default(),
            last_import_report: Mutex::new(None),
            sessions: SessionManager::default(),
        }
    }

    /// The files of the app.
    pub fn paths(&self) -> &AppPaths {
        &self.paths
    }

    /// The configuration, read each time so that changes made meanwhile by
    /// the CLI are seen.
    ///
    /// # Errors
    /// An invalid configuration file.
    pub fn config(&self) -> Result<AppConfig, IpcError> {
        Ok(AppConfig::load(&self.paths.config).map_err(OpsError::from)?)
    }

    /// The default document and the vault of the configuration.
    ///
    /// # Errors
    /// An invalid configuration, no home directory.
    pub fn locations(&self) -> Result<Locations, IpcError> {
        Ok(Locations::resolve(
            &GlobalOptions::default(),
            &self.config()?,
        )?)
    }

    /// The registered protocols.
    pub fn protocols(&self) -> &portolano_connectors::Registry {
        &self.protocols
    }

    /// The registered importers.
    pub fn importers(&self) -> &portolano_import::Registry {
        &self.importers
    }

    /// The master password slot of the file vault.
    pub fn unlock_slot(&self) -> &UnlockSlot {
        &self.unlock
    }

    /// The vault of the configuration. When the configuration names another
    /// vault than the last one used (changed by the preferences or by the
    /// CLI), the previous one is dropped, locked, and the new one is opened
    /// on first use.
    ///
    /// # Errors
    /// An invalid configuration.
    pub fn vault(&self) -> Result<Arc<LazyVault>, IpcError> {
        let selection = self.locations()?.vault;
        let mut handle = recover(self.vault.lock());
        if let Some(current) = handle.as_ref().filter(|h| h.selection == selection) {
            return Ok(Arc::clone(&current.vault));
        }
        let vault = Arc::new(LazyVault::new(
            selection.clone(),
            Box::new(SharedSlot(Arc::clone(&self.unlock))),
        ));
        *handle = Some(VaultHandle {
            selection,
            vault: Arc::clone(&vault),
        });
        Ok(vault)
    }

    /// Runs `work` on the open document.
    ///
    /// # Errors
    /// [`IpcError::no_document`] without an open document, and the errors
    /// of `work`.
    pub fn with_document<T>(
        &self,
        work: impl FnOnce(&mut OpenDocument) -> Result<T, IpcError>,
    ) -> Result<T, IpcError> {
        let mut open = recover(self.open.lock());
        let document = open.as_mut().ok_or_else(IpcError::no_document)?;
        work(document)
    }

    /// Replaces the open document (or closes it with `None`), returning the
    /// previous one.
    pub fn replace_document(&self, document: Option<OpenDocument>) -> Option<OpenDocument> {
        std::mem::replace(&mut *recover(self.open.lock()), document)
    }

    /// The path of the open document, if any.
    pub fn document_path(&self) -> Option<PathBuf> {
        recover(self.open.lock())
            .as_ref()
            .map(|open| open.path().to_owned())
    }

    /// The icon store of the open document, if any.
    pub fn icons(&self) -> Option<IconStore> {
        recover(self.open.lock())
            .as_ref()
            .map(|open| open.store().icons())
    }

    /// The preferences of the interface.
    pub fn ui_preferences(&self) -> UiPreferences {
        self.ui_preferences
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// Changes the preferences of the interface and saves them in `ui.toml`.
    ///
    /// # Errors
    /// The file cannot be written; the preferences in memory change anyway,
    /// for the rest of the session.
    pub fn update_ui_preferences(
        &self,
        change: impl FnOnce(&mut UiPreferences),
    ) -> Result<UiPreferences, IpcError> {
        let mut preferences = self
            .ui_preferences
            .write()
            .unwrap_or_else(PoisonError::into_inner);
        change(&mut preferences);
        preferences.save(&self.paths.ui_preferences)?;
        Ok(preferences.clone())
    }

    /// Replaces the watcher of the document file; `None` stops watching.
    pub fn set_watcher(&self, watcher: Option<DocumentWatcher>) {
        *recover(self.watcher.lock()) = watcher;
    }

    /// The deletions that can still be undone.
    pub fn pending_deletes(&self) -> &PendingDeletes {
        &self.pending_deletes
    }

    /// The open sessions (SPEC-UI §4.5).
    pub fn sessions(&self) -> &SessionManager {
        &self.sessions
    }

    /// The files chosen in the native dialogs.
    pub fn chosen(&self) -> &ChosenFiles {
        &self.chosen
    }

    /// Remembers the report of the last import, for "Esporta JSON".
    pub fn set_last_import_report(&self, report: portolano_import::ImportReport) {
        *recover(self.last_import_report.lock()) = Some(report);
    }

    /// The report of the last import, if any.
    pub fn last_import_report(&self) -> Option<portolano_import::ImportReport> {
        recover(self.last_import_report.lock()).clone()
    }

    /// Finishes every deletion that could still be undone: before another
    /// document is opened, and when the app quits.
    pub fn purge_pending_deletes(&self) {
        for pending in self.pending_deletes.take_all() {
            self.purge(pending);
        }
    }

    /// Gives back deletion `id`, taken by an undo that failed for a reason
    /// that can be fixed: it stays undoable until it expires; if it already
    /// expired meanwhile, it is finished now.
    pub fn put_back(&self, id: UndoId, pending: PendingDelete) {
        if pending.expired() {
            self.purge(pending);
        } else {
            self.pending_deletes.put_back(id, pending);
        }
    }

    /// Finishes a deletion that can no longer be undone: its secrets leave
    /// the vault. A secret that cannot be deleted is logged and stays in the
    /// vault, unused; so does every secret of a deletion whose document is
    /// no longer open.
    pub fn purge(&self, pending: PendingDelete) {
        let vault = match self.vault() {
            Ok(vault) => vault,
            Err(error) => {
                tracing::warn!(
                    "segreti eliminati non cancellati dal vault: {}",
                    error.message
                );
                return;
            }
        };
        let open = recover(self.open.lock());
        let Some(open) = open.as_ref().filter(|open| open.path() == pending.document) else {
            tracing::warn!(
                "segreti eliminati non cancellati dal vault: il documento non è più aperto"
            );
            return;
        };
        let outcome = purge_secrets(
            open.store(),
            open.document(),
            vault.as_ref(),
            pending.deleted.secrets(),
        );
        for failure in outcome.cleanup_failures {
            tracing::warn!(reference = failure.reference.as_str(), "{}", failure.reason);
        }
    }
}

/// Recovers a poisoned lock (see the module documentation).
pub(crate) fn recover<'a, T>(
    result: Result<MutexGuard<'a, T>, PoisonError<MutexGuard<'a, T>>>,
) -> MutexGuard<'a, T> {
    result.unwrap_or_else(PoisonError::into_inner)
}
