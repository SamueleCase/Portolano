//! Portolano operations (DEC-018.2, DEC-022).
//!
//! The workflows that the CLI and the future app share: inserting an import,
//! composed validation, deletion with references and secrets, secrets,
//! creation and editing of nodes, moves and the preparation of `connect`.
//!
//! Functions here never print, never read a terminal and never exit the
//! process. They receive the [`Store`](portolano_core::Store), the document,
//! the [`Vault`](portolano_core::Vault) and the registries, and return
//! structured results. Questions to the user are two-phase APIs (a preview or
//! a request, then the operation), so that a graphical interface can show its
//! own dialogs.
//!
//! An operation that fails leaves the document it was given unchanged: it
//! works on a copy and replaces the caller's document only on success.

mod category;
mod commit;
pub mod config;
pub mod connect;
pub mod delete;
pub mod document;
pub mod duplicate;
mod error;
pub mod import;
pub mod inspect;
pub mod locations;
pub mod moves;
pub mod nodes;
pub mod secrets;
#[cfg(test)]
mod testutil;
pub mod trace;
mod tree_util;
pub mod usage;
pub mod validate;
pub mod vault;

pub use category::ErrorCategory;
pub use commit::CleanupFailure;
pub use error::OpsError;

/// Result type of the operations, with [`OpsError`] as the default error.
pub type Result<T, E = OpsError> = std::result::Result<T, E>;
