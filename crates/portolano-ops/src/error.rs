//! Errors of the operations.
//!
//! Messages are in Italian because they reach the user unchanged. They name
//! the node (by path) or the file involved and never contain a secret.

use std::path::PathBuf;

use portolano_connectors::ConnectError;
use portolano_core::VaultError;
use portolano_import::ImportError;

/// Errors of the operations.
#[derive(Debug, thiserror::Error)]
pub enum OpsError {
    /// An error of the core: paths, tree, store, configuration.
    #[error(transparent)]
    Core(#[from] portolano_core::Error),

    /// An error of the vault.
    #[error(transparent)]
    Vault(#[from] VaultError),

    /// An error raised while planning or launching a connection.
    #[error(transparent)]
    Connect(#[from] ConnectError),

    /// An error of an importer.
    #[error(transparent)]
    Import(#[from] ImportError),

    /// The document file does not exist.
    #[error("il documento '{path}' non esiste")]
    DocumentNotFound {
        /// The document file.
        path: PathBuf,
    },

    /// A vault file was given together with a backend other than `file`
    /// (DEC-022.6).
    #[error("il file del vault vale solo con il backend 'file', non con '{backend}'")]
    VaultFileConflict {
        /// The backend that was chosen explicitly.
        backend: &'static str,
    },
}
