//! Error types of the core and validation issues.
//!
//! Messages are in Italian because they reach the user unchanged (SPEC §10.3).
//! They name the object involved (id, path, file) and never contain a secret.
//! The `Display` of a variant does not repeat its `source`: callers print the
//! whole chain (for example with anyhow's `{:#}`).

use std::path::PathBuf;

use serde::Serialize;
use uuid::Uuid;

/// Result type of the core, with [`Error`] as the default error.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Errors of the core: tree, paths, credential resolution, store, migrations,
/// icons and configuration.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// No node has the given id.
    #[error("nodo {id} non trovato")]
    NodeNotFound {
        /// The id that was looked up.
        id: Uuid,
    },

    /// A path, an `@id` or a credential name matches no node.
    #[error("nessun nodo corrisponde a '{query}'")]
    NotFound {
        /// The path, `@id` or name as given by the caller.
        query: String,
    },

    /// A path, an `@id` prefix or a credential name matches more than one node.
    #[error("'{query}' è ambiguo: corrisponde a {} nodi", .candidates.len())]
    Ambiguous {
        /// The path, `@id` prefix or name as given by the caller.
        query: String,
        /// The matching nodes, as `(id, path)`.
        candidates: Vec<(Uuid, String)>,
    },

    /// The node exists but is not a folder where a folder is required
    /// (parent of an insert or a move, segment of a path).
    #[error("il nodo {id} non è una cartella")]
    NotAFolder {
        /// The node that should have been a folder.
        id: Uuid,
    },

    /// An id is already present in the document.
    #[error("id duplicato nel documento: {id}")]
    DuplicateId {
        /// The duplicated id.
        id: Uuid,
    },

    /// A node cannot be moved into itself or one of its descendants.
    #[error(
        "impossibile spostare il nodo {id} dentro sé stesso o un suo discendente ({new_parent})"
    )]
    InvalidMove {
        /// The node being moved.
        id: Uuid,
        /// The requested new parent.
        new_parent: Uuid,
    },

    /// An insert, a move or the creation of a path would put a node deeper
    /// than [`MAX_DEPTH`](crate::model::MAX_DEPTH) (DEC-021).
    #[error("l'albero supererebbe la profondità massima di {max} livelli")]
    TooDeep {
        /// The maximum depth.
        max: usize,
    },

    /// A path is syntactically invalid or cannot be used for the operation.
    #[error("percorso non valido '{path}': {reason}")]
    InvalidPath {
        /// The path as given by the caller.
        path: String,
        /// What is wrong with it.
        reason: String,
    },

    /// A reference (`$ref`) points to a node that does not exist.
    #[error("il nodo {from} fa riferimento al nodo {target}, che non esiste")]
    DanglingReference {
        /// The node that holds the reference.
        from: Uuid,
        /// The missing target.
        target: Uuid,
    },

    /// A credential reference points to a node that is not a credential.
    #[error("il nodo {target}, a cui fa riferimento il nodo {from}, non è una credenziale")]
    NotACredential {
        /// The node that holds the reference.
        from: Uuid,
        /// The referenced node.
        target: Uuid,
    },

    /// The node has no credential to resolve (an unsupported node, DEC-010).
    #[error("il nodo {id} non ha credenziali")]
    NoCredential {
        /// The node.
        id: Uuid,
    },

    /// A filesystem operation failed.
    #[error("errore di I/O su '{path}'")]
    Io {
        /// The file or directory involved.
        path: PathBuf,
        /// The underlying error.
        #[source]
        source: std::io::Error,
    },

    /// The document file changed after this store last read or wrote it, so
    /// saving would overwrite someone else's changes (DEC-016.1).
    #[error("il documento '{path}' è stato modificato da un altro processo: ricaricalo")]
    ModifiedExternally {
        /// The document file.
        path: PathBuf,
    },

    /// The document file disappeared after the store was opened (moved or
    /// deleted), so there is nothing to reload.
    #[error("il documento '{path}' non esiste più: è stato spostato o cancellato")]
    DocumentMissing {
        /// The document file.
        path: PathBuf,
    },

    /// The file to create already exists.
    #[error("il file '{path}' esiste già")]
    AlreadyExists {
        /// The existing file.
        path: PathBuf,
    },

    /// The file is not a Portolano document (`format` or `schema_version`
    /// missing or wrong).
    #[error("'{path}' non è un documento Portolano")]
    NotADocument {
        /// The file that was read.
        path: PathBuf,
    },

    /// The document is not valid JSON or does not match the model.
    #[error("documento '{path}' non valido")]
    InvalidDocument {
        /// The file that was read.
        path: PathBuf,
        /// The parse or deserialization error.
        #[source]
        source: serde_json::Error,
    },

    /// The document was written by a newer version of Portolano.
    #[error(
        "versione dello schema {found} non supportata (massima supportata: {supported}): aggiorna Portolano"
    )]
    UnsupportedSchema {
        /// The `schema_version` found in the document.
        found: u32,
        /// The highest version this build understands.
        supported: u32,
    },

    /// No migration starts from the given schema version.
    #[error("manca la migrazione dello schema dalla versione {from}")]
    MissingMigration {
        /// The version without a migration.
        from: u32,
    },

    /// A schema migration failed.
    #[error("migrazione dello schema dalla versione {from} non riuscita: {reason}")]
    Migration {
        /// The version the migration starts from.
        from: u32,
        /// What went wrong.
        reason: String,
    },

    /// The document could not be serialized.
    #[error("serializzazione del documento non riuscita")]
    Serialize {
        /// The serialization error.
        #[source]
        source: serde_json::Error,
    },

    /// The bytes of an icon image are not of an accepted type (SPEC §6.4),
    /// whatever type was declared.
    #[error(
        "il contenuto dell'icona non corrisponde a nessun formato immagine ammesso (dichiarato: {mime})"
    )]
    UnsupportedIcon {
        /// The declared MIME type.
        mime: String,
    },

    /// The configuration file is not valid TOML or does not match the schema.
    #[error("configurazione '{path}' non valida")]
    InvalidConfig {
        /// The configuration file.
        path: PathBuf,
        /// The parse or deserialization error.
        #[source]
        source: toml::de::Error,
    },

    /// The configuration could not be serialized.
    #[error("serializzazione della configurazione non riuscita")]
    SerializeConfig {
        /// The serialization error.
        #[source]
        source: toml::ser::Error,
    },

    /// The per-user data directory cannot be determined (no home directory).
    #[error("impossibile determinare la directory dei dati dell'utente")]
    NoDataDir,
}

/// Errors of the secret vaults (SPEC §5).
///
/// No variant carries secret material: backends must turn platform errors
/// into a description that contains neither the secret nor the plaintext.
#[derive(Debug, thiserror::Error)]
pub enum VaultError {
    /// The master password of the file vault is wrong. Authenticated
    /// encryption cannot tell this apart from a tampered ciphertext.
    #[error("master password errata o vault manomesso")]
    BadMasterPassword,

    /// The master password could not be obtained (no TTY, no environment
    /// variable, confirmation mismatch).
    #[error("master password non disponibile: {reason}")]
    MasterPasswordUnavailable {
        /// Why it is unavailable.
        reason: String,
    },

    /// A filesystem operation on the vault file failed.
    #[error("errore di I/O sul vault '{path}'")]
    Io {
        /// The vault file.
        path: PathBuf,
        /// The underlying error.
        #[source]
        source: std::io::Error,
    },

    /// The vault file is malformed (bad JSON, bad base64, unknown algorithm).
    #[error("vault '{path}' danneggiato: {reason}")]
    Corrupt {
        /// The vault file.
        path: PathBuf,
        /// What is wrong; never a fragment of the plaintext.
        reason: String,
    },

    /// The vault file was written by a newer version of Portolano.
    #[error(
        "versione del vault '{path}' non supportata: {found} (massima supportata: {supported})"
    )]
    UnsupportedVersion {
        /// The vault file.
        path: PathBuf,
        /// The version found in the file.
        found: u32,
        /// The highest version this build understands.
        supported: u32,
    },

    /// Key derivation or encryption failed.
    #[error("errore crittografico del vault: {reason}")]
    Crypto {
        /// What failed.
        reason: String,
    },

    /// The storage backend (Keychain, Credential Manager, Secret Service,
    /// in-memory map) reported an error.
    #[error("errore del vault '{backend}': {reason}")]
    Backend {
        /// The backend name, as returned by `Vault::backend`.
        backend: &'static str,
        /// The backend's description of the failure.
        reason: String,
    },
}

/// Severity of a validation [`Issue`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// The document is inconsistent; `validate` exits with code 4.
    Error,
    /// Suspicious but usable.
    Warning,
    /// Worth knowing, not a problem: for example a node kept verbatim because
    /// Portolano does not support it (DEC-016). Never affects the exit code.
    Info,
}

/// A problem found by validation (SPEC §4.4), by a protocol or by the CLI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Issue {
    /// How serious the problem is.
    pub severity: Severity,
    /// The node concerned, if any.
    pub node: Option<Uuid>,
    /// Stable machine-readable code, such as `dangling_ref`.
    pub code: &'static str,
    /// Human-readable description, in Italian.
    pub message: String,
}
