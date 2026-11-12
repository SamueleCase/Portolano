//! The state shared by the commands (SPEC-UI §4.1). UI-WP0 keeps only what
//! its command and the icon scheme need; UI-WP2 adds the open document, the
//! vault and the rest.

use std::path::{Path, PathBuf};
use std::sync::{PoisonError, RwLock};

use portolano_core::IconStore;

/// The state of the app, managed by Tauri.
#[derive(Debug)]
pub struct AppState {
    config_path: PathBuf,
    icons: RwLock<Option<IconStore>>,
}

impl AppState {
    /// The state of an app that reads its configuration from `config_path`
    /// and has no document open.
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            config_path,
            icons: RwLock::new(None),
        }
    }

    /// The configuration file. Commands read it each time, so that changes
    /// made meanwhile by the CLI are seen.
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// The icon store of the open document, if any.
    pub fn icons(&self) -> Option<IconStore> {
        // A panic while the lock was held cannot leave the value
        // half-written: it is replaced in one assignment.
        self.icons
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}
