//! Where the document, the configuration and the vault are (SPEC §6.1, §9).
//!
//! Precedence is flag > environment variable > configuration > default. The
//! caller merges the first two (clap does it for the CLI) into
//! [`GlobalOptions`]; this module applies the configuration and the defaults.

use std::path::{Path, PathBuf};

use crate::{OpsError, Result};
use portolano_core::config::{
    CONFIG_FILE_NAME, DOCUMENT_FILE_NAME, VAULT_FILE_NAME, VaultBackend, data_dir,
};
use portolano_core::{AppConfig, VaultSelection};

/// A vault backend chosen explicitly (flag or environment variable).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultChoice {
    /// The system keychain.
    Keychain,
    /// The encrypted file vault.
    File,
    /// An empty in-memory vault (tests, hidden CLI option).
    Memory,
}

impl VaultChoice {
    /// The backend name, as returned by `Vault::backend`.
    pub fn name(self) -> &'static str {
        match self {
            VaultChoice::Keychain => "keychain",
            VaultChoice::File => "file",
            VaultChoice::Memory => "memory",
        }
    }
}

/// Locations chosen explicitly, by flags or environment variables.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GlobalOptions {
    /// The document file.
    pub file: Option<PathBuf>,
    /// The vault backend.
    pub vault: Option<VaultChoice>,
    /// The file of the file vault.
    pub vault_file: Option<PathBuf>,
    /// The configuration file.
    pub config: Option<PathBuf>,
}

/// The configuration file: the explicit one, or [`CONFIG_FILE_NAME`] in the
/// data directory.
///
/// # Errors
/// [`portolano_core::Error::NoDataDir`] without a home directory.
pub fn config_path(explicit: Option<&Path>) -> Result<PathBuf> {
    match explicit {
        Some(path) => Ok(path.to_owned()),
        None => Ok(data_dir()?.join(CONFIG_FILE_NAME)),
    }
}

/// The effective document and vault.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Locations {
    /// The document file.
    pub document: PathBuf,
    /// The vault to open.
    pub vault: VaultSelection,
}

impl Locations {
    /// Applies the configuration and the defaults to the explicit options.
    ///
    /// A vault file given without a backend implies the file backend; given
    /// with another explicit backend it is an error (DEC-022.6).
    ///
    /// # Errors
    /// [`OpsError::VaultFileConflict`],
    /// [`portolano_core::Error::NoDataDir`] when a default is needed and
    /// there is no home directory.
    pub fn resolve(options: &GlobalOptions, config: &AppConfig) -> Result<Locations> {
        let document = match options.file.clone().or(config.default_document.clone()) {
            Some(path) => path,
            None => data_dir()?.join(DOCUMENT_FILE_NAME),
        };
        Ok(Locations {
            document,
            vault: vault_selection(options, config)?,
        })
    }
}

fn vault_selection(options: &GlobalOptions, config: &AppConfig) -> Result<VaultSelection> {
    let explicit_file = options.vault_file.clone();
    match (options.vault, explicit_file) {
        (Some(choice @ (VaultChoice::Keychain | VaultChoice::Memory)), Some(_)) => {
            Err(OpsError::VaultFileConflict {
                backend: choice.name(),
            })
        }
        (Some(VaultChoice::Keychain), None) => Ok(VaultSelection::Keychain),
        (Some(VaultChoice::Memory), None) => Ok(VaultSelection::Memory),
        (Some(VaultChoice::File), file) | (None, file @ Some(_)) => file_vault(file, config),
        (None, None) => match config.vault.backend {
            VaultBackend::Keychain => Ok(VaultSelection::Keychain),
            VaultBackend::File => file_vault(None, config),
        },
    }
}

fn file_vault(explicit: Option<PathBuf>, config: &AppConfig) -> Result<VaultSelection> {
    let path = match explicit.or(config.vault.file.clone()) {
        Some(path) => path,
        None => data_dir()?.join(VAULT_FILE_NAME),
    };
    Ok(VaultSelection::File { path })
}

#[cfg(test)]
mod tests {
    use portolano_core::config::VaultConfig;
    use pretty_assertions::assert_eq;

    use super::*;

    fn options(vault: Option<VaultChoice>, vault_file: Option<&str>) -> GlobalOptions {
        GlobalOptions {
            vault,
            vault_file: vault_file.map(PathBuf::from),
            ..GlobalOptions::default()
        }
    }

    fn file_config(path: Option<&str>) -> AppConfig {
        AppConfig {
            vault: VaultConfig {
                backend: VaultBackend::File,
                file: path.map(PathBuf::from),
            },
            ..AppConfig::default()
        }
    }

    #[test]
    fn explicit_file_wins_over_the_configured_document() {
        let config = AppConfig {
            default_document: Some(PathBuf::from("/config/doc.json")),
            ..AppConfig::default()
        };
        let options = GlobalOptions {
            file: Some(PathBuf::from("/flag/doc.json")),
            ..GlobalOptions::default()
        };
        let locations = Locations::resolve(&options, &config).unwrap();
        assert_eq!(locations.document, PathBuf::from("/flag/doc.json"));
    }

    #[test]
    fn configured_document_is_used_without_a_flag() {
        let config = AppConfig {
            default_document: Some(PathBuf::from("/config/doc.json")),
            ..AppConfig::default()
        };
        let locations = Locations::resolve(&GlobalOptions::default(), &config).unwrap();
        assert_eq!(locations.document, PathBuf::from("/config/doc.json"));
    }

    #[test]
    fn explicit_backend_wins_over_the_configuration() {
        let selection = vault_selection(
            &options(Some(VaultChoice::Memory), None),
            &file_config(None),
        )
        .unwrap();
        assert_eq!(selection, VaultSelection::Memory);
    }

    #[test]
    fn configured_file_backend_uses_the_configured_path() {
        let selection =
            vault_selection(&options(None, None), &file_config(Some("/cfg/vault.json"))).unwrap();
        assert_eq!(
            selection,
            VaultSelection::File {
                path: PathBuf::from("/cfg/vault.json")
            }
        );
    }

    #[test]
    fn vault_file_alone_implies_the_file_backend() {
        let selection = vault_selection(
            &options(None, Some("/flag/vault.json")),
            &AppConfig::default(),
        )
        .unwrap();
        assert_eq!(
            selection,
            VaultSelection::File {
                path: PathBuf::from("/flag/vault.json")
            }
        );
    }

    #[test]
    fn explicit_vault_file_wins_over_the_configured_one() {
        let selection = vault_selection(
            &options(Some(VaultChoice::File), Some("/flag/vault.json")),
            &file_config(Some("/cfg/vault.json")),
        )
        .unwrap();
        assert_eq!(
            selection,
            VaultSelection::File {
                path: PathBuf::from("/flag/vault.json")
            }
        );
    }

    #[test]
    fn vault_file_with_the_keychain_backend_is_rejected() {
        let result = vault_selection(
            &options(Some(VaultChoice::Keychain), Some("/flag/vault.json")),
            &AppConfig::default(),
        );
        assert!(matches!(
            result,
            Err(OpsError::VaultFileConflict {
                backend: "keychain"
            })
        ));
    }

    #[test]
    fn explicit_config_path_is_used_as_is() {
        let path = config_path(Some(Path::new("/flag/config.toml"))).unwrap();
        assert_eq!(path, PathBuf::from("/flag/config.toml"));
    }
}
