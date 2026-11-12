//! Why a session cannot be opened or driven.

use std::io;
use std::path::PathBuf;

use portolano_connectors::ConnectError;

use super::model::{SessionId, SessionKind};
use crate::sidecar::SidecarError;

/// Errors of the sessions. Messages are in Italian and never contain a
/// secret.
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    /// No open session has this id.
    #[error("sessione {id} inesistente")]
    NotFound {
        /// The id asked for.
        id: SessionId,
    },

    /// The session does not accept this operation, such as typing into an
    /// RDP window.
    #[error("la sessione {} non supporta {operation}", .kind.label())]
    Unsupported {
        /// Its kind.
        kind: SessionKind,
        /// The operation, such as "l'input".
        operation: &'static str,
    },

    /// The program wants a secret on its stdin, which in a terminal of the
    /// app would be its keyboard: the secret could be echoed to the tab
    /// (DEC-030.3).
    #[error(
        "questa connessione deve ricevere la password sullo stdin e non può aprirsi nel \
         terminale integrato: usa \"Apri in terminale esterno\""
    )]
    StdinSecretInTerminal,

    /// The plan does not end with a program to run in a session.
    #[error("il piano di lancio non apre una sessione: {reason}")]
    NotASession {
        /// What the plan does instead.
        reason: &'static str,
    },

    /// The terminal of the session could not be created or started.
    #[error("terminale integrato: {reason}")]
    Terminal {
        /// What failed.
        reason: String,
    },

    /// A program could not be started.
    #[error("avvio di '{}' non riuscito", .program.display())]
    Spawn {
        /// The program.
        program: PathBuf,
        /// The underlying error.
        #[source]
        source: io::Error,
    },

    /// Talking to the program failed.
    #[error("errore di I/O: {context}")]
    Io {
        /// What was being done.
        context: &'static str,
        /// The underlying error.
        #[source]
        source: io::Error,
    },

    /// The launcher of a new window failed.
    #[error("apertura della nuova finestra non riuscita (codice {code})")]
    NewWindow {
        /// Exit code of the launcher.
        code: i32,
    },

    /// Preparing or running the plan failed.
    #[error(transparent)]
    Connect(#[from] ConnectError),

    /// The CLI shipped with the app, needed by askpass and new windows, is
    /// missing.
    #[error(transparent)]
    Sidecar(#[from] SidecarError),
}

impl SessionError {
    /// An [`SessionError::Io`] while doing `context`.
    pub(super) fn io(context: &'static str) -> impl FnOnce(io::Error) -> SessionError {
        move |source| SessionError::Io { context, source }
    }
}
