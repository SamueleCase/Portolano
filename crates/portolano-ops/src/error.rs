//! Errors of the operations.
//!
//! Messages are in Italian because they reach the user unchanged. They name
//! the node (by path) or the file involved and never contain a secret.

use std::path::PathBuf;

use portolano_connectors::ConnectError;
use portolano_core::VaultError;
use portolano_import::ImportError;

use crate::secrets::SecretSlot;

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

    /// A node name is empty or blank.
    #[error("il nome del nodo non può essere vuoto")]
    EmptyName,

    /// A sibling already has this name, which would make paths ambiguous
    /// (DEC-022.5).
    #[error("in '{parent}' esiste già un nodo chiamato '{name}'")]
    NameTaken {
        /// The parent folder.
        parent: String,
        /// The name.
        name: String,
    },

    /// A connection without a host.
    #[error("la connessione '{path}' deve avere un host")]
    EmptyHost {
        /// The connection.
        path: String,
    },

    /// Port 0.
    #[error("porta non valida: {port}")]
    InvalidPort {
        /// The port.
        port: u16,
    },

    /// A color that is not `#RRGGBB`.
    #[error("colore non valido '{value}': atteso #RRGGBB")]
    InvalidColor {
        /// The value given.
        value: String,
    },

    /// The node is not a connection.
    #[error("'{path}' non è una connessione")]
    NotAConnection {
        /// The node.
        path: String,
    },

    /// A credential reference points to a node that is not a credential.
    #[error("'{path}' non è una credenziale")]
    NotACredential {
        /// The referenced node.
        path: String,
    },

    /// A protocol that is not registered.
    #[error("protocollo sconosciuto '{protocol}' (disponibili: {})", .known.join(", "))]
    UnknownProtocol {
        /// The protocol id.
        protocol: String,
        /// The registered protocols.
        known: Vec<String>,
    },

    /// Settings rejected by the protocol.
    #[error("impostazioni non valide per '{path}': {reason}")]
    InvalidSettings {
        /// The connection.
        path: String,
        /// The protocol's description of the problem.
        reason: String,
    },

    /// A field that this kind of node does not have.
    #[error("il campo '{field}' non si può modificare su '{path}'")]
    NotEditable {
        /// The node.
        path: String,
        /// The field.
        field: &'static str,
    },

    /// A secret this node cannot hold, such as a password on a connection
    /// whose credential is not inline.
    #[error("{slot} non è applicabile a '{path}'")]
    SecretNotApplicable {
        /// The node.
        path: String,
        /// The secret.
        slot: SecretSlot,
    },

    /// A secret property was expected, but the property holds text.
    #[error("la proprietà '{name}' di '{path}' è testuale, non segreta")]
    PropertyIsText {
        /// The node.
        path: String,
        /// The property.
        name: String,
    },

    /// There is no secret to clear.
    #[error("{slot} di '{path}' non è impostata")]
    SecretNotSet {
        /// The node.
        path: String,
        /// The secret.
        slot: SecretSlot,
    },

    /// A key file and a key for the vault were both given.
    #[error("indica la chiave privata come file oppure nel vault, non entrambe")]
    KeyFileAndVaultKey,
}
