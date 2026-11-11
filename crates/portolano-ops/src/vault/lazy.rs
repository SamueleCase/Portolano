//! A vault opened on first use.

use std::sync::{Mutex, MutexGuard};

use portolano_core::{
    MasterPasswordSource, Secret, SecretRef, Vault, VaultError, VaultSelection, open_vault,
};

/// A vault that is opened only when a secret is first read or written.
///
/// Opening the file vault asks for the master password, and the keychain may
/// ask the system for access: operations that end up touching no secret,
/// such as deleting a node without secrets, then ask nothing. A failed
/// opening is not remembered, so the next use tries again.
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
            None => open_vault(&self.selection, self.master_password.as_ref())?,
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
