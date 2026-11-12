//! Secrets typed in the interface (SPEC-UI §1.2).

use portolano_core::Secret;
use serde::{Deserialize, Deserializer};

/// A password, passphrase, key or master password typed in the interface.
///
/// It reaches the backend in the one command that uses it and becomes a
/// [`Secret`] while the request is read, so that no other value of the app
/// holds it as plain text; it cannot be serialized back. `Debug` shows
/// `Secret(***)`.
///
/// Known limit (SPEC-UI §1.2, README): the text of the request is not wiped
/// in the webview or in the IPC layer of Tauri.
#[derive(Debug, specta::Type)]
#[specta(type = String)]
pub struct SecretText(Secret);

impl SecretText {
    /// The secret.
    pub fn into_secret(self) -> Secret {
        self.0
    }
}

impl<'de> Deserialize<'de> for SecretText {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer).map(|text| SecretText(Secret::new(text)))
    }
}

#[cfg(test)]
impl SecretText {
    /// A secret typed in a test.
    pub fn typed(text: &str) -> Self {
        SecretText(Secret::new(text.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_text_becomes_a_secret_that_debug_does_not_show() {
        let typed: SecretText = serde_json::from_str("\"sentinel-app-secret\"").unwrap();

        assert!(!format!("{typed:?}").contains("sentinel-app-secret"));
        assert_eq!(typed.into_secret().expose(), "sentinel-app-secret");
    }
}
