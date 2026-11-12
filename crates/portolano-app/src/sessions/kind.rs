//! The kind of session that runs a launch plan (DEC-030.4).
//!
//! A plan does not name its client, but its shape tells how it must be
//! shown, by the rules of the connectors (SPEC §7.2, §7.3):
//! - Windows App first writes the `.rdp` file it opens ([`LaunchStep::WriteTempFile`]);
//! - FreeRDP runs detached (`wait = false`) unless it has to ask for a
//!   password (DEC-025), and then it runs in the foreground;
//! - `ssh` is always waited for, since the askpass server must outlive it.
//!
//! So a waited program is a terminal, a detached one an RDP window.

use portolano_connectors::{LaunchPlan, LaunchStep};

use super::error::SessionError;
use super::model::SessionKind;

/// The kind of session for `plan`.
///
/// # Errors
/// [`SessionError::NotASession`] for a plan that opens a new terminal
/// window (see [`super::open_in_new_window`]) or does not end with a
/// program.
pub fn session_kind(plan: &LaunchPlan) -> Result<SessionKind, SessionError> {
    match plan.steps.last() {
        Some(LaunchStep::Run { wait, .. }) => {
            let writes_files = plan
                .steps
                .iter()
                .any(|step| matches!(step, LaunchStep::WriteTempFile { .. }));
            Ok(match (writes_files, *wait) {
                (true, _) => SessionKind::External,
                (false, true) => SessionKind::Terminal,
                (false, false) => SessionKind::RdpExternal,
            })
        }
        Some(LaunchStep::OpenInTerminal { .. }) => Err(SessionError::NotASession {
            reason: "apre una nuova finestra del terminale",
        }),
        Some(LaunchStep::SshAddKey { .. } | LaunchStep::WriteTempFile { .. }) | None => {
            Err(SessionError::NotASession {
                reason: "non termina con un programma da eseguire",
            })
        }
    }
}

#[cfg(test)]
mod tests;
