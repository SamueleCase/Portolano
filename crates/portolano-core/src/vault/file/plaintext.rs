//! The plaintext of the vault: the JSON object
//! `{ "<secretref>": "<base64 secret>" }` (SPEC §5.2).
//!
//! It only ever lives in buffers that are wiped on drop, and it is never
//! copied into ordinary `String`s or `Vec`s.

use std::collections::HashMap;
use std::io::{self, Write};

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use zeroize::{Zeroize, Zeroizing};

use crate::error::VaultError;
use crate::secret::{Secret, SecretRef};
use crate::vault::SecretMap;

/// Serializes the secrets, leaving `spare_capacity` free bytes at the end of
/// the buffer.
pub(super) fn encode(
    secrets: &SecretMap,
    spare_capacity: usize,
) -> Result<Zeroizing<Vec<u8>>, VaultError> {
    let mut buffer = WipingBuffer::default();
    serde_json::to_writer(&mut buffer, &EncodedSecrets(secrets)).map_err(|_| {
        VaultError::Crypto {
            reason: "serializzazione dei segreti non riuscita".to_owned(),
        }
    })?;
    buffer.reserve(spare_capacity);
    Ok(buffer.0)
}

/// Parses the decrypted secrets.
///
/// # Errors
/// A fixed description: the messages of `serde_json` may quote fragments of
/// the plaintext.
pub(super) fn decode(plaintext: &[u8]) -> Result<SecretMap, &'static str> {
    // The values are borrowed from the plaintext buffer instead of being
    // copied into strings that would not be wiped.
    let encoded: HashMap<SecretRef, &str> =
        serde_json::from_slice(plaintext).map_err(|_| "contenuto decifrato non valido")?;
    encoded
        .into_iter()
        .map(|(reference, value)| Ok((reference, decode_secret(value)?)))
        .collect()
}

fn decode_secret(encoded: &str) -> Result<Secret, &'static str> {
    let mut bytes = Zeroizing::new(vec![0; base64::decoded_len_estimate(encoded.len())]);
    let len = BASE64
        .decode_slice(encoded, &mut bytes[..])
        .map_err(|_| "segreto non in base64")?;
    bytes.truncate(len);
    // The allocation moves into the `Secret` without being copied.
    let text = String::from_utf8(std::mem::take(&mut *bytes)).map_err(|error| {
        error.into_bytes().zeroize();
        "segreto non in UTF-8"
    })?;
    Ok(Secret::new(text))
}

/// Serializes a [`SecretMap`] with each value encoded in base64 into a
/// zeroizing string.
struct EncodedSecrets<'a>(&'a SecretMap);

impl Serialize for EncodedSecrets<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (reference, secret) in self.0 {
            let encoded = Zeroizing::new(BASE64.encode(secret.expose()));
            map.serialize_entry(reference.as_str(), encoded.as_str())?;
        }
        map.end()
    }
}

/// A growable byte buffer for plaintext. A `Vec` that grows frees its old
/// allocation without wiping it; this buffer moves to a larger zeroizing
/// allocation itself, so the old one is wiped.
#[derive(Default)]
struct WipingBuffer(Zeroizing<Vec<u8>>);

impl WipingBuffer {
    fn reserve(&mut self, additional: usize) {
        let needed = self.0.len().saturating_add(additional);
        if needed > self.0.capacity() {
            let capacity = needed.max(self.0.capacity().saturating_mul(2));
            let mut larger = Zeroizing::new(Vec::with_capacity(capacity));
            larger.extend_from_slice(&self.0);
            self.0 = larger;
        }
    }
}

impl Write for WipingBuffer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.reserve(bytes.len());
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
