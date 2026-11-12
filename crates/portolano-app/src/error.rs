//! The error of every command (SPEC-UI §4.2).
//!
//! The frontend chooses how to present an error from its [`ErrorCode`]; the
//! message, in Italian, is shown as it is. No message contains a secret:
//! they come from the `Display` of the errors of `portolano-ops`, which never
//! do (SPEC §1.1).

use std::error::Error;
use std::io;
use std::path::Path;

use portolano_connectors::ConnectError;
use portolano_ops::delete::ExternalReference;
use portolano_ops::secrets::SecretSlot;
use portolano_ops::{ErrorCategory, OpsError};
use serde::Serialize;
use uuid::Uuid;

/// The error returned by a command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct IpcError {
    /// What kind of failure it is.
    pub code: ErrorCode,
    /// The message to show, with its causes.
    pub message: String,
    /// Data the frontend needs to react, for the codes that have any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<IpcErrorDetails>,
}

/// The kind of a failure, from [`OpsError::category`] or from the app.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum ErrorCode {
    /// No node matches.
    NotFound,
    /// Several nodes match.
    Ambiguous,
    /// A value or a request the document does not accept.
    Invalid,
    /// Other nodes refer to the node to delete.
    Referenced,
    /// The vault needs the master password.
    VaultLocked,
    /// Any other error of the vault.
    Vault,
    /// The document changed on disk after it was read.
    ModifiedExternally,
    /// The document file does not exist (any more).
    DocumentMissing,
    /// The command needs an open document and there is none.
    NoDocument,
    /// The connection could not be planned or launched.
    Connect,
    /// A filesystem error.
    Io,
    /// Anything else.
    Internal,
}

/// Data attached to some error codes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum IpcErrorDetails {
    /// The nodes an ambiguous name matches, to choose from.
    Ambiguous {
        /// The matching nodes.
        candidates: Vec<Candidate>,
    },
    /// The field of the editor whose value is refused, so that the message
    /// goes under it.
    Invalid {
        /// The field, as named by the operation (`name`, `host`, `port`,
        /// `color`, `settings`, a configuration key, …).
        field: String,
    },
    /// The references that prevent a deletion.
    Referenced {
        /// References from outside the subtree to delete.
        references: Vec<ExternalReference>,
    },
}

/// A node an ambiguous name matches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
pub struct Candidate {
    /// The node.
    pub id: Uuid,
    /// Its path.
    pub path: String,
}

impl IpcError {
    /// No document is open.
    pub fn no_document() -> Self {
        Self::plain(ErrorCode::NoDocument, "nessun documento aperto".to_owned())
    }

    /// A value refused by the app itself, shown under `field`.
    pub fn invalid(field: &str, message: String) -> Self {
        Self {
            code: ErrorCode::Invalid,
            message,
            details: Some(IpcErrorDetails::Invalid {
                field: field.to_owned(),
            }),
        }
    }

    /// A file of the app that cannot be read as what it should be.
    pub fn invalid_file(path: &Path, reason: &str) -> Self {
        Self::plain(
            ErrorCode::Invalid,
            format!("file '{}' non valido: {reason}", path.display()),
        )
    }

    /// A filesystem error on `path`.
    pub fn io(path: &Path, error: &io::Error) -> Self {
        Self::plain(
            ErrorCode::Io,
            format!("errore di I/O su '{}': {error}", path.display()),
        )
    }

    /// A failure of the app that is not the user's to fix.
    pub fn internal(message: &str) -> Self {
        Self::plain(ErrorCode::Internal, message.to_owned())
    }

    /// Something the command refers to that no longer exists, such as a
    /// deletion that can no longer be undone.
    pub fn not_found(message: String) -> Self {
        Self::plain(ErrorCode::NotFound, message)
    }

    fn plain(code: ErrorCode, message: String) -> Self {
        Self {
            code,
            message,
            details: None,
        }
    }
}

impl From<OpsError> for IpcError {
    fn from(error: OpsError) -> Self {
        Self {
            code: error.category().into(),
            message: message_with_causes(&error),
            details: details_of(&error),
        }
    }
}

fn details_of(error: &OpsError) -> Option<IpcErrorDetails> {
    let invalid = |field: &str| {
        Some(IpcErrorDetails::Invalid {
            field: field.to_owned(),
        })
    };
    if let Some(portolano_core::Error::Ambiguous { candidates, .. }) = core_error_of(error) {
        return Some(IpcErrorDetails::Ambiguous {
            candidates: candidates
                .iter()
                .map(|(id, path)| Candidate {
                    id: *id,
                    path: path.clone(),
                })
                .collect(),
        });
    }
    match error {
        OpsError::Referenced { references, .. } => Some(IpcErrorDetails::Referenced {
            references: references.clone(),
        }),
        OpsError::EmptyName | OpsError::NameTaken { .. } => invalid("name"),
        OpsError::EmptyHost { .. } => invalid("host"),
        OpsError::InvalidPort { .. } => invalid("port"),
        OpsError::InvalidColor { .. } => invalid("color"),
        OpsError::InvalidSettings { .. } | OpsError::InvalidJumpHost { .. } => invalid("settings"),
        OpsError::UnknownProtocol { .. } => invalid("protocol"),
        OpsError::NotACredential { .. } | OpsError::CredentialWithLogin { .. } => {
            invalid("credential")
        }
        OpsError::KeyFileAndVaultKey => invalid("private_key"),
        OpsError::IconNotStored => invalid("icon"),
        OpsError::SecretNotApplicable { slot, .. } | OpsError::SecretNotSet { slot, .. } => {
            invalid(&slot_field(slot))
        }
        OpsError::NotEditable { field, .. } => invalid(field),
        OpsError::UnknownConfigKey { key } | OpsError::InvalidConfigValue { key, .. } => {
            invalid(key)
        }
        _ => None,
    }
}

/// The field of the editor that holds `slot`.
fn slot_field(slot: &SecretSlot) -> String {
    match slot {
        SecretSlot::Password => "password".to_owned(),
        SecretSlot::Passphrase => "passphrase".to_owned(),
        SecretSlot::PrivateKey => "private_key".to_owned(),
        SecretSlot::Property(name) => format!("property:{name}"),
    }
}

/// The error of the core inside `error`, wherever an operation wrapped it.
fn core_error_of(error: &OpsError) -> Option<&portolano_core::Error> {
    match error {
        OpsError::Core(core) | OpsError::Connect(ConnectError::Core(core)) => Some(core),
        _ => None,
    }
}

impl From<ErrorCategory> for ErrorCode {
    fn from(category: ErrorCategory) -> Self {
        match category {
            ErrorCategory::NotFound => ErrorCode::NotFound,
            ErrorCategory::Ambiguous => ErrorCode::Ambiguous,
            // For the app a wrong combination of inputs, a request the
            // document refuses and an unusable file all mean that the input
            // has to change.
            ErrorCategory::Usage
            | ErrorCategory::Invalid
            | ErrorCategory::Rejected
            | ErrorCategory::Unreadable => ErrorCode::Invalid,
            ErrorCategory::Referenced => ErrorCode::Referenced,
            ErrorCategory::VaultLocked => ErrorCode::VaultLocked,
            ErrorCategory::Vault => ErrorCode::Vault,
            ErrorCategory::Connect => ErrorCode::Connect,
            ErrorCategory::ModifiedExternally => ErrorCode::ModifiedExternally,
            ErrorCategory::DocumentMissing => ErrorCode::DocumentMissing,
            ErrorCategory::Io => ErrorCode::Io,
            ErrorCategory::Other => ErrorCode::Internal,
        }
    }
}

/// The message of `error` followed by those of its causes, as the CLI shows
/// them: an I/O error names the file in its own message and the reason only
/// in its source.
fn message_with_causes(error: &dyn Error) -> String {
    let mut message = error.to_string();
    let mut cause = error.source();
    while let Some(source) = cause {
        let text = source.to_string();
        // Transparent wrappers repeat the message of their source.
        if !message.ends_with(&text) {
            message.push_str(": ");
            message.push_str(&text);
        }
        cause = source.source();
    }
    message
}

#[cfg(test)]
mod tests;
