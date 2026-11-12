//! Errors of the CLI itself, and the report of any error on stderr with
//! the exit code of SPEC §10.3 and DEC-022.7.

use std::io;
use std::path::PathBuf;

use portolano_connectors::ConnectError;
use portolano_connectors::secret_input::SecretInputError;
use portolano_core::{Document, VaultError};
use portolano_ops::{ErrorCategory, OpsError};

use crate::output::{short_id, visible};

/// Exit code: generic error.
pub const EXIT_ERROR: i32 = 1;
/// Exit code: wrong usage.
pub const EXIT_USAGE: i32 = 2;
/// Exit code: not found or ambiguous.
pub const EXIT_NOT_FOUND: i32 = 3;
/// Exit code: validation.
pub const EXIT_VALIDATION: i32 = 4;
/// Exit code: vault.
pub const EXIT_VAULT: i32 = 5;
/// Exit code: connection error before launch.
pub const EXIT_CONNECT: i32 = 6;

/// Conflicts of an import listed by name; a re-import of a whole document
/// conflicts on every node, and the rest is only counted.
const MAX_LISTED_CONFLICTS: usize = 20;

/// Errors raised by the CLI layer.
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    /// `--json` on a command that writes.
    #[error("--json vale solo per i comandi di lettura")]
    JsonNotSupported,
    /// Two options that cannot be used together.
    #[error("{first} non si può usare insieme a {second}")]
    Incompatible {
        /// The first option.
        first: &'static str,
        /// The second option.
        second: &'static str,
    },
    /// A move into the node itself or one of its descendants.
    #[error(
        "impossibile spostare '{path}' dentro sé stesso o un suo discendente ('{destination}')"
    )]
    InvalidMove {
        /// The node to move.
        path: String,
        /// The destination.
        destination: String,
    },
    /// A question with no terminal to ask it on.
    #[error("impossibile chiedere {what}: nessun terminale")]
    NoTerminal {
        /// What was to be asked.
        what: String,
    },
    /// `delete` without `-y` and without a terminal.
    #[error("conferma richiesta: usa -y per eliminare senza conferma")]
    ConfirmationRequired,
    /// The user answered no.
    #[error("operazione annullata")]
    Aborted,
    /// An empty secret.
    #[error("il segreto non può essere vuoto")]
    EmptySecret,
    /// The two entries of a secret differ.
    #[error("i due valori non coincidono")]
    SecretMismatch,
    /// A private key typed at a prompt.
    #[error("la chiave privata si legge solo da stdin: usa --stdin")]
    KeyNeedsStdin,
    /// A secret could not be read from stdin: too long, not UTF-8, or an
    /// I/O error. The reason is the source, which the report already joins
    /// to this message.
    #[error("stdin")]
    Stdin(#[from] SecretInputError),
    /// The terminal could not be used.
    #[error("terminale non disponibile")]
    Terminal(#[source] io::Error),
    /// A file named on the command line could not be read or written.
    #[error("errore di I/O su '{path}'")]
    File {
        /// The file.
        path: PathBuf,
        /// The underlying error.
        #[source]
        source: io::Error,
    },
}

impl CliError {
    fn exit_code(&self) -> i32 {
        match self {
            CliError::JsonNotSupported
            | CliError::Incompatible { .. }
            | CliError::KeyNeedsStdin => EXIT_USAGE,
            _ => EXIT_ERROR,
        }
    }
}

/// `error` with the nodes it names by id shown by their path in `doc`,
/// which is what the user typed.
pub fn with_paths(error: OpsError, doc: &Document) -> anyhow::Error {
    let path = |id| doc.path_of(id).unwrap_or_else(|| format!("@{id}"));
    match error {
        OpsError::Core(portolano_core::Error::InvalidMove { id, new_parent }) => {
            CliError::InvalidMove {
                path: path(id),
                destination: path(new_parent),
            }
            .into()
        }
        other => other.into(),
    }
}

/// Prints `error` on stderr, with the details that help to fix it, and
/// returns the exit code.
pub fn report(error: &anyhow::Error) -> i32 {
    // Messages quote names, paths and hosts of the document.
    eprintln!("errore: {}", visible(&format!("{error:#}")));
    for cause in error.chain() {
        if let Some(error) = cause.downcast_ref::<OpsError>() {
            print_details(error);
        } else if let Some(portolano_core::Error::Ambiguous { candidates, .. }) =
            cause.downcast_ref::<portolano_core::Error>()
        {
            print_candidates(candidates);
        }
    }
    exit_code(error)
}

/// The exit code of the most specific error in the chain.
fn exit_code(error: &anyhow::Error) -> i32 {
    for cause in error.chain() {
        if let Some(error) = cause.downcast_ref::<CliError>() {
            return error.exit_code();
        }
        let category = if let Some(error) = cause.downcast_ref::<OpsError>() {
            error.category()
        } else if let Some(error) = cause.downcast_ref::<ConnectError>() {
            error.into()
        } else if let Some(error) = cause.downcast_ref::<portolano_core::Error>() {
            error.into()
        } else if let Some(error) = cause.downcast_ref::<VaultError>() {
            error.into()
        } else {
            continue;
        };
        return category_code(category);
    }
    EXIT_ERROR
}

/// The exit code of an error category (SPEC §10.3, DEC-022.7).
fn category_code(category: ErrorCategory) -> i32 {
    match category {
        ErrorCategory::NotFound | ErrorCategory::Ambiguous => EXIT_NOT_FOUND,
        ErrorCategory::Usage => EXIT_USAGE,
        ErrorCategory::Invalid => EXIT_VALIDATION,
        ErrorCategory::VaultLocked | ErrorCategory::Vault => EXIT_VAULT,
        ErrorCategory::Connect => EXIT_CONNECT,
        ErrorCategory::Referenced
        | ErrorCategory::Rejected
        | ErrorCategory::ModifiedExternally
        | ErrorCategory::DocumentMissing
        | ErrorCategory::Unreadable
        | ErrorCategory::Io
        | ErrorCategory::Other => EXIT_ERROR,
    }
}

fn print_details(error: &OpsError) {
    match error {
        OpsError::Core(portolano_core::Error::Ambiguous { candidates, .. }) => {
            print_candidates(candidates);
        }
        OpsError::Referenced { references, .. } => {
            eprintln!("riferimenti:");
            for reference in references {
                eprintln!(
                    "  {} -> {}",
                    visible(&reference.from_path),
                    visible(&reference.target_path)
                );
            }
            eprintln!("usa --force per eliminare comunque");
        }
        OpsError::FolderNotEmpty { .. } => eprintln!("usa -r per eliminare anche il contenuto"),
        OpsError::ImportConflicts { conflicts } => {
            for (_, path) in conflicts.iter().take(MAX_LISTED_CONFLICTS) {
                eprintln!("  {}", visible(path));
            }
            if conflicts.len() > MAX_LISTED_CONFLICTS {
                eprintln!("  … e altri {}", conflicts.len() - MAX_LISTED_CONFLICTS);
            }
            eprintln!("usa --on-conflict update oppure new-ids");
        }
        OpsError::DocumentNotFound { .. } => eprintln!("crealo con `portolano init`"),
        _ => {}
    }
}

fn print_candidates(candidates: &[(uuid::Uuid, String)]) {
    eprintln!("candidati:");
    for (id, path) in candidates {
        eprintln!("  @{}  {}", short_id(*id), visible(path));
    }
}
