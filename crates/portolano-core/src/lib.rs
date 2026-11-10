//! Portolano core library.
//!
//! Data model, tree operations, credential resolution, JSON store with schema
//! migrations, icon store, secret vaults and application configuration.
//! It knows nothing about SSH, RDP or Royal (SPEC §1.5).

pub mod error;
pub mod model;
pub mod secret;

mod path;
mod resolve;
mod tree;
mod validate;

pub use error::{Error, Issue, Result, Severity, VaultError};
pub use model::{
    Connection, Credential, CredentialSpec, CustomProperty, Document, FORMAT_ID, Folder, Icon,
    Node, NodeKind, PrivateKey, PropertyValue, SCHEMA_VERSION, SourceInfo, Unsupported,
};
pub use resolve::{CredentialOrigin, ResolvedCredential};
pub use secret::{Secret, SecretRef};
