//! In-memory vault, for tests and the hidden `--vault memory` option.

use std::sync::Mutex;

use super::{SecretMap, Vault, copy_secret, lock_secrets};
use crate::error::VaultError;
use crate::secret::{Secret, SecretRef};

const BACKEND: &str = "memory";

/// A vault that keeps secrets in a map; its content is lost on drop.
#[derive(Debug, Default)]
pub struct MemoryVault {
    entries: Mutex<SecretMap>,
}

impl Vault for MemoryVault {
    fn backend(&self) -> &'static str {
        BACKEND
    }

    fn get(&self, reference: &SecretRef) -> Result<Option<Secret>, VaultError> {
        Ok(lock_secrets(&self.entries, BACKEND)?
            .get(reference)
            .map(copy_secret))
    }

    fn set(&self, reference: &SecretRef, secret: &Secret) -> Result<(), VaultError> {
        lock_secrets(&self.entries, BACKEND)?.insert(reference.clone(), copy_secret(secret));
        Ok(())
    }

    fn delete(&self, reference: &SecretRef) -> Result<(), VaultError> {
        lock_secrets(&self.entries, BACKEND)?.remove(reference);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::thread;

    use pretty_assertions::assert_eq;

    use super::*;

    const SENTINEL: &str = "sentinel-memory-secret";

    fn exposed(secret: Option<Secret>) -> Option<String> {
        secret.map(|secret| secret.expose().to_owned())
    }

    #[test]
    fn backend_is_memory() {
        assert_eq!(MemoryVault::default().backend(), "memory");
    }

    #[test]
    fn get_returns_the_secret_that_was_set() {
        let vault = MemoryVault::default();
        let reference = SecretRef::new();

        vault
            .set(&reference, &Secret::new(SENTINEL.to_owned()))
            .unwrap();

        assert_eq!(
            exposed(vault.get(&reference).unwrap()),
            Some(SENTINEL.to_owned())
        );
    }

    #[test]
    fn get_of_a_missing_entry_is_none() {
        let vault = MemoryVault::default();

        assert!(vault.get(&SecretRef::new()).unwrap().is_none());
    }

    #[test]
    fn set_replaces_the_previous_value() {
        let vault = MemoryVault::default();
        let reference = SecretRef::new();
        vault
            .set(&reference, &Secret::new("first".to_owned()))
            .unwrap();

        vault
            .set(&reference, &Secret::new("second".to_owned()))
            .unwrap();

        assert_eq!(
            exposed(vault.get(&reference).unwrap()),
            Some("second".to_owned())
        );
    }

    #[test]
    fn delete_removes_the_entry() {
        let vault = MemoryVault::default();
        let reference = SecretRef::new();
        vault
            .set(&reference, &Secret::new(SENTINEL.to_owned()))
            .unwrap();

        vault.delete(&reference).unwrap();

        assert!(!vault.contains(&reference).unwrap());
    }

    #[test]
    fn delete_of_a_missing_entry_succeeds() {
        let vault = MemoryVault::default();

        assert!(vault.delete(&SecretRef::new()).is_ok());
    }

    #[test]
    fn debug_does_not_show_the_secrets() {
        let vault = MemoryVault::default();
        vault
            .set(&SecretRef::new(), &Secret::new(SENTINEL.to_owned()))
            .unwrap();

        assert!(!format!("{vault:?}").contains(SENTINEL));
    }

    #[test]
    fn poisoned_lock_is_reported_as_a_backend_error() {
        let vault = Arc::new(MemoryVault::default());
        let poisoner = Arc::clone(&vault);
        let _ = thread::spawn(move || {
            let _guard = poisoner.entries.lock().unwrap();
            panic!("poison the lock");
        })
        .join();

        let result = vault.get(&SecretRef::new());

        assert!(matches!(
            result,
            Err(VaultError::Backend {
                backend: "memory",
                ..
            })
        ));
    }
}
