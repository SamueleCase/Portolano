//! State shared by the mapping of one object, and the collection of the
//! secrets typed by the user.

use std::collections::BTreeMap;

use portolano_core::{Secret, SecretRef};
use serde_json::Value;

use super::EXTRA_PREFIX;
use crate::royal::fields::Fields;
use crate::{ImportError, SecretField, SecretPrompter, SecretRequest};

/// The object being mapped.
pub(super) struct ObjectContext<'a> {
    pub(super) fields: Fields<'a>,
    /// Path of the node in the imported tree.
    pub(super) path: String,
    /// User name shown with the secret requests of this node.
    pub(super) username: Option<String>,
    /// Why the node is only partially imported.
    pub(super) reasons: Vec<String>,
    pub(super) extra: BTreeMap<String, Value>,
}

impl<'a> ObjectContext<'a> {
    pub(super) fn new(fields: Fields<'a>, path: String) -> Self {
        Self {
            fields,
            path,
            username: None,
            reasons: Vec::new(),
            extra: BTreeMap::new(),
        }
    }

    /// Keeps the original value of a field that was mapped, but whose
    /// mapping is lossy or uncertain.
    pub(super) fn keep_original(&mut self, field: &str, value: &str) {
        self.extra.insert(
            format!("{EXTRA_PREFIX}{field}"),
            Value::String(value.to_owned()),
        );
    }

    pub(super) fn add_reason(&mut self, reason: impl Into<String>) {
        self.reasons.push(reason.into());
    }
}

/// Asks the prompter for the secrets of encrypted fields and keeps the
/// answers for the vault.
pub(super) struct SecretCollector<'p> {
    prompter: &'p mut dyn SecretPrompter,
    collected: Vec<(SecretRef, Secret)>,
}

impl<'p> SecretCollector<'p> {
    pub(super) fn new(prompter: &'p mut dyn SecretPrompter) -> Self {
        Self {
            prompter,
            collected: Vec::new(),
        }
    }

    /// If the encrypted `field` is present and not blank, asks the user for
    /// its value. The ciphertext is only checked for presence: it cannot be
    /// decrypted and must not be kept (D3). A skipped secret makes the node
    /// partial.
    ///
    /// # Errors
    /// The error of the prompter.
    pub(super) fn request(
        &mut self,
        ctx: &mut ObjectContext<'_>,
        field: &str,
        kind: SecretField,
    ) -> Result<Option<SecretRef>, ImportError> {
        if ctx.fields.take(field).is_none() {
            return Ok(None);
        }
        let request = SecretRequest {
            node_path: ctx.path.clone(),
            username: ctx.username.clone(),
            field: kind,
        };
        match self.prompter.ask(&request)? {
            Some(secret) => {
                let reference = SecretRef::new();
                self.collected.push((reference.clone(), secret));
                Ok(Some(reference))
            }
            None => {
                ctx.add_reason(skipped_reason(&request.field));
                Ok(None)
            }
        }
    }

    pub(super) fn into_collected(self) -> Vec<(SecretRef, Secret)> {
        self.collected
    }
}

/// The reason of a skipped secret; for a password it is the exact text of
/// SPEC §8.3.5.
fn skipped_reason(field: &SecretField) -> String {
    let what = match field {
        SecretField::Password => "password Royal cifrata".to_owned(),
        SecretField::Passphrase => "passphrase Royal cifrata".to_owned(),
        SecretField::PrivateKey => "chiave privata Royal cifrata".to_owned(),
        SecretField::CustomProperty(name) => format!("proprietà protetta `{name}` cifrata"),
    };
    format!("{what} non importata: impostala con `portolano secret set`")
}
