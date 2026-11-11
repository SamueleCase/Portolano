//! Secret vaults (SPEC §5.2): the [`Vault`] trait, its backends and
//! [`open_vault`].
//!
//! Documents only hold [`SecretRef`]s; the secret values live in a vault.

// Stub (WP0): the parameters are used once WP3 implements the bodies.
#![expect(unused_variables)]

mod file;
mod keychain;
mod memory;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

pub use file::{FileVault, KdfParams};
pub use keychain::{KEYCHAIN_SERVICE, KeychainVault};
pub use memory::MemoryVault;

use crate::error::VaultError;
use crate::secret::{Secret, SecretRef};

/// A store of secrets keyed by [`SecretRef`].
pub trait Vault: Send + Sync {
    /// Name of the backend: `"keychain"`, `"file"` or `"memory"`.
    fn backend(&self) -> &'static str;

    /// The secret for `reference`, or `None` if the vault has no such entry.
    ///
    /// # Errors
    /// Backend failures.
    fn get(&self, reference: &SecretRef) -> Result<Option<Secret>, VaultError>;

    /// Stores `secret` under `reference`, replacing any previous value.
    ///
    /// # Errors
    /// Backend failures.
    fn set(&self, reference: &SecretRef, secret: &Secret) -> Result<(), VaultError>;

    /// Removes the entry for `reference`. Idempotent: removing a missing
    /// entry succeeds.
    ///
    /// # Errors
    /// Backend failures.
    fn delete(&self, reference: &SecretRef) -> Result<(), VaultError>;

    /// Whether the vault has an entry for `reference`.
    ///
    /// # Errors
    /// Backend failures.
    fn contains(&self, reference: &SecretRef) -> Result<bool, VaultError> {
        Ok(self.get(reference)?.is_some())
    }
}

/// Which vault to open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VaultSelection {
    /// The system keychain.
    Keychain,
    /// The encrypted file vault at `path`.
    File {
        /// The vault file.
        path: PathBuf,
    },
    /// An empty in-memory vault (tests, hidden CLI option).
    Memory,
}

/// Provides the master password of the file vault, from a hidden prompt or
/// from `PORTOLANO_VAULT_PASSWORD`.
pub trait MasterPasswordSource {
    /// The master password; `creating` is true when the vault file does not
    /// exist yet, so that a prompt can ask for confirmation.
    ///
    /// # Errors
    /// [`VaultError::MasterPasswordUnavailable`] when no password can be
    /// obtained.
    fn master_password(&self, creating: bool) -> Result<Secret, VaultError>;
}

/// Opens the selected vault. The master password is requested only for the
/// file vault.
///
/// # Errors
/// The errors of the selected backend, such as
/// [`VaultError::BadMasterPassword`].
pub fn open_vault(
    selection: &VaultSelection,
    master_password: &dyn MasterPasswordSource,
) -> Result<Box<dyn Vault>, VaultError> {
    todo!()
}

/// Decrypted secrets held in memory by the memory and file backends.
type SecretMap = HashMap<SecretRef, Secret>;

/// Locks the secrets of a backend.
///
/// A poisoned lock means that a thread panicked in the middle of an update:
/// the map may be inconsistent with the backend, so it is reported as an
/// error instead of being used.
fn lock_secrets<'a>(
    secrets: &'a Mutex<SecretMap>,
    backend: &'static str,
) -> Result<MutexGuard<'a, SecretMap>, VaultError> {
    secrets.lock().map_err(|_| VaultError::Backend {
        backend,
        reason: "stato interno non più affidabile dopo un errore precedente".to_owned(),
    })
}

/// An owned copy of `secret`. `Secret` is not `Clone`, so that every copy of
/// a secret value is explicit; this one lives in a zeroizing buffer too.
fn copy_secret(secret: &Secret) -> Secret {
    Secret::new(secret.expose().to_owned())
}
