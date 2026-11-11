//! Vault wrappers: [`LazyVault`] opens the vault on first use, and
//! [`SessionVault`] keeps the secrets typed for one connection in memory,
//! over the real vault (DEC-016.3).

mod lazy;
mod session;

pub use lazy::{LazyVault, MIN_MASTER_PASSWORD_CHARS};
pub use session::{Login, SessionVault};
