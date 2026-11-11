//! What every command needs: configuration, document location, vault and
//! registries, resolved from the global options (SPEC §9, §10.1).

use std::path::{Path, PathBuf};

use anyhow::Result;
use portolano_connectors::Registry as Protocols;
use portolano_core::{AppConfig, Document, Store};
use portolano_import::Registry as Importers;
use portolano_ops::document::open_document;
use portolano_ops::locations::{GlobalOptions, Locations, config_path};
use portolano_ops::vault::LazyVault;

use crate::args::GlobalArgs;
use crate::prompt::MasterPassword;

/// The resolved global options.
pub struct Context {
    /// The options as given (flags or environment variables).
    pub options: GlobalOptions,
    /// The configuration file.
    pub config_path: PathBuf,
    /// The configuration.
    pub config: AppConfig,
    /// Document and vault.
    pub locations: Locations,
    /// Output JSON.
    pub json: bool,
    /// The protocols.
    pub protocols: Protocols,
    /// The importers.
    pub importers: Importers,
}

impl Context {
    /// Loads the configuration and resolves the locations.
    ///
    /// # Errors
    /// An invalid configuration file, a vault file with another backend, no
    /// home directory when a default location is needed.
    pub fn new(args: &GlobalArgs) -> Result<Context> {
        let options = GlobalOptions {
            file: args.file.clone(),
            vault: args.vault.map(Into::into),
            vault_file: args.vault_file.clone(),
            config: args.config.clone(),
        };
        let config_path = config_path(options.config.as_deref())?;
        let config = AppConfig::load(&config_path)?;
        let locations = Locations::resolve(&options, &config)?;
        Ok(Context {
            options,
            config_path,
            config,
            locations,
            json: args.json,
            protocols: Protocols::with_defaults(),
            importers: Importers::with_defaults(),
        })
    }

    /// Opens and loads the document.
    ///
    /// # Errors
    /// A missing or invalid document.
    pub fn open(&self) -> Result<(Store, Document)> {
        Ok(open_document(&self.locations.document)?)
    }

    /// The vault, opened on first use.
    pub fn vault(&self) -> LazyVault {
        LazyVault::new(self.locations.vault.clone(), Box::new(MasterPassword))
    }

    /// The global options that a new window must receive to use the same
    /// document, vault and configuration (DEC-004, DEC-022.8): the document
    /// always, the others when given, as absolute paths because the new
    /// terminal neither inherits the environment nor the directory. The
    /// master password is never passed.
    pub fn relaunch_args(&self) -> Vec<String> {
        let mut args = vec!["--file".to_owned(), absolute(&self.locations.document)];
        if let Some(vault) = self.options.vault {
            args.extend(["--vault".to_owned(), vault.name().to_owned()]);
        }
        if let Some(path) = &self.options.vault_file {
            args.extend(["--vault-file".to_owned(), absolute(path)]);
        }
        if let Some(path) = &self.options.config {
            args.extend(["--config".to_owned(), absolute(path)]);
        }
        args
    }
}

fn absolute(path: &Path) -> String {
    std::path::absolute(path)
        .unwrap_or_else(|_| path.to_owned())
        .to_string_lossy()
        .into_owned()
}
