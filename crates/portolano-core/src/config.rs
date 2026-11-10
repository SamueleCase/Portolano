//! Application configuration, `config.toml` (SPEC §9, DEC-003).
//!
//! The core types only the sections it owns (`default_document`, `[vault]`,
//! `[terminal]`). Every other top-level section, such as `[ssh]` or `[rdp]`,
//! is kept as raw TOML and interpreted by the protocol it belongs to, so that
//! a new protocol does not change the core (SPEC §1.5).
//!
//! Default locations (SPEC §6.1) are inside [`data_dir`]: the document
//! [`DOCUMENT_FILE_NAME`], the configuration [`CONFIG_FILE_NAME`] and the file
//! vault [`VAULT_FILE_NAME`].

// Stub (WP0): the parameters are used once WP2 implements the bodies.
#![expect(unused_variables)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::Result;

/// File name of the default document inside [`data_dir`].
pub const DOCUMENT_FILE_NAME: &str = "portolano.json";

/// File name of the configuration inside [`data_dir`].
pub const CONFIG_FILE_NAME: &str = "config.toml";

/// File name of the file vault inside [`data_dir`].
pub const VAULT_FILE_NAME: &str = "vault.json";

/// The per-user data directory of Portolano,
/// `ProjectDirs::from("dev", "portolano", "Portolano")`; on macOS
/// `~/Library/Application Support/dev.portolano.Portolano/`.
///
/// # Errors
/// [`Error::NoDataDir`](crate::Error::NoDataDir) if there is no home
/// directory.
pub fn data_dir() -> Result<PathBuf> {
    todo!()
}

/// Contents of `config.toml`. Every field is optional in the file.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    /// Document used when neither `--file` nor `PORTOLANO_FILE` is given.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_document: Option<PathBuf>,
    /// `[vault]` section.
    #[serde(default)]
    pub vault: VaultConfig,
    /// `[terminal]` section.
    #[serde(default)]
    pub terminal: TerminalConfig,
    /// The other top-level sections, keyed by name (for example `"ssh"`,
    /// `"rdp"`), left to the protocols to interpret.
    #[serde(flatten)]
    pub protocols: BTreeMap<String, toml::Value>,
}

impl AppConfig {
    /// Loads the configuration from `config.toml` in [`data_dir`]; defaults
    /// if the file does not exist.
    ///
    /// # Errors
    /// [`Error::NoDataDir`](crate::Error::NoDataDir), plus the errors of
    /// [`AppConfig::load`].
    pub fn load_default() -> Result<AppConfig> {
        todo!()
    }

    /// Loads the configuration from `path`; defaults if the file does not
    /// exist.
    ///
    /// # Errors
    /// [`Error::InvalidConfig`](crate::Error::InvalidConfig),
    /// [`Error::Io`](crate::Error::Io).
    pub fn load(path: &Path) -> Result<AppConfig> {
        todo!()
    }

    /// Writes the configuration to `path`, creating its directory if needed.
    ///
    /// # Errors
    /// [`Error::SerializeConfig`](crate::Error::SerializeConfig),
    /// [`Error::Io`](crate::Error::Io).
    pub fn save(&self, path: &Path) -> Result<()> {
        todo!()
    }
}

/// `[vault]` section.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct VaultConfig {
    /// Vault backend.
    pub backend: VaultBackend,
    /// Path of the file vault; `None` means [`VAULT_FILE_NAME`] in
    /// [`data_dir`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<PathBuf>,
}

/// Vault backend selectable in the configuration. The in-memory vault is only
/// available through the hidden CLI option.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VaultBackend {
    /// The system keychain (macOS Keychain, Windows Credential Manager, Linux
    /// Secret Service).
    #[default]
    Keychain,
    /// The encrypted file vault.
    File,
}

/// `[terminal]` section: the terminal used by `connect --new-window`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TerminalConfig {
    /// Terminal application.
    pub app: TerminalApp,
    /// Command for [`TerminalApp::Custom`], with the placeholder `{cmd}`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
}

/// Terminal application for new windows (SPEC §7.2).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminalApp {
    /// macOS Terminal.
    #[default]
    Terminal,
    /// iTerm2.
    Iterm,
    /// Ghostty.
    Ghostty,
    /// The command in [`TerminalConfig::command`].
    Custom,
}
