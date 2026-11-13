//! The values that describe a session to the frontend (SPEC-UI §4.4, §4.5).

use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Identifies an open session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
#[serde(transparent)]
pub struct SessionId(Uuid);

impl SessionId {
    /// A new, unique id.
    pub(super) fn new() -> SessionId {
        SessionId(Uuid::new_v4())
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// How a session is shown in its tab. The integrated RDP panel will be one
/// more kind, with its own view and frames (DEC-027).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "kebab-case")]
pub enum SessionKind {
    /// A program in a terminal of the app: `ssh`, or FreeRDP when it has to
    /// ask for a password (DEC-025).
    Terminal,
    /// FreeRDP in its own window; the tab follows its process.
    RdpExternal,
    /// An application that opens the connection on its own (Windows App):
    /// there is no process to follow.
    External,
}

impl SessionKind {
    /// The kind in error messages, such as "nel terminale".
    pub(super) fn label(self) -> &'static str {
        match self {
            SessionKind::Terminal => "nel terminale",
            SessionKind::RdpExternal => "RDP in finestra propria",
            SessionKind::External => "in un'applicazione esterna",
        }
    }
}

/// The state of a session (ambiguity 11 of SPEC-UI §6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum SessionState {
    /// Started, nothing received yet.
    Connecting,
    /// A terminal showed its first output, an RDP client is running, an
    /// external application has the connection.
    Connected,
    /// The program ended, or the user closed the session.
    Ended {
        /// Its exit code.
        code: i32,
    },
    /// The connection did not succeed.
    Failed {
        /// The exit code, when the program ended with one.
        code: Option<i32>,
        /// What happened, in Italian; never a secret. For FreeRDP it is the
        /// end of its log: text that the frontend shows as untrusted.
        reason: String,
    },
}

/// The size of a terminal, in characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, specta::Type)]
pub struct TerminalSize {
    /// Columns.
    pub cols: u16,
    /// Rows.
    pub rows: u16,
}

/// A session just opened.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, specta::Type)]
pub struct SessionOpened {
    /// Its id.
    pub id: SessionId,
    /// Its kind, which picks the view of the tab.
    pub kind: SessionKind,
}

/// A session whose program is over: the `session-ended` event (SPEC-UI
/// §4.4). It may arrive before the frontend knows the id of a session that
/// ended at once.
#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type, tauri_specta::Event,
)]
pub struct SessionEnded {
    /// The session.
    pub id: SessionId,
    /// Its last state: [`SessionState::Ended`] or [`SessionState::Failed`].
    pub state: SessionState,
}
