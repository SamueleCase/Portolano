//! Key derivation (Argon2id) and authenticated encryption
//! (XChaCha20-Poly1305) of the vault plaintext.

use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::{AeadInOut, Key, KeyInit, XChaCha20Poly1305, XNonce};
use zeroize::Zeroizing;

use super::KdfParams;
use crate::error::VaultError;
use crate::secret::Secret;

/// Length of the XChaCha20 nonce.
pub(super) const NONCE_LEN: usize = 24;
/// Length of the Poly1305 tag appended to the ciphertext.
pub(super) const TAG_LEN: usize = 16;
const SALT_LEN: usize = 16;
const KEY_LEN: usize = 32;

/// The key derived from the master password, wiped on drop.
pub(super) type DerivedKey = Zeroizing<[u8; KEY_LEN]>;

/// The Argon2id instance for `params`.
///
/// # Errors
/// The parameters are outside the ranges accepted by Argon2.
pub(super) fn argon2(params: KdfParams) -> Result<Argon2<'static>, argon2::Error> {
    let params = Params::new(
        params.m_cost_kib,
        params.t_cost,
        params.p_cost,
        Some(KEY_LEN),
    )?;
    Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
}

/// The key derivation settings of one vault file, fixed when it is created.
pub(super) struct KdfSetup {
    pub(super) params: KdfParams,
    pub(super) salt: Vec<u8>,
}

impl KdfSetup {
    /// New settings with `params` and a random salt.
    pub(super) fn generate(params: KdfParams) -> Result<Self, VaultError> {
        let mut salt = vec![0; SALT_LEN];
        fill_random(&mut salt)?;
        Ok(Self { params, salt })
    }

    /// Derives the encryption key from the master password.
    pub(super) fn derive_key(&self, password: &Secret) -> Result<DerivedKey, VaultError> {
        let crypto_error = |error: argon2::Error| VaultError::Crypto {
            reason: format!("derivazione della chiave non riuscita: {error}"),
        };
        let mut key = Zeroizing::new([0; KEY_LEN]);
        argon2(self.params)
            .map_err(crypto_error)?
            .hash_password_into(password.expose().as_bytes(), &self.salt, &mut key[..])
            .map_err(crypto_error)?;
        Ok(key)
    }
}

/// Encrypts `plaintext` in place under a fresh random nonce and returns the
/// nonce and the ciphertext.
///
/// `plaintext` must have [`TAG_LEN`] bytes of spare capacity: appending the
/// tag then does not reallocate, which would leave a copy of the plaintext
/// in freed memory.
pub(super) fn seal(
    key: &DerivedKey,
    mut plaintext: Zeroizing<Vec<u8>>,
) -> Result<([u8; NONCE_LEN], Zeroizing<Vec<u8>>), VaultError> {
    let mut nonce = [0; NONCE_LEN];
    fill_random(&mut nonce)?;
    cipher(key)
        .encrypt_in_place(&XNonce::from(nonce), b"", &mut *plaintext)
        .map_err(|_| VaultError::Crypto {
            reason: "cifratura non riuscita".to_owned(),
        })?;
    Ok((nonce, plaintext))
}

/// Decrypts `ciphertext` in place.
///
/// # Errors
/// [`VaultError::BadMasterPassword`] when authentication fails: a wrong
/// master password and a tampered file cannot be told apart.
pub(super) fn open(
    key: &DerivedKey,
    nonce: &[u8; NONCE_LEN],
    ciphertext: Vec<u8>,
) -> Result<Zeroizing<Vec<u8>>, VaultError> {
    let mut buffer = Zeroizing::new(ciphertext);
    cipher(key)
        .decrypt_in_place(&XNonce::from(*nonce), b"", &mut *buffer)
        .map_err(|_| VaultError::BadMasterPassword)?;
    Ok(buffer)
}

fn cipher(key: &DerivedKey) -> XChaCha20Poly1305 {
    XChaCha20Poly1305::new(<&Key>::from(&**key))
}

fn fill_random(buffer: &mut [u8]) -> Result<(), VaultError> {
    getrandom::fill(buffer).map_err(|error| VaultError::Crypto {
        reason: format!("generatore casuale del sistema non disponibile: {error}"),
    })
}
