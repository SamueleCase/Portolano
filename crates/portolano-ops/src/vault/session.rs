//! Secrets typed for one connection, kept in memory over the real vault
//! (DEC-016.3).

use portolano_connectors::SessionCredential;
use portolano_core::vault::MemoryVault;
use portolano_core::{Secret, SecretRef, Vault, VaultError};

/// Login data typed by the user for a connection whose credential is "ask at
/// launch" (`Prompt`).
#[derive(Debug, Default)]
pub struct Login {
    /// User name.
    pub username: Option<String>,
    /// Windows domain.
    pub domain: Option<String>,
    /// Password.
    pub password: Option<Secret>,
}

/// A vault that answers first from an in-memory layer, then from the real
/// vault. Writes and deletions go to the memory layer only: the secrets of a
/// session never reach the real vault and vanish with this value.
pub struct SessionVault<'a> {
    base: &'a dyn Vault,
    session: MemoryVault,
}

impl<'a> SessionVault<'a> {
    /// An empty session over `base`.
    pub fn new(base: &'a dyn Vault) -> Self {
        Self {
            base,
            session: MemoryVault::default(),
        }
    }

    /// Stores the password of `login` in the session and returns the
    /// credential to put in `ConnectOptions::session_credential`.
    ///
    /// # Errors
    /// Errors of the in-memory vault.
    pub fn session_credential(&self, login: Login) -> Result<SessionCredential, VaultError> {
        let password = match login.password {
            Some(secret) => {
                let reference = SecretRef::new();
                self.session.set(&reference, &secret)?;
                Some(reference)
            }
            None => None,
        };
        Ok(SessionCredential {
            username: login.username,
            domain: login.domain,
            password,
        })
    }
}

impl Vault for SessionVault<'_> {
    fn backend(&self) -> &'static str {
        self.base.backend()
    }

    fn get(&self, reference: &SecretRef) -> Result<Option<Secret>, VaultError> {
        match self.session.get(reference)? {
            Some(secret) => Ok(Some(secret)),
            None => self.base.get(reference),
        }
    }

    fn set(&self, reference: &SecretRef, secret: &Secret) -> Result<(), VaultError> {
        self.session.set(reference, secret)
    }

    fn delete(&self, reference: &SecretRef) -> Result<(), VaultError> {
        self.session.delete(reference)
    }

    fn contains(&self, reference: &SecretRef) -> Result<bool, VaultError> {
        Ok(self.session.contains(reference)? || self.base.contains(reference)?)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn secret(value: &str) -> Secret {
        Secret::new(value.to_owned())
    }

    #[test]
    fn session_credential_keeps_the_password_in_the_session_only() {
        let base = MemoryVault::default();
        let session = SessionVault::new(&base);
        let credential = session
            .session_credential(Login {
                username: Some("admin".to_owned()),
                domain: None,
                password: Some(secret("sentinel-session")),
            })
            .unwrap();
        let reference = credential.password.unwrap();
        assert_eq!(
            session.get(&reference).unwrap().unwrap().expose(),
            "sentinel-session"
        );
        assert!(!base.contains(&reference).unwrap());
        assert_eq!(credential.username.as_deref(), Some("admin"));
    }

    #[test]
    fn reads_fall_back_to_the_real_vault() {
        let base = MemoryVault::default();
        let reference = SecretRef::new();
        base.set(&reference, &secret("sentinel-base")).unwrap();
        let session = SessionVault::new(&base);
        assert_eq!(
            session.get(&reference).unwrap().unwrap().expose(),
            "sentinel-base"
        );
    }

    #[test]
    fn writes_and_deletions_never_reach_the_real_vault() {
        let base = MemoryVault::default();
        let reference = SecretRef::new();
        base.set(&reference, &secret("sentinel-base")).unwrap();
        let session = SessionVault::new(&base);
        session
            .set(&reference, &secret("sentinel-session"))
            .unwrap();
        session.delete(&reference).unwrap();
        assert_eq!(
            base.get(&reference).unwrap().unwrap().expose(),
            "sentinel-base"
        );
    }

    #[test]
    fn login_without_password_gives_no_reference() {
        let base = MemoryVault::default();
        let session = SessionVault::new(&base);
        let credential = session.session_credential(Login::default()).unwrap();
        assert_eq!(credential.password, None);
    }
}
