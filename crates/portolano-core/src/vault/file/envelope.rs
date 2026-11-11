//! The JSON layout of the vault file (SPEC §5.2): format, version, KDF
//! settings, cipher, nonce and ciphertext.

use std::path::Path;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde::{Deserialize, Serialize};

use super::KdfParams;
use super::crypto::{self, KdfSetup, NONCE_LEN};
use crate::error::VaultError;

const FORMAT: &str = "portolano-vault";
const VERSION: u32 = 1;
const KDF_ALGORITHM: &str = "argon2id";
const CIPHER: &str = "xchacha20poly1305";

// Upper bounds for the KDF costs read from a file (DEC-017). The parameters
// are not authenticated, so a tampered file could otherwise make the open
// allocate gigabytes or run for hours before failing. Each bound leaves ample
// room above `KdfParams::RECOMMENDED` (64 MiB, 3 passes, 1 lane):
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

#[derive(Serialize, Deserialize)]
struct VaultFile {
    format: String,
    version: u32,
    kdf: KdfSection,
    cipher: String,
    nonce: String,
    ciphertext: String,
}

#[derive(Serialize, Deserialize)]
struct KdfSection {
    alg: String,
    m_cost_kib: u32,
    t_cost: u32,
    p_cost: u32,
    salt: String,
}

/// The fields read first, so that a newer version is reported as such even
/// when the rest of its layout differs.
#[derive(Deserialize)]
struct FormatProbe {
    format: String,
    version: u32,
}

/// The content of a vault file, decoded but still encrypted.
pub(super) struct Sealed {
    pub(super) kdf: KdfSetup,
    pub(super) nonce: [u8; NONCE_LEN],
    pub(super) ciphertext: Vec<u8>,
}

/// Parses and validates the vault file at `path`.
///
/// # Errors
/// [`VaultError::Corrupt`] or [`VaultError::UnsupportedVersion`].
pub(super) fn parse(path: &Path, contents: &[u8]) -> Result<Sealed, VaultError> {
    let corrupt = |reason: String| VaultError::Corrupt {
        path: path.to_owned(),
        reason,
    };
    let probe: FormatProbe = serde_json::from_slice(contents)
        .map_err(|error| corrupt(format!("non è un vault Portolano: {error}")))?;
    if probe.format != FORMAT {
        return Err(corrupt(format!(
            "formato '{}' non riconosciuto",
            probe.format
        )));
    }
    if probe.version != VERSION {
        return Err(VaultError::UnsupportedVersion {
            path: path.to_owned(),
            found: probe.version,
            supported: VERSION,
        });
    }
    let file: VaultFile = serde_json::from_slice(contents)
        .map_err(|error| corrupt(format!("struttura non valida: {error}")))?;
    file.into_sealed().map_err(corrupt)
}

impl VaultFile {
    /// Checks the algorithms and the KDF parameters and decodes the binary
    /// fields. The error is the reason of a [`VaultError::Corrupt`].
    fn into_sealed(self) -> Result<Sealed, String> {
        if self.kdf.alg != KDF_ALGORITHM || self.cipher != CIPHER {
            return Err(format!(
                "algoritmi '{}' / '{}' non supportati",
                self.kdf.alg, self.cipher
            ));
        }
        let params = KdfParams {
            m_cost_kib: self.kdf.m_cost_kib,
            t_cost: self.kdf.t_cost,
            p_cost: self.kdf.p_cost,
        };
        check_kdf_limits(params)?;
        crypto::argon2(params).map_err(|error| format!("parametri KDF non validi: {error}"))?;
        let salt = decode_field("salt", &self.kdf.salt)?;
        if salt.len() < argon2::MIN_SALT_LEN {
            return Err("salt troppo corto".to_owned());
        }
        let nonce = decode_field("nonce", &self.nonce)?
            .try_into()
            .map_err(|_| format!("il nonce deve essere di {NONCE_LEN} byte"))?;
        Ok(Sealed {
            kdf: KdfSetup { params, salt },
            nonce,
            ciphertext: decode_field("ciphertext", &self.ciphertext)?,
        })
    }
}

fn check_kdf_limits(params: KdfParams) -> Result<(), String> {
    let limits = [
        ("m_cost_kib", params.m_cost_kib, MAX_M_COST_KIB),
        ("t_cost", params.t_cost, MAX_T_COST),
        ("p_cost", params.p_cost, MAX_P_COST),
    ];
    match limits.into_iter().find(|&(_, value, max)| value > max) {
        Some((name, value, max)) => Err(format!(
            "parametro KDF '{name}' = {value} oltre il massimo ammesso ({max})"
        )),
        None => Ok(()),
    }
}

fn decode_field(name: &str, value: &str) -> Result<Vec<u8>, String> {
    BASE64
        .decode(value)
        .map_err(|_| format!("campo '{name}' non in base64"))
}

/// The vault file for the given settings and encrypted content.
///
/// # Errors
/// [`VaultError::Crypto`] if serialization fails.
pub(super) fn serialize(
    kdf: &KdfSetup,
    nonce: &[u8; NONCE_LEN],
    ciphertext: &[u8],
) -> Result<Vec<u8>, VaultError> {
    let file = VaultFile {
        format: FORMAT.to_owned(),
        version: VERSION,
        kdf: KdfSection {
            alg: KDF_ALGORITHM.to_owned(),
            m_cost_kib: kdf.params.m_cost_kib,
            t_cost: kdf.params.t_cost,
            p_cost: kdf.params.p_cost,
            salt: BASE64.encode(&kdf.salt),
        },
        cipher: CIPHER.to_owned(),
        nonce: BASE64.encode(nonce),
        ciphertext: BASE64.encode(ciphertext),
    };
    serde_json::to_vec_pretty(&file).map_err(|_| VaultError::Crypto {
        reason: "serializzazione del vault non riuscita".to_owned(),
    })
}
