//! Encrypted file vault: Argon2id key derivation and XChaCha20-Poly1305
//! (SPEC §5.2). The file is rewritten atomically on every change.
//!
//! The key is derived once, when the vault is opened, and kept in a zeroizing
//! buffer; the decrypted secrets are kept as [`Secret`]s. Every change
//! serializes all the secrets again and encrypts them under the same key with
//! a fresh random nonce (the salt is fixed for the life of the file).

mod atomic;
mod crypto;
mod envelope;
mod plaintext;
#[cfg(test)]
mod tests;

use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::Mutex;

use self::atomic::{WriteMode, write_atomically};
use self::crypto::{DerivedKey, KdfSetup, TAG_LEN};
use super::{MasterPasswordSource, SecretMap, Vault, copy_secret, lock_secrets};
use crate::error::VaultError;
use crate::secret::{Secret, SecretRef};

const BACKEND: &str = "file";

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
pub struct FileVault {
    path: PathBuf,
    kdf: KdfSetup,
    key: DerivedKey,
    entries: Mutex<SecretMap>,
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
        Self::with_kdf_params(path, master_password, KdfParams::RECOMMENDED)
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
        let path = path.into();
        match fs::read(&path) {
            Ok(contents) => Self::load(path, &contents, master_password),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                Self::create(path, master_password, params)
            }
            Err(source) => Err(VaultError::Io { path, source }),
        }
    }

    fn load(
        path: PathBuf,
        contents: &[u8],
        master_password: &dyn MasterPasswordSource,
    ) -> Result<Self, VaultError> {
        // The file is validated before the password is requested, so that a
        // damaged file is reported without a useless prompt.
        let sealed = envelope::parse(&path, contents)?;
        let key = sealed
            .kdf
            .derive_key(&master_password.master_password(false)?)?;
        let plaintext = crypto::open(&key, &sealed.nonce, sealed.ciphertext)?;
        let entries = plaintext::decode(&plaintext).map_err(|reason| VaultError::Corrupt {
            path: path.clone(),
            reason: reason.to_owned(),
        })?;
        Ok(Self {
            path,
            kdf: sealed.kdf,
            key,
            entries: Mutex::new(entries),
        })
    }

    fn create(
        path: PathBuf,
        master_password: &dyn MasterPasswordSource,
        params: KdfParams,
    ) -> Result<Self, VaultError> {
        let kdf = KdfSetup::generate(params)?;
        let key = kdf.derive_key(&master_password.master_password(true)?)?;
        let vault = Self {
            path,
            kdf,
            key,
            entries: Mutex::new(SecretMap::new()),
        };
        // Written at once, so that the next open finds the vault instead of
        // asking for a new master password again.
        vault.persist(&SecretMap::new(), WriteMode::CreateNew)?;
        Ok(vault)
    }

    /// Applies one change (`Some` stores, `None` removes) and rewrites the
    /// file. If the file cannot be written the change is undone, so that the
    /// secrets in memory never differ from those on disk.
    fn update(&self, reference: &SecretRef, value: Option<Secret>) -> Result<(), VaultError> {
        let mut entries = lock_secrets(&self.entries, BACKEND)?;
        let removing = value.is_none();
        let previous = match value {
            Some(secret) => entries.insert(reference.clone(), secret),
            None => entries.remove(reference),
        };
        if removing && previous.is_none() {
            return Ok(());
        }
        if let Err(error) = self.persist(&entries, WriteMode::Replace) {
            match previous {
                Some(secret) => entries.insert(reference.clone(), secret),
                None => entries.remove(reference),
            };
            return Err(error);
        }
        Ok(())
    }

    fn persist(&self, entries: &SecretMap, mode: WriteMode) -> Result<(), VaultError> {
        let plaintext = plaintext::encode(entries, TAG_LEN)?;
        let (nonce, ciphertext) = crypto::seal(&self.key, plaintext)?;
        let contents = envelope::serialize(&self.kdf, &nonce, &ciphertext)?;
        write_atomically(&self.path, &contents, mode).map_err(|source| VaultError::Io {
            path: self.path.clone(),
            source,
        })
    }
}

impl Vault for FileVault {
    fn backend(&self) -> &'static str {
        BACKEND
    }

    fn get(&self, reference: &SecretRef) -> Result<Option<Secret>, VaultError> {
        Ok(lock_secrets(&self.entries, BACKEND)?
            .get(reference)
            .map(copy_secret))
    }

    fn set(&self, reference: &SecretRef, secret: &Secret) -> Result<(), VaultError> {
        self.update(reference, Some(copy_secret(secret)))
    }

    fn delete(&self, reference: &SecretRef) -> Result<(), VaultError> {
        self.update(reference, None)
    }
}
