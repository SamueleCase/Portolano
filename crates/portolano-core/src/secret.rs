//! Secret values in memory and opaque references to secrets in the vault
//! (SPEC §5.1).

use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zeroize::Zeroizing;

/// Opaque reference to a secret stored in the vault.
///
/// It is a v4 UUID in text form and is the only trace of a secret in the
/// document: it is not itself sensitive, so it can be logged and displayed.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SecretRef(String);

impl SecretRef {
    /// Generates a new, random reference.
    // A `Default` impl would hide that every call yields a different value.
    #[expect(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// The reference in text form, as stored in the document and used as the
    /// vault key.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A secret value (password, passphrase, private key) held in memory.
///
/// The buffer is zeroed on drop. `Debug` prints `Secret(***)`; `Display`,
/// `Serialize` and `Deserialize` are deliberately not implemented so that a
/// secret cannot end up in output, logs or files by accident (SPEC §1).
pub struct Secret(Zeroizing<String>);

impl Secret {
    /// Wraps a secret value; the string is zeroed when the `Secret` is dropped.
    pub fn new(value: String) -> Self {
        Self(Zeroizing::new(value))
    }

    /// The secret value. Callers must not copy it into unprotected storage.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(***)")
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const SENTINEL: &str = "sentinel-not-a-real-password";

    #[test]
    fn secret_debug_is_masked_and_hides_the_value() {
        let secret = Secret::new(SENTINEL.to_owned());

        let debug = format!("{secret:?}");

        assert_eq!(debug, "Secret(***)");
        assert!(!debug.contains(SENTINEL));
    }

    #[test]
    fn secret_debug_inside_a_container_hides_the_value() {
        let secrets = vec![(SecretRef::new(), Secret::new(SENTINEL.to_owned()))];

        let debug = format!("{secrets:?}");

        assert!(debug.contains("Secret(***)"));
        assert!(!debug.contains(SENTINEL));
    }

    #[test]
    fn secret_expose_returns_the_value() {
        let secret = Secret::new(SENTINEL.to_owned());

        assert_eq!(secret.expose(), SENTINEL);
    }

    #[test]
    fn secret_ref_new_generates_distinct_v4_uuids() {
        let first = SecretRef::new();
        let second = SecretRef::new();

        assert_ne!(first, second);
        let parsed = Uuid::parse_str(first.as_str()).unwrap();
        assert_eq!(parsed.get_version_num(), 4);
    }

    #[test]
    fn secret_ref_serializes_as_a_plain_string() {
        let reference = SecretRef::new();

        let json = serde_json::to_value(&reference).unwrap();
        let back: SecretRef = serde_json::from_value(json.clone()).unwrap();

        assert_eq!(
            json,
            serde_json::Value::String(reference.as_str().to_owned())
        );
        assert_eq!(back, reference);
    }
}
