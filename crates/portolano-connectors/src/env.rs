//! Discovery of the client programs available on this machine.

// Stub (WP0): the parameters are used once WP4 implements the bodies.
#![expect(unused_variables)]

use std::path::PathBuf;

use portolano_core::AppConfig;

use crate::ConnectError;

/// Operating system, as far as launching clients is concerned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    /// macOS.
    MacOs,
    /// Linux.
    Linux,
    /// Windows.
    Windows,
    /// Any other system.
    Other,
}

/// The programs a plan can use. Built by [`Environment::discover`]; tests
/// build it by hand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Environment {
    /// The current operating system.
    pub os: Os,
    /// `ssh`, from `[ssh] program` or the `PATH`.
    pub ssh: Option<PathBuf>,
    /// `ssh-add`.
    pub ssh_add: Option<PathBuf>,
    /// The FreeRDP client, from `[rdp] freerdp_program` or the search of
    /// SPEC §7.3.
    pub freerdp: Option<PathBuf>,
    /// The Windows App (or Microsoft Remote Desktop) bundle.
    pub windows_app: Option<PathBuf>,
    /// The running `portolano` executable, used for askpass and new windows.
    pub self_exe: PathBuf,
}

impl Environment {
    /// Looks for the client programs, honoring the overrides in `config`.
    ///
    /// # Errors
    /// [`ConnectError::InvalidConfig`] for an invalid protocol section,
    /// [`ConnectError::Io`] if the running executable cannot be determined.
    pub fn discover(config: &AppConfig) -> Result<Environment, ConnectError> {
        todo!()
    }
}
