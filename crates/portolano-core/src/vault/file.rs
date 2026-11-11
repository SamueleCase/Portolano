//! Encrypted file vault: Argon2id key derivation and XChaCha20-Poly1305
//! (SPEC §5.2). The file is rewritten atomically on every change.

// Stub (WP0): the parameters are used once WP3 implements the bodies.
#![expect(unused_variables)]

use std::path::PathBuf;

use super::{MasterPasswordSource, Vault};
use crate::error::VaultError;
use crate::secret::{Secret, SecretRef};

/// Argon2id cost parameters, stored in the vault file when it is created.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KdfParams {
    /// Memory cost in KiB.
    pub m_cost_kib: u32,
    /// Number of iterations.
    pub t_cost: u32,
    /// Degree of parallelism.
    pub p_cost: u32,
}

impl KdfParams {
    /// The parameters of SPEC §5.2, used for every new vault outside tests.
    pub const RECOMMENDED: KdfParams = KdfParams {
        m_cost_kib: 65536,
        t_cost: 3,
        p_cost: 1,
    };
}

/// A vault stored in one encrypted file.
///
/// It deliberately does not implement `Debug`: it may hold key material.
// Stub (WP0): the field is read once WP3 implements the methods.
#[expect(dead_code)]
pub struct FileVault {
    path: PathBuf,
}

impl FileVault {
    /// Opens the vault file at `path`, or creates it with
    /// [`KdfParams::RECOMMENDED`] if it does not exist.
    ///
    /// # Errors
    /// [`VaultError::BadMasterPassword`], [`VaultError::Corrupt`],
    /// [`VaultError::UnsupportedVersion`], [`VaultError::Io`], or the error of
    /// the master password source.
    pub fn open(
        path: impl Into<PathBuf>,
        master_password: &dyn MasterPasswordSource,
    ) -> Result<FileVault, VaultError> {
        todo!()
    }

    /// Like [`FileVault::open`], but creates a missing vault with `params`
    /// (reduced costs for tests). An existing vault keeps the parameters
    /// stored in its file.
    ///
    /// # Errors
    /// As [`FileVault::open`].
    pub fn with_kdf_params(
        path: impl Into<PathBuf>,
        master_password: &dyn MasterPasswordSource,
        params: KdfParams,
    ) -> Result<FileVault, VaultError> {
        todo!()
    }
}

impl Vault for FileVault {
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
