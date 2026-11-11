//! Key derivation (Argon2id) and authenticated encryption
//! (XChaCha20-Poly1305) of the vault plaintext.

use argon2::{Algorithm, Argon2, Block, Params, Version};
use chacha20poly1305::{AeadInOut, Key, KeyInit, XChaCha20Poly1305, XNonce};
use zeroize::{Zeroize, Zeroizing};

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
///
/// It lives on the heap: moving it (into the `FileVault`, out of a function)
/// moves only the pointer, instead of leaving copies of the key on the stack.
pub(super) struct DerivedKey(Box<[u8; KEY_LEN]>);

impl DerivedKey {
    fn zeroed() -> Self {
        Self(Box::new([0; KEY_LEN]))
    }
}

impl Drop for DerivedKey {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

// Upper bounds for the KDF costs (DEC-017). The parameters stored in a file
// are not authenticated, so a tampered file could otherwise make the open
// allocate gigabytes or run for hours before failing; the same bounds apply
// when a vault is created, so that every vault can be opened again. Each
// bound leaves ample room above `KdfParams::RECOMMENDED` (64 MiB, 3 passes,
// 1 lane):
// - 1 GiB of memory is 16 times the recommendation and still fits on any
//   machine that runs the app;
// - 16 passes is over 5 times the recommendation;
// - 16 lanes: lanes split the same memory, so they do not add work, but no
//   real setup uses more and a larger value can only come from tampering.
// Even with every cost at its bound the open stays in the order of tens of
// seconds, never hours, and the allocation stays bounded.
const MAX_M_COST_KIB: u32 = 1024 * 1024;
const MAX_T_COST: u32 = 16;
const MAX_P_COST: u32 = 16;

/// Checks that `params` are within the bounds above and accepted by Argon2.
///
/// # Errors
/// A description of the first invalid parameter.
pub(super) fn check_params(params: KdfParams) -> Result<(), String> {
    let limits = [
        ("m_cost_kib", params.m_cost_kib, MAX_M_COST_KIB),
        ("t_cost", params.t_cost, MAX_T_COST),
        ("p_cost", params.p_cost, MAX_P_COST),
    ];
    if let Some((name, value, max)) = limits.into_iter().find(|&(_, value, max)| value > max) {
        return Err(format!(
            "parametro KDF '{name}' = {value} oltre il massimo ammesso ({max})"
        ));
    }
    argon2(params)
        .map(drop)
        .map_err(|error| format!("parametri KDF non validi: {error}"))
}

fn argon2(params: KdfParams) -> Result<Argon2<'static>, argon2::Error> {
    let params = Params::new(
        params.m_cost_kib,
        params.t_cost,
        params.p_cost,
        Some(KEY_LEN),
    )?;
    Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
}

/// The key derivation settings of one vault file, fixed when it is created.
/// Two files with different settings are different vaults.
#[derive(PartialEq, Eq)]
pub(super) struct KdfSetup {
    pub(super) params: KdfParams,
    pub(super) salt: Vec<u8>,
}

impl KdfSetup {
    /// New settings with `params` and a random salt.
    ///
    /// # Errors
    /// [`VaultError::Crypto`] if `params` fail [`check_params`].
    pub(super) fn generate(params: KdfParams) -> Result<Self, VaultError> {
        check_params(params).map_err(|reason| VaultError::Crypto { reason })?;
        let mut salt = vec![0; SALT_LEN];
        fill_random(&mut salt)?;
        Ok(Self { params, salt })
    }

    /// Derives the encryption key from the master password.
    pub(super) fn derive_key(&self, password: &Secret) -> Result<DerivedKey, VaultError> {
        let crypto_error = |error: argon2::Error| VaultError::Crypto {
            reason: format!("derivazione della chiave non riuscita: {error}"),
        };
        let argon2 = argon2(self.params).map_err(crypto_error)?;
        // The memory blocks hold values from which the key can be computed.
        // `hash_password_into` frees its own blocks without wiping them, so
        // the blocks are allocated here, in a buffer wiped on drop. The
        // reservation is fallible so that a lack of memory is an error, not
        // an abort.
        let block_count = argon2.params().block_count();
        let mut blocks = Zeroizing::new(Vec::new());
        blocks
            .try_reserve_exact(block_count)
            .map_err(|_| crypto_error(argon2::Error::OutOfMemory))?;
        blocks.resize(block_count, Block::new());
        let mut key = DerivedKey::zeroed();
        argon2
            .hash_password_into_with_memory(
                password.expose().as_bytes(),
                &self.salt,
                &mut key.0[..],
                &mut blocks[..],
            )
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
    XChaCha20Poly1305::new(<&Key>::from(&*key.0))
}

fn fill_random(buffer: &mut [u8]) -> Result<(), VaultError> {
    getrandom::fill(buffer).map_err(|error| VaultError::Crypto {
        reason: format!("generatore casuale del sistema non disponibile: {error}"),
    })
}
