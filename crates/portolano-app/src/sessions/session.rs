//! What every kind of session offers to the manager.

use super::error::SessionError;
use super::model::{SessionKind, SessionState, TerminalSize};

/// Called once by a session whose program is over, with its last state.
pub(super) type Finish = Box<dyn FnOnce(SessionState) + Send>;

/// A running session of some kind. A new kind (the integrated RDP panel)
/// implements it without changing the manager, the channels or the
/// commands (SPEC-UI §4.5).
pub(super) trait Session: Send + Sync {
    /// Its kind.
    fn kind(&self) -> SessionKind;

    /// Sends what the user typed to the program.
    ///
    /// # Errors
    /// [`SessionError::Unsupported`] unless the session has a terminal.
    fn input(&self, _bytes: &[u8]) -> Result<(), SessionError> {
        Err(SessionError::Unsupported {
            kind: self.kind(),
            operation: "l'input da tastiera",
        })
    }

    /// Tells the program the new size of its terminal.
    ///
    /// # Errors
    /// [`SessionError::Unsupported`] unless the session has a terminal.
    fn resize(&self, _size: TerminalSize) -> Result<(), SessionError> {
        Err(SessionError::Unsupported {
            kind: self.kind(),
            operation: "il ridimensionamento",
        })
    }

    /// Ends the program, if there is one still running; its last state
    /// follows through [`Finish`].
    fn close(&self);
}
