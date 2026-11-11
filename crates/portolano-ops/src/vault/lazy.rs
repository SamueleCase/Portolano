//! A vault opened on first use.

use std::sync::{Mutex, MutexGuard};

use portolano_core::{
    MasterPasswordSource, Secret, SecretRef, Vault, VaultError, VaultSelection, open_vault,
};

/// Shortest master password accepted when a file vault is created, in
/// characters (DEC-024.7): the vault file may be copied elsewhere and
/// attacked offline, where only the length of the password slows down the
/// guessing that Argon2id makes costly. Existing vaults still open with
/// their password.
pub const MIN_MASTER_PASSWORD_CHARS: usize = 12;

/// A vault that is opened only when a secret is first read or written.
///
/// Opening the file vault asks for the master password, and the keychain may
/// ask the system for access: operations that end up touching no secret,
/// such as deleting a node without secrets, then ask nothing. A failed
/// opening is not remembered, so the next use tries again. A new file vault
/// is created only with a master password of at least
/// [`MIN_MASTER_PASSWORD_CHARS`] characters.
pub struct LazyVault {
    selection: VaultSelection,
    master_password: Box<dyn MasterPasswordSource + Send + Sync>,
    opened: Mutex<Option<Box<dyn Vault>>>,
}

impl LazyVault {
    /// A vault that will open `selection`, asking `master_password` if it is
    /// the file vault.
    pub fn new(
        selection: VaultSelection,
        master_password: Box<dyn MasterPasswordSource + Send + Sync>,
    ) -> Self {
        Self {
            selection,
            master_password,
            opened: Mutex::new(None),
        }
    }

    /// Whether the vault has been opened.
    pub fn is_open(&self) -> bool {
        self.slot().is_ok_and(|slot| slot.is_some())
    }

    /// Runs `operation` on the vault, opening it first if needed.
    fn with_vault<T>(
        &self,
        operation: impl FnOnce(&dyn Vault) -> Result<T, VaultError>,
    ) -> Result<T, VaultError> {
        let mut slot = self.slot()?;
        let vault = match slot.take() {
            Some(vault) => vault,
            None => open_vault(
                &self.selection,
                &NewVaultPolicy(self.master_password.as_ref()),
            )?,
        };
        let result = operation(vault.as_ref());
        *slot = Some(vault);
        result
    }

    fn slot(&self) -> Result<MutexGuard<'_, Option<Box<dyn Vault>>>, VaultError> {
        self.opened.lock().map_err(|_| VaultError::Backend {
            backend: self.backend(),
            reason: "stato interno non più affidabile dopo un errore precedente".to_owned(),
        })
    }
}

/// The master password source of the caller, refusing a short password for
/// a vault being created: the vault file is written only after the source
/// answers, so a refused password leaves nothing behind.
struct NewVaultPolicy<'a>(&'a dyn MasterPasswordSource);

impl MasterPasswordSource for NewVaultPolicy<'_> {
    fn master_password(&self, creating: bool) -> Result<Secret, VaultError> {
        let password = self.0.master_password(creating)?;
        if creating && password.expose().chars().count() < MIN_MASTER_PASSWORD_CHARS {
            return Err(VaultError::WeakMasterPassword {
                min_chars: MIN_MASTER_PASSWORD_CHARS,
            });
        }
        Ok(password)
    }
}

impl Vault for LazyVault {
    fn backend(&self) -> &'static str {
        match self.selection {
            VaultSelection::Keychain => "keychain",
            VaultSelection::File { .. } => "file",
            VaultSelection::Memory => "memory",
        }
    }

    fn get(&self, reference: &SecretRef) -> Result<Option<Secret>, VaultError> {
        self.with_vault(|vault| vault.get(reference))
    }

    fn set(&self, reference: &SecretRef, secret: &Secret) -> Result<(), VaultError> {
        self.with_vault(|vault| vault.set(reference, secret))
    }

    fn delete(&self, reference: &SecretRef) -> Result<(), VaultError> {
        self.with_vault(|vault| vault.delete(reference))
    }

    fn contains(&self, reference: &SecretRef) -> Result<bool, VaultError> {
        self.with_vault(|vault| vault.contains(reference))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use pretty_assertions::assert_eq;

    use super::*;

    /// Counts how many times the master password is asked.
    struct CountingSource(Arc<AtomicUsize>);

    impl MasterPasswordSource for CountingSource {
        fn master_password(&self, _creating: bool) -> Result<Secret, VaultError> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok(Secret::new("test-only-master-password".to_owned()))
        }
    }

    struct FailingSource;

    impl MasterPasswordSource for FailingSource {
        fn master_password(&self, _creating: bool) -> Result<Secret, VaultError> {
            Err(VaultError::MasterPasswordUnavailable {
                reason: "nessun terminale".to_owned(),
            })
        }
    }

    /// Fails the first time, then answers the test password.
    struct FailsOnce(AtomicUsize);

    impl MasterPasswordSource for FailsOnce {
        fn master_password(&self, _creating: bool) -> Result<Secret, VaultError> {
            if self.0.fetch_add(1, Ordering::SeqCst) == 0 {
                return Err(VaultError::MasterPasswordUnavailable {
                    reason: "annullato".to_owned(),
                });
            }
            Ok(Secret::new("test-only-master-password".to_owned()))
        }
    }

    /// The test master password cut one character short of the minimum.
    const SHORT_PASSWORD: &str = "test-only-m";

    /// Always answers [`SHORT_PASSWORD`].
    struct ShortSource;

    impl MasterPasswordSource for ShortSource {
        fn master_password(&self, _creating: bool) -> Result<Secret, VaultError> {
            Ok(Secret::new(SHORT_PASSWORD.to_owned()))
        }
    }

    #[test]
    fn new_file_vault_refuses_a_short_master_password() {
        assert_eq!(
            SHORT_PASSWORD.chars().count(),
            MIN_MASTER_PASSWORD_CHARS - 1
        );
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault.json");
        let vault = LazyVault::new(
            VaultSelection::File { path: path.clone() },
            Box::new(ShortSource),
        );

        let result = vault.contains(&SecretRef::new());

        assert!(
            matches!(
                result,
                Err(VaultError::WeakMasterPassword { min_chars: 12 })
            ),
            "{result:?}"
        );
        assert!(!path.exists());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("almeno 12 caratteri")
        );
    }

    #[test]
    fn existing_file_vault_opens_with_a_short_master_password() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault.json");
        // Created before the rule, without the policy of `LazyVault`.
        portolano_core::vault::FileVault::with_kdf_params(
            path.clone(),
            &ShortSource,
            crate::testutil::TEST_KDF_PARAMS,
        )
        .unwrap();
        let vault = LazyVault::new(VaultSelection::File { path }, Box::new(ShortSource));

        assert!(!vault.contains(&SecretRef::new()).unwrap());
    }

    fn memory_vault() -> LazyVault {
        LazyVault::new(VaultSelection::Memory, Box::new(FailingSource))
    }

    #[test]
    fn backend_is_known_without_opening() {
        let vault = LazyVault::new(
            VaultSelection::File {
                path: "/nonexistent/vault.json".into(),
            },
            Box::new(FailingSource),
        );
        assert_eq!(vault.backend(), "file");
        assert!(!vault.is_open());
    }

    #[test]
    fn vault_opens_on_first_use_and_keeps_its_content() {
        let vault = memory_vault();
        let reference = SecretRef::new();
        vault
            .set(&reference, &Secret::new("sentinel-lazy".to_owned()))
            .unwrap();
        assert!(vault.is_open());
        assert_eq!(
            vault.get(&reference).unwrap().unwrap().expose(),
            "sentinel-lazy"
        );
    }

    #[test]
    fn master_password_is_asked_once() {
        let dir = tempfile::tempdir().unwrap();
        let asked = Arc::new(AtomicUsize::new(0));
        let vault = LazyVault::new(
            VaultSelection::File {
                path: crate::testutil::fast_file_vault(dir.path()),
            },
            Box::new(CountingSource(Arc::clone(&asked))),
        );
        let reference = SecretRef::new();
        vault
            .set(&reference, &Secret::new("sentinel-lazy".to_owned()))
            .unwrap();
        assert!(vault.contains(&reference).unwrap());
        assert_eq!(asked.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn failed_opening_is_reported_and_retried() {
        let dir = tempfile::tempdir().unwrap();
        let vault = LazyVault::new(
            VaultSelection::File {
                path: dir.path().join("vault.json"),
            },
            Box::new(FailingSource),
        );
        let first = vault.contains(&SecretRef::new());
        assert!(matches!(
            first,
            Err(VaultError::MasterPasswordUnavailable { .. })
        ));
        assert!(!vault.is_open());
    }

    #[test]
    fn vault_opens_on_the_use_after_a_failed_opening() {
        let dir = tempfile::tempdir().unwrap();
        let vault = LazyVault::new(
            VaultSelection::File {
                path: crate::testutil::fast_file_vault(dir.path()),
            },
            Box::new(FailsOnce(AtomicUsize::new(0))),
        );
        let reference = SecretRef::new();
        assert!(vault.contains(&reference).is_err());
        assert!(!vault.contains(&reference).unwrap());
        assert!(vault.is_open());
    }
}
