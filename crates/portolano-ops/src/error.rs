//! Errors of the operations.
//!
//! Messages are in Italian because they reach the user unchanged. They name
//! the node (by path) or the file involved and never contain a secret.

use std::path::PathBuf;

use portolano_connectors::ConnectError;
use portolano_core::VaultError;
use portolano_import::ImportError;
use uuid::Uuid;

use crate::delete::ExternalReference;
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

    /// A jump host that is not an SSH connection.
    #[error("il jump host '{path}' non è una connessione ssh")]
    InvalidJumpHost {
        /// The node given as jump host.
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

    /// An edit gives a credential together with a user or a domain.
    #[error("per '{path}' indica la credenziale oppure utente e dominio, non entrambi")]
    CredentialWithLogin {
        /// The node.
        path: String,
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

    /// A folder with children was to be deleted without `recursive`.
    #[error("la cartella '{path}' contiene {children} elementi: serve l'eliminazione ricorsiva")]
    FolderNotEmpty {
        /// The folder.
        path: String,
        /// Its direct children.
        children: usize,
    },

    /// Nodes outside the deleted subtree refer to it (DEC-018.1).
    #[error("'{path}' è usato da {} riferimenti esterni: eliminazione rifiutata", .references.len())]
    Referenced {
        /// The node to delete.
        path: String,
        /// The references from outside the subtree.
        references: Vec<ExternalReference>,
    },

    /// `before`/`after` of a move names a node that is not a child of the
    /// destination.
    #[error("'{path}' non è un elemento di '{destination}'")]
    NotASibling {
        /// The node given as reference.
        path: String,
        /// The destination folder.
        destination: String,
    },

    /// Ids of the import already exist in the document
    /// (`ConflictPolicy::Fail`).
    #[error("{} nodi dell'import esistono già nel documento", .conflicts.len())]
    ImportConflicts {
        /// The existing nodes, as `(id, path)`.
        conflicts: Vec<(Uuid, String)>,
    },

    /// An update would change the kind of a node (folder, connection,
    /// credential, unsupported).
    #[error("l'import cambierebbe il tipo di '{path}': usa un'altra strategia per i conflitti")]
    ImportKindChanged {
        /// The existing node.
        path: String,
    },

    /// The import does not fit under the destination (DEC-021).
    #[error(
        "l'import in '{into}' supererebbe la profondità massima di {max} livelli: scegli una cartella meno profonda"
    )]
    ImportTooDeep {
        /// The destination folder.
        into: String,
        /// The maximum depth.
        max: usize,
    },

    /// `--format` names no importer.
    #[error("formato d'importazione sconosciuto '{format}' (disponibili: {})", .known.join(", "))]
    UnknownImportFormat {
        /// The format given.
        format: String,
        /// The registered importers.
        known: Vec<String>,
    },

    /// No importer recognizes the file.
    #[error("formato del file '{file}' non riconosciuto: indicalo esplicitamente")]
    UnrecognizedImportFile {
        /// The file name.
        file: String,
    },

    /// A configuration key that does not exist.
    #[error("chiave di configurazione sconosciuta '{key}'")]
    UnknownConfigKey {
        /// The key.
        key: String,
    },

    /// A configuration value that the core or the protocol rejects.
    #[error("valore non valido per '{key}': {reason}")]
    InvalidConfigValue {
        /// The key.
        key: String,
        /// Why it is rejected.
        reason: String,
    },

    /// A key file and a key for the vault were both given.
    #[error("indica la chiave privata come file oppure nel vault, non entrambe")]
    KeyFileAndVaultKey,
}
