//! Classification of errors, shared by the exit codes of the CLI
//! (DEC-022.7) and the error codes of the app (SPEC-UI §4.2), so that the
//! two cannot drift apart.

use portolano_connectors::ConnectError;
use portolano_core::VaultError;
use portolano_import::ImportError;

use crate::OpsError;

/// What kind of failure an error is, for whoever has to react to it: an exit
/// code, a dialog, a field marked as invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    /// No node matches.
    NotFound,
    /// Several nodes match; the error lists them.
    Ambiguous,
    /// Options or inputs that cannot be given together.
    Usage,
    /// A value that would make the document or the configuration invalid.
    Invalid,
    /// Nodes outside the subtree to delete refer to it (DEC-018.1).
    Referenced,
    /// The request does not apply to the document as it is: a name already
    /// taken, a folder that is not empty, a move into a descendant, an
    /// import that conflicts with existing nodes, a secret the node cannot
    /// hold, a credential reference that leads nowhere.
    Rejected,
    /// The vault needs a master password that is not available: in the app,
    /// the vault is locked.
    VaultLocked,
    /// Any other error of the vault.
    Vault,
    /// The connection could not be planned or launched.
    Connect,
    /// The document changed on disk after it was read (DEC-016.1).
    ModifiedExternally,
    /// The document file does not exist, or no longer exists.
    DocumentMissing,
    /// A file that cannot be used as what it should be: not a document, a
    /// newer schema, an invalid configuration, an import file in an unknown
    /// or broken format, an unsupported icon.
    Unreadable,
    /// A filesystem error.
    Io,
    /// Anything else, such as a failed serialization or a cancelled import.
    Other,
}

impl OpsError {
    /// The category of this error.
    pub fn category(&self) -> ErrorCategory {
        match self {
            OpsError::Core(error) => error.into(),
            OpsError::Vault(error) => error.into(),
            OpsError::Connect(error) => error.into(),
            OpsError::Import(error) => error.into(),
            OpsError::DocumentNotFound { .. } => ErrorCategory::DocumentMissing,
            OpsError::VaultFileConflict { .. }
            | OpsError::KeyFileAndVaultKey
            | OpsError::CredentialWithLogin { .. }
            | OpsError::UnknownImportFormat { .. } => ErrorCategory::Usage,
            OpsError::NotAConnection { .. } => ErrorCategory::Connect,
            OpsError::EmptyName
            | OpsError::NotACredential { .. }
            | OpsError::InvalidJumpHost { .. }
            | OpsError::ImportTooDeep { .. }
            | OpsError::EmptyHost { .. }
            | OpsError::InvalidPort { .. }
            | OpsError::InvalidColor { .. }
            | OpsError::UnknownProtocol { .. }
            | OpsError::InvalidSettings { .. }
            | OpsError::UnknownConfigKey { .. }
            | OpsError::InvalidConfigValue { .. } => ErrorCategory::Invalid,
            OpsError::Referenced { .. } => ErrorCategory::Referenced,
            OpsError::NameTaken { .. }
            | OpsError::NotEditable { .. }
            | OpsError::SecretNotApplicable { .. }
            | OpsError::PropertyIsText { .. }
            | OpsError::SecretNotSet { .. }
            | OpsError::FolderNotEmpty { .. }
            | OpsError::NotASibling { .. }
            | OpsError::ImportConflicts { .. }
            | OpsError::ImportKindChanged { .. } => ErrorCategory::Rejected,
            OpsError::UnrecognizedImportFile { .. } => ErrorCategory::Unreadable,
        }
    }
}

impl From<&portolano_core::Error> for ErrorCategory {
    fn from(error: &portolano_core::Error) -> Self {
        use portolano_core::Error;
        match error {
            Error::NodeNotFound { .. } | Error::NotFound { .. } => ErrorCategory::NotFound,
            Error::Ambiguous { .. } => ErrorCategory::Ambiguous,
            Error::TooDeep { .. } => ErrorCategory::Invalid,
            Error::NotAFolder { .. }
            | Error::InvalidMove { .. }
            | Error::InvalidPath { .. }
            | Error::DanglingReference { .. }
            | Error::NotACredential { .. }
            | Error::NoCredential { .. }
            | Error::AlreadyExists { .. } => ErrorCategory::Rejected,
            Error::ModifiedExternally { .. } => ErrorCategory::ModifiedExternally,
            Error::DocumentMissing { .. } => ErrorCategory::DocumentMissing,
            Error::DuplicateId { .. }
            | Error::NotADocument { .. }
            | Error::InvalidDocument { .. }
            | Error::UnsupportedSchema { .. }
            | Error::MissingMigration { .. }
            | Error::Migration { .. }
            | Error::UnsupportedIcon { .. }
            | Error::InvalidConfig { .. } => ErrorCategory::Unreadable,
            Error::Io { .. } => ErrorCategory::Io,
            Error::Serialize { .. } | Error::SerializeConfig { .. } | Error::NoDataDir => {
                ErrorCategory::Other
            }
        }
    }
}

impl From<&VaultError> for ErrorCategory {
    fn from(error: &VaultError) -> Self {
        match error {
            VaultError::MasterPasswordUnavailable { .. } => ErrorCategory::VaultLocked,
            _ => ErrorCategory::Vault,
        }
    }
}

impl From<&ConnectError> for ErrorCategory {
    fn from(error: &ConnectError) -> Self {
        match error {
            ConnectError::Vault(error) => error.into(),
            ConnectError::Core(error) => match ErrorCategory::from(error) {
                // The node to connect to was not found: the same answer as
                // any other lookup.
                found @ (ErrorCategory::NotFound | ErrorCategory::Ambiguous) => found,
                _ => ErrorCategory::Connect,
            },
            // Everything else happens while planning or launching.
            _ => ErrorCategory::Connect,
        }
    }
}

impl From<&ImportError> for ErrorCategory {
    fn from(error: &ImportError) -> Self {
        match error {
            ImportError::UnrecognizedFormat
            | ImportError::Decompress { .. }
            | ImportError::Xml { .. }
            | ImportError::InvalidInput { .. } => ErrorCategory::Unreadable,
            ImportError::Prompt { .. } | ImportError::Cancelled => ErrorCategory::Other,
        }
    }
}

#[cfg(test)]
mod tests;
