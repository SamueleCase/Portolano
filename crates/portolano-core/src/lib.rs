//! Portolano core library.
//!
//! Data model, tree operations, credential resolution, JSON store with schema
//! migrations, icon store, secret vaults and application configuration.
//! It knows nothing about SSH, RDP or Royal (SPEC §1.5).

pub mod config;
pub mod error;
pub mod icons;
pub mod migrate;
pub mod model;
pub mod secret;
pub mod store;

mod path;
mod resolve;
mod tree;
mod validate;

pub use config::AppConfig;
pub use error::{Error, Issue, Result, Severity, VaultError};
pub use icons::IconStore;
pub use model::{
    Connection, Credential, CredentialSpec, CustomProperty, Document, FORMAT_ID, Folder, Icon,
    Node, NodeKind, PrivateKey, PropertyValue, SCHEMA_VERSION, SourceInfo, Unsupported,
};
pub use resolve::{CredentialOrigin, ResolvedCredential};
pub use secret::{Secret, SecretRef};
pub use store::Store;
