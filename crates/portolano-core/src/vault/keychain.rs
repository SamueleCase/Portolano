//! System keychain vault through `keyring`: macOS Keychain, Windows
//! Credential Manager, Linux Secret Service.
//!
//! Unsigned development builds on macOS may trigger the "allow access to the
//! keychain" prompt after every rebuild; this is expected.
//!
//! On macOS reading the value of an item goes through its access control
//! list, which may ask the user; [`Vault::contains`] therefore looks the item
//! up by its attributes only, so that showing the state of the secrets asks
//! nothing (DEC-049). Elsewhere it reads the value, as the trait does.
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

    /// The search of the macOS store asks for the attributes of the matching
    /// generic passwords (`kSecReturnAttributes`) and never for their data,
    /// in the same keychain and class that `get` reads: an absent entry is an
    /// empty result, a failure of the keychain an error.
    #[cfg(target_os = "macos")]
    fn contains(&self, reference: &SecretRef) -> Result<bool, VaultError> {
        // keyring installs its native store as the default one of keyring-core
        // on first use, which may not have happened yet: the search needs it.
        if keyring::Entry::store_status().is_err() {
            return Err(backend_error(keyring::Error::NoDefaultStore));
        }
        any_entry_found(keyring_core::Entry::search(&search_spec(reference)))
    }
}

/// The search for the entry (`KEYCHAIN_SERVICE`, reference).
#[cfg(target_os = "macos")]
fn search_spec(reference: &SecretRef) -> std::collections::HashMap<&'static str, &str> {
    std::collections::HashMap::from([("service", KEYCHAIN_SERVICE), ("user", reference.as_str())])
}

/// Whether a search found an entry; its failures are backend errors.
#[cfg(target_os = "macos")]
fn any_entry_found(search: keyring::Result<Vec<keyring_core::Entry>>) -> Result<bool, VaultError> {
    Ok(!search.map_err(backend_error)?.is_empty())
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

    #[cfg(target_os = "macos")]
    #[test]
    fn search_spec_names_the_service_and_the_reference_as_user() {
        let reference = SecretRef::new();

        let spec = search_spec(&reference);

        assert_eq!(
            spec,
            std::collections::HashMap::from([
                ("service", KEYCHAIN_SERVICE),
                ("user", reference.as_str()),
            ])
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn search_without_matches_means_the_entry_is_absent() {
        assert!(!any_entry_found(Ok(Vec::new())).unwrap());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn search_with_a_match_means_the_entry_is_present() {
        use keyring_core::api::CredentialStoreApi;

        // The mock store of keyring-core lives in memory: no keychain involved.
        let entry = keyring_core::mock::Store::new()
            .unwrap()
            .build(KEYCHAIN_SERVICE, SecretRef::new().as_str(), None)
            .unwrap();

        assert!(any_entry_found(Ok(vec![entry])).unwrap());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn search_failure_is_a_backend_error_not_an_absence() {
        let failure = keyring::Error::NoStorageAccess(Box::new(std::io::Error::other("locked")));

        let result = any_entry_found(Err(failure));

        assert!(matches!(
            result,
            Err(VaultError::Backend {
                backend: "keychain",
                ..
            })
        ));
    }

    /// Deletes the test entry when dropped, so that a failed assertion does
    /// not leave it in the user's keychain.
    struct EntryCleanup<'a>(&'a SecretRef);

    impl Drop for EntryCleanup<'_> {
        fn drop(&mut self) {
            if let Err(error) = KeychainVault.delete(self.0) {
                eprintln!(
                    "could not delete the test entry {} from the keychain: {error}",
                    self.0.as_str()
                );
            }
        }
    }

    /// Touches the real system keychain, which may show an access prompt, so
    /// it runs only on request.
    #[test]
    #[ignore = "uses the system keychain: set PORTOLANO_TEST_KEYCHAIN=1 and pass --ignored"]
    fn keychain_creates_reads_and_deletes_an_entry() {
        if std::env::var("PORTOLANO_TEST_KEYCHAIN").as_deref() != Ok("1") {
            eprintln!("skipped: set PORTOLANO_TEST_KEYCHAIN=1 to use the system keychain");
            return;
        }
        let vault = KeychainVault;
        // A fresh v4 UUID: the test account cannot clash with a real entry.
        let reference = SecretRef::new();
        let _cleanup = EntryCleanup(&reference);

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
