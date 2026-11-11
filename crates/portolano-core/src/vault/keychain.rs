//! System keychain vault through `keyring`: macOS Keychain, Windows
//! Credential Manager, Linux Secret Service.
//!
//! Unsigned development builds on macOS may trigger the "allow access to the
//! keychain" prompt after every rebuild; this is expected.
//!
//! Errors of `keyring` become [`VaultError::Backend`] through their `Display`,
//! never their `Debug`. `keyring::Error::BadEncoding` carries the bytes of the
//! secret, so it is matched explicitly and mapped to a fixed message.

use keyring::Entry;
use zeroize::Zeroize;

use super::Vault;
use crate::error::VaultError;
use crate::secret::{Secret, SecretRef};

/// Keychain service under which every secret is stored; the account is the
/// [`SecretRef`].
pub const KEYCHAIN_SERVICE: &str = "dev.portolano";

const BACKEND: &str = "keychain";

/// The system keychain. It holds no state: every operation addresses the
/// entry (`KEYCHAIN_SERVICE`, reference).
#[derive(Debug, Clone, Copy, Default)]
pub struct KeychainVault;

impl KeychainVault {
    fn entry(reference: &SecretRef) -> Result<Entry, VaultError> {
        Entry::new(KEYCHAIN_SERVICE, reference.as_str()).map_err(backend_error)
    }
}

impl Vault for KeychainVault {
    fn backend(&self) -> &'static str {
        BACKEND
    }

    fn get(&self, reference: &SecretRef) -> Result<Option<Secret>, VaultError> {
        match Self::entry(reference)?.get_password() {
            // The password is moved, not copied, into the zeroizing buffer.
            Ok(password) => Ok(Some(Secret::new(password))),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(backend_error(error)),
        }
    }

    fn set(&self, reference: &SecretRef, secret: &Secret) -> Result<(), VaultError> {
        Self::entry(reference)?
            .set_password(secret.expose())
            .map_err(backend_error)
    }

    fn delete(&self, reference: &SecretRef) -> Result<(), VaultError> {
        match Self::entry(reference)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(backend_error(error)),
        }
    }
}

/// Describes a `keyring` error without any secret material.
///
/// The variants that carry the stored bytes are wiped and replaced by a fixed
/// message. `BadDataFormat` and `Ambiguous` are also replaced because their
/// `Display` embeds the `Debug` of inner values.
fn backend_error(error: keyring::Error) -> VaultError {
    let reason = match error {
        keyring::Error::BadEncoding(mut bytes) => {
            bytes.zeroize();
            "il valore salvato nel portachiavi non è testo UTF-8 valido".to_owned()
        }
        keyring::Error::BadDataFormat(mut bytes, _) => {
            bytes.zeroize();
            "il valore salvato nel portachiavi ha un formato non valido".to_owned()
        }
        keyring::Error::Ambiguous(_) => {
            "più voci del portachiavi corrispondono allo stesso riferimento".to_owned()
        }
        other => other.to_string(),
    };
    VaultError::Backend {
        backend: BACKEND,
        reason,
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const SENTINEL: &str = "sentinel-keychain-secret";

    #[test]
    fn backend_is_keychain() {
        assert_eq!(KeychainVault.backend(), "keychain");
    }

    #[test]
    fn bad_encoding_error_does_not_carry_the_stored_bytes() {
        let error = keyring::Error::BadEncoding(SENTINEL.as_bytes().to_vec());

        let mapped = backend_error(error);

        assert!(!format!("{mapped}").contains(SENTINEL));
        assert!(!format!("{mapped:?}").contains(SENTINEL));
    }

    #[test]
    fn bad_data_format_error_does_not_carry_the_stored_bytes() {
        let error = keyring::Error::BadDataFormat(
            SENTINEL.as_bytes().to_vec(),
            Box::new(std::io::Error::other("bad blob")),
        );

        let mapped = backend_error(error);

        assert!(!format!("{mapped:?}").contains(SENTINEL));
    }

    #[test]
    fn other_errors_keep_their_display() {
        let mapped = backend_error(keyring::Error::NoDefaultStore);

        assert!(matches!(
            mapped,
            VaultError::Backend { backend: "keychain", ref reason }
                if *reason == keyring::Error::NoDefaultStore.to_string()
        ));
    }

    /// Touches the real system keychain, which may show an access prompt, so
    /// it runs only on request.
    #[test]
    #[ignore = "uses the system keychain: set PORTOLANO_TEST_KEYCHAIN=1 and pass --ignored"]
    fn keychain_creates_reads_and_deletes_an_entry() {
        if std::env::var("PORTOLANO_TEST_KEYCHAIN").as_deref() != Ok("1") {
            return;
        }
        let vault = KeychainVault;
        // A fresh v4 UUID: the test account cannot clash with a real entry.
        let reference = SecretRef::new();

        vault
            .set(&reference, &Secret::new(SENTINEL.to_owned()))
            .unwrap();
        let read = vault.get(&reference).unwrap();
        vault.delete(&reference).unwrap();

        assert_eq!(read.as_ref().map(Secret::expose), Some(SENTINEL));
        assert!(vault.get(&reference).unwrap().is_none());
        assert!(vault.delete(&reference).is_ok());
    }
}
