//! Encrypted file vault: Argon2id key derivation and XChaCha20-Poly1305
//! (SPEC §5.2). The file is rewritten atomically on every change.
//!
//! Several processes may use the same vault at once (DEC-017): every
//! operation rereads and decrypts the current file under an inter-process
//! lock, so a change made by another process is never lost. Only the
//! derived key stays in memory between operations, in a zeroizing buffer;
//! the salt is fixed for the life of the file, so the key stays valid. Every
//! write encrypts all the secrets again with a fresh random nonce.

mod atomic;
mod crypto;
mod envelope;
mod lock;
mod plaintext;
#[cfg(test)]
mod tests;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use self::atomic::{WriteMode, write_atomically};
use self::crypto::{DerivedKey, KdfSetup, TAG_LEN};
use self::envelope::Sealed;
use self::lock::{Access, with_lock};
use super::{MasterPasswordSource, SecretMap, Vault, copy_secret};
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
/// It deliberately does not implement `Debug`: it holds key material.
pub struct FileVault {
    path: PathBuf,
    kdf: KdfSetup,
    key: DerivedKey,
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
        // The file is validated before the password is requested, so that a
        // damaged file is reported without a useless prompt. The prompt runs
        // without the lock held, so that it never blocks other processes.
        match with_lock(&path, Access::Shared, || read_sealed(&path))? {
            Some(sealed) => Self::unlock(path, sealed.kdf, master_password),
            None => Self::create(path, master_password, params),
        }
    }

    fn unlock(
        path: PathBuf,
        kdf: KdfSetup,
        master_password: &dyn MasterPasswordSource,
    ) -> Result<Self, VaultError> {
        let key = kdf.derive_key(&master_password.master_password(false)?)?;
        let vault = Self { path, kdf, key };
        // Decrypting the current content is what checks the password.
        with_lock(&vault.path, Access::Shared, || vault.read_secrets())?;
        Ok(vault)
    }

    fn create(
        path: PathBuf,
        master_password: &dyn MasterPasswordSource,
        params: KdfParams,
    ) -> Result<Self, VaultError> {
        let kdf = KdfSetup::generate(params)?;
        let key = kdf.derive_key(&master_password.master_password(true)?)?;
        let vault = Self { path, kdf, key };
        // Written at once, so that the next open finds the vault instead of
        // asking for a new master password again. `CreateNew` never replaces
        // a vault that another process created during the prompt.
        //
        // That case is an error rather than a silent reopen: the password was
        // asked for (and confirmed) to create a new vault, and using it on the
        // other vault would either succeed without the user knowing which
        // vault they set up, or fail as a "wrong password" that they never
        // mistyped. Retrying opens the existing vault with the right prompt.
        with_lock(&vault.path, Access::Exclusive, || {
            vault.write_secrets(&SecretMap::new(), WriteMode::CreateNew)
        })
        .map_err(|error| match error {
            VaultError::Io { source, .. } if source.kind() == io::ErrorKind::AlreadyExists => {
                vault.changed("è stato creato da un altro processo durante la creazione: riprova")
            }
            other => other,
        })?;
        Ok(vault)
    }

    /// The secrets in the file as it is now. The caller holds the lock.
    fn read_secrets(&self) -> Result<SecretMap, VaultError> {
        let sealed = read_sealed(&self.path)?
            .ok_or_else(|| self.changed("è stato rimosso dopo l'apertura: va riaperto"))?;
        if sealed.kdf != self.kdf {
            return Err(
                self.changed("è stato sostituito da un altro vault dopo l'apertura: va riaperto")
            );
        }
        let plaintext = crypto::open(&self.key, &sealed.nonce, sealed.ciphertext)?;
        plaintext::decode(&plaintext).map_err(|reason| VaultError::Corrupt {
            path: self.path.clone(),
            reason: reason.to_owned(),
        })
    }

    /// Rereads the file, applies one change (`Some` stores, `None` removes)
    /// and rewrites it, all under the exclusive lock, so that changes made
    /// by other processes in the meantime are kept.
    fn update(&self, reference: &SecretRef, value: Option<Secret>) -> Result<(), VaultError> {
        with_lock(&self.path, Access::Exclusive, || {
            let mut secrets = self.read_secrets()?;
            let changed = match value {
                Some(secret) => {
                    secrets.insert(reference.clone(), secret);
                    true
                }
                None => secrets.remove(reference).is_some(),
            };
            if changed {
                self.write_secrets(&secrets, WriteMode::Replace)
            } else {
                Ok(())
            }
        })
    }

    /// Encrypts and writes the secrets. The caller holds the exclusive lock.
    fn write_secrets(&self, secrets: &SecretMap, mode: WriteMode) -> Result<(), VaultError> {
        let plaintext = plaintext::encode(secrets, TAG_LEN)?;
        let (nonce, ciphertext) = crypto::seal(&self.key, plaintext)?;
        let contents = envelope::serialize(&self.kdf, &nonce, &ciphertext)?;
        write_atomically(&self.path, &contents, mode).map_err(|source| VaultError::Io {
            path: self.path.clone(),
            source,
        })
    }

    /// Another process removed, replaced or created the file behind this
    /// vault, so the key in memory does not match it. `what` completes the
    /// sentence "il vault '<path>' …".
    fn changed(&self, what: &str) -> VaultError {
        VaultError::Backend {
            backend: BACKEND,
            reason: format!("il vault '{}' {what}", self.path.display()),
        }
    }
}

/// The current vault file, decoded but still encrypted, or `None` if it does
/// not exist.
fn read_sealed(path: &Path) -> Result<Option<Sealed>, VaultError> {
    match fs::read(path) {
        Ok(contents) => envelope::parse(path, &contents).map(Some),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(VaultError::Io {
            path: path.to_owned(),
            source,
        }),
    }
}

impl Vault for FileVault {
    fn backend(&self) -> &'static str {
        BACKEND
    }

    fn get(&self, reference: &SecretRef) -> Result<Option<Secret>, VaultError> {
        with_lock(&self.path, Access::Shared, || {
            Ok(self.read_secrets()?.remove(reference))
        })
    }

    fn set(&self, reference: &SecretRef, secret: &Secret) -> Result<(), VaultError> {
        self.update(reference, Some(copy_secret(secret)))
    }

    fn delete(&self, reference: &SecretRef) -> Result<(), VaultError> {
        self.update(reference, None)
    }
}
