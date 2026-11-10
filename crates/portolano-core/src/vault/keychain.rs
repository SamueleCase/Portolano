//! System keychain vault through `keyring`: macOS Keychain, Windows
//! Credential Manager, Linux Secret Service.
//!
//! Unsigned development builds on macOS may trigger the "allow access to the
//! keychain" prompt after every rebuild; this is expected.

// Stub (WP0): the parameters are used once WP3 implements the bodies.
#![expect(unused_variables)]

use super::Vault;
use crate::error::VaultError;
use crate::secret::{Secret, SecretRef};

/// Keychain service under which every secret is stored; the account is the
/// [`SecretRef`].
pub const KEYCHAIN_SERVICE: &str = "dev.portolano";

/// The system keychain. It holds no state: every operation addresses the
/// entry (`KEYCHAIN_SERVICE`, reference).
#[derive(Debug, Clone, Copy, Default)]
pub struct KeychainVault;

impl Vault for KeychainVault {
    fn backend(&self) -> &'static str {
        todo!()
    }

    fn get(&self, reference: &SecretRef) -> Result<Option<Secret>, VaultError> {
        todo!()
    }

    fn set(&self, reference: &SecretRef, secret: &Secret) -> Result<(), VaultError> {
        todo!()
    }

    fn delete(&self, reference: &SecretRef) -> Result<(), VaultError> {
        todo!()
    }
}
