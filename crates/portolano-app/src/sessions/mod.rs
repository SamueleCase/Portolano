//! Sessions of the app (SPEC-UI §4.5): the programs that open connections,
//! followed in the tabs whatever their protocol.
//!
//! A launch plan of `portolano-ops` is prepared by the connectors
//! (DEC-016.2), its [`SessionKind`] comes from its shape
//! ([`session_kind`]) and the [`SessionManager`] runs it: in a terminal of
//! the app, as FreeRDP in its own window, or handed to Windows App. What the
//! tab shows goes through a [`SessionSink`], raw output frames and states,
//! so this module does not depend on the windows of Tauri and is tested
//! without them. No secret ever reaches a sink.

mod channel;
mod error;
mod external;
pub(crate) mod ipc;
mod kind;
mod manager;
mod model;
mod process;
mod rdp;
mod session;
mod terminal;

use portolano_connectors::{Environment, LaunchPlan, LaunchStep};
use portolano_core::AppConfig;

pub use channel::{FRAME_VERSION, FrameKind, SessionSink, frame};
pub use error::SessionError;
pub use external::open_in_new_window;
pub use kind::session_kind;
pub use manager::SessionManager;
pub use model::{SessionEnded, SessionId, SessionKind, SessionOpened, SessionState, TerminalSize};

use crate::sidecar::{self, SidecarError};

/// The programs available to the plans of the app. Askpass and new windows
/// run the CLI shipped with the app, not the app itself (SPEC-UI §4.5): the
/// plans name it where it should be, and [`require_cli`] checks that it is
/// there only for the plans that run it.
///
/// # Errors
/// [`SessionError::Connect`] for a refused configuration,
/// [`SessionError::Sidecar`] if the path of the app is unknown.
pub fn environment(config: &AppConfig) -> Result<Environment, SessionError> {
    let mut environment = Environment::discover(config)?;
    environment.self_exe = match sidecar::cli_path() {
        Ok(path) | Err(SidecarError::Missing { path }) => path,
        Err(error) => return Err(error.into()),
    };
    Ok(environment)
}

/// Checks that the CLI of the app is there if `plan` runs it: a FreeRDP or
/// Windows App connection works without it.
///
/// # Errors
/// [`SessionError::Sidecar`] if the plan runs the CLI and it is missing.
pub fn require_cli(plan: &LaunchPlan) -> Result<(), SessionError> {
    if runs_the_cli(plan) {
        sidecar::cli_path()?;
    }
    Ok(())
}

/// Whether `plan` runs the CLI of the app: to answer askpass prompts, or as
/// the command of a new window.
fn runs_the_cli(plan: &LaunchPlan) -> bool {
    plan.steps.iter().any(|step| match step {
        LaunchStep::SshAddKey { passphrase, .. } => passphrase.is_some(),
        LaunchStep::Run { askpass, .. } => askpass
            .as_ref()
            .is_some_and(|spec| !spec.entries.is_empty()),
        LaunchStep::OpenInTerminal { .. } => true,
        LaunchStep::WriteTempFile { .. } => false,
    })
}

#[cfg(test)]
pub(crate) mod tests;
