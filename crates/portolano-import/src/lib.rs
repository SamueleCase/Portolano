//! Portolano importers (SPEC §8).
//!
//! An [`Importer`] turns the bytes of a foreign document into a folder of
//! Portolano nodes plus a report. Importers do not touch the vault or the
//! filesystem: secrets and icons are returned to the caller, which stores
//! them.

mod prompter;
mod registry;
mod report;

pub mod royal;

use std::path::Path;

use portolano_core::{Node, Secret, SecretRef};

pub use prompter::{NonInteractive, SecretField, SecretPrompter, SecretRequest};
pub use registry::Registry;
pub use report::{ImportReport, ItemStatus, ReportItem, TypeCounts};

/// A foreign document format that can be imported.
pub trait Importer: Send + Sync {
    /// Identifier used by `--format`, such as `"royal"`.
    fn id(&self) -> &'static str;

    /// Name shown to the user, such as `"Royal TS/TSX"`.
    fn display_name(&self) -> &'static str;

    /// Whether the file looks like this format, from its path (extension) and
    /// its first bytes.
    fn detect(&self, path: &Path, head: &[u8]) -> bool;

    /// Imports a whole document. Encrypted secrets are requested from
    /// `prompter`; the answers are returned in [`ImportOutcome::secrets`].
    ///
    /// # Errors
    /// [`ImportError`] when the input cannot be read as this format, or the
    /// error of the prompter.
    fn import(
        &self,
        input: &[u8],
        opts: &ImportOptions,
        prompter: &mut dyn SecretPrompter,
    ) -> Result<ImportOutcome, ImportError>;
}

/// Options of an import.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImportOptions {
    /// Name of the source file, without its directory, recorded in
    /// [`SourceInfo`](portolano_core::SourceInfo) and in the report.
    pub source_file_name: String,
    /// Import the trash too, into a dedicated folder.
    pub include_trash: bool,
}

/// The result of an import, to be inserted in a document by the caller.
#[derive(Debug)]
pub struct ImportOutcome {
    /// A folder that contains everything imported.
    pub root: Node,
    /// Secrets obtained from the prompter, to be written to the vault.
    pub secrets: Vec<(SecretRef, Secret)>,
    /// Custom icons, to be stored in the icon store.
    pub icons: Vec<PendingIcon>,
    /// What was imported, partially imported, unsupported or skipped.
    pub report: ImportReport,
}

/// A custom icon found by an importer. The nodes already reference it as
/// [`Icon::Custom`](portolano_core::Icon::Custom) with the same hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingIcon {
    /// SHA-256 of `bytes`, lowercase hex.
    pub placeholder_sha256: String,
    /// The image.
    pub bytes: Vec<u8>,
    /// MIME type of the image.
    pub mime: String,
}

/// Errors of an import.
///
/// Messages are in Italian and never contain field values: they may hold
/// encrypted secrets of the source document.
#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    /// The input is not in a recognized format.
    #[error("formato del file non riconosciuto")]
    UnrecognizedFormat,

    /// A compressed input could not be decompressed.
    #[error("decompressione del file non riuscita: {reason}")]
    Decompress {
        /// What went wrong.
        reason: String,
    },

    /// The input is not well-formed XML.
    #[error("XML non valido: {reason}")]
    Xml {
        /// What went wrong, with the position if known.
        reason: String,
    },

    /// The input is well-formed but not a valid document of the format.
    #[error("contenuto non valido: {reason}")]
    InvalidInput {
        /// What is wrong, such as an unexpected root element.
        reason: String,
    },

    /// The prompter could not read a secret.
    #[error("lettura del segreto non riuscita")]
    Prompt {
        /// The underlying error.
        #[source]
        source: std::io::Error,
    },

    /// The user aborted the import while being prompted.
    #[error("importazione annullata")]
    Cancelled,
}
