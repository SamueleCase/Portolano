//! Secret vaults (SPEC §5.2): the [`Vault`] trait, its backends and
//! [`open_vault`].
//!
//! Documents only hold [`SecretRef`]s; the secret values live in a vault.

mod file;
mod keychain;
mod memory;

use std::collections::HashMap;
use std::path::PathBuf;

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
    Ok(match selection {
        VaultSelection::Keychain => Box::new(KeychainVault),
        VaultSelection::File { path } => Box::new(FileVault::open(path.clone(), master_password)?),
        VaultSelection::Memory => Box::new(MemoryVault::default()),
    })
}

/// Secrets by reference: the content of the memory vault, and of the file
/// vault while one operation decrypts and rewrites it.
type SecretMap = HashMap<SecretRef, Secret>;

/// An owned copy of `secret`. `Secret` is not `Clone`, so that every copy of
/// a secret value is explicit; this one lives in a zeroizing buffer too.
fn copy_secret(secret: &Secret) -> Secret {
    Secret::new(secret.expose().to_owned())
}

#[cfg(test)]
mod testutil {
    use std::cell::Cell;

    use super::{KdfParams, MasterPasswordSource, Secret, VaultError};

    /// The only master password allowed in tests (SPEC §1.3).
    pub(super) const TEST_MASTER_PASSWORD: &str = "test-only-master-password";

    /// The cheapest Argon2id costs, to keep the file vault tests fast.
    pub(super) const TEST_KDF_PARAMS: KdfParams = KdfParams {
        m_cost_kib: 8,
        t_cost: 1,
        p_cost: 1,
    };

    /// A master password source that returns a fixed password and records
    /// the `creating` flag of its last call.
    pub(super) struct FixedPassword {
        password: &'static str,
        pub(super) last_creating: Cell<Option<bool>>,
    }

    impl FixedPassword {
        pub(super) fn new(password: &'static str) -> Self {
            Self {
                password,
                last_creating: Cell::new(None),
            }
        }

        pub(super) fn test() -> Self {
            Self::new(TEST_MASTER_PASSWORD)
        }
    }

    impl MasterPasswordSource for FixedPassword {
        fn master_password(&self, creating: bool) -> Result<Secret, VaultError> {
            self.last_creating.set(Some(creating));
            Ok(Secret::new(self.password.to_owned()))
        }
    }

    /// A master password source that is never expected to be asked.
    pub(super) struct NoPassword;

    impl MasterPasswordSource for NoPassword {
        fn master_password(&self, _creating: bool) -> Result<Secret, VaultError> {
            Err(VaultError::MasterPasswordUnavailable {
                reason: "not expected in this test".to_owned(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::testutil::{FixedPassword, NoPassword, TEST_KDF_PARAMS};
    use super::*;

    #[test]
    fn open_vault_memory_is_empty_and_needs_no_password() {
        let vault = open_vault(&VaultSelection::Memory, &NoPassword).unwrap();

        assert_eq!(vault.backend(), "memory");
        assert!(!vault.contains(&SecretRef::new()).unwrap());
    }

    #[test]
    fn open_vault_keychain_needs_no_password() {
        let vault = open_vault(&VaultSelection::Keychain, &NoPassword).unwrap();

        assert_eq!(vault.backend(), "keychain");
    }

    #[test]
    fn open_vault_file_opens_the_vault_at_the_given_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault.json");
        let reference = SecretRef::new();
        FileVault::with_kdf_params(&path, &FixedPassword::test(), TEST_KDF_PARAMS)
            .unwrap()
            .set(&reference, &Secret::new("sentinel-open-vault".to_owned()))
            .unwrap();

        let vault = open_vault(&VaultSelection::File { path }, &FixedPassword::test()).unwrap();

        assert_eq!(vault.backend(), "file");
        assert!(vault.contains(&reference).unwrap());
    }

    #[test]
    fn open_vault_file_reports_the_password_source_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault.json");

        let result = open_vault(&VaultSelection::File { path }, &NoPassword);

        assert!(matches!(
            result,
            Err(VaultError::MasterPasswordUnavailable { .. })
        ));
    }
}
