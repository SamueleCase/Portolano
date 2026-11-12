//! The files chosen in the native dialogs (SPEC-UI §4.6): the frontend
//! receives only the name to show, the path stays here until the command
//! that uses it.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use super::recover;

/// What a chosen file is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileKind {
    /// A document to open.
    Document,
    /// A document to create.
    NewDocument,
    /// A file to import.
    Import,
    /// An image to add to the icons.
    Icon,
}

/// The last file chosen for each purpose.
#[derive(Default)]
pub struct ChosenFiles {
    files: Mutex<HashMap<FileKind, PathBuf>>,
}

impl ChosenFiles {
    /// Remembers `path` as the file chosen for `kind`.
    pub fn set(&self, kind: FileKind, path: &Path) {
        recover(self.files.lock()).insert(kind, path.to_owned());
    }

    /// The file chosen for `kind`, if any.
    pub fn get(&self, kind: FileKind) -> Option<PathBuf> {
        recover(self.files.lock()).get(&kind).cloned()
    }
}
