//! Portolano core library.
//!
//! Data model, tree operations, credential resolution, JSON store with schema
//! migrations, icon store, secret vaults and application configuration.
//! It knows nothing about SSH, RDP or Royal (SPEC §1.5).

pub mod error;

pub use error::{Error, Issue, Result, Severity, VaultError};
