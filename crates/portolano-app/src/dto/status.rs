//! The state of the app and of its documents.

use std::path::Path;

use portolano_core::VaultSelection;
use serde::{Deserialize, Serialize};

use crate::prefs::InterfacePreferences;

/// What the app knows: the documents, the vault, the preferences.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AppStatus {
    /// The document opened by default: from the configuration, or
    /// `portolano.json` in the data directory.
    pub default_document: DocumentLocation,
    /// The vault backend of the configuration.
    pub vault: VaultKind,
    /// The open document, if any.
    pub document: Option<DocumentInfo>,
    /// The document opened last, to open again at startup.
    pub last_document: Option<DocumentLocation>,
    /// The preferences of the interface.
    pub preferences: InterfacePreferences,
}

/// A document file and whether it exists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
pub struct DocumentLocation {
    /// The path, for display.
    pub path: String,
    /// Whether the file exists.
    pub exists: bool,
}

impl DocumentLocation {
    /// The location of `path`.
    pub fn of(path: &Path) -> Self {
        Self {
            path: path.display().to_string(),
            exists: path.is_file(),
        }
    }
}

/// The open document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct DocumentInfo {
    /// The name inside the document.
    pub name: String,
    /// The file name, for the title.
    pub file_name: String,
    /// The path, for display.
    pub path: String,
}

impl DocumentInfo {
    /// The document named `name` in `path`.
    pub fn new(name: &str, path: &Path) -> Self {
        Self {
            name: name.to_owned(),
            file_name: file_name(path),
            path: path.display().to_string(),
        }
    }
}

/// A file chosen in a native dialog: only its name reaches the frontend
/// (SPEC-UI §4.6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ChosenFile {
    /// The file name.
    pub file_name: String,
}

impl ChosenFile {
    /// The chosen `path`.
    pub fn of(path: &Path) -> Self {
        Self {
            file_name: file_name(path),
        }
    }
}

/// The name of the file at `path`, for display.
pub fn file_name(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
}

/// The backend of a vault.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum VaultKind {
    /// The system keychain.
    Keychain,
    /// The encrypted file vault.
    File,
    /// An in-memory vault, for tests.
    Memory,
}

impl From<&VaultSelection> for VaultKind {
    fn from(selection: &VaultSelection) -> Self {
        match selection {
            VaultSelection::Keychain => VaultKind::Keychain,
            VaultSelection::File { .. } => VaultKind::File,
            VaultSelection::Memory => VaultKind::Memory,
        }
    }
}
