//! The state of the app at startup.

use portolano_core::VaultSelection;
use serde::Serialize;

/// What the app knows before opening a document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AppStatus {
    /// The document opened by default: from the configuration, or
    /// `portolano.json` in the data directory.
    pub default_document: DocumentLocation,
    /// The vault backend of the configuration.
    pub vault: VaultKind,
}

/// A document file and whether it exists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
pub struct DocumentLocation {
    /// The path, for display.
    pub path: String,
    /// Whether the file exists.
    pub exists: bool,
}

/// The backend of a vault.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, specta::Type)]
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
