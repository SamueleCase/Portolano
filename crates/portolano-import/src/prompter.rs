//! Requests for the secrets of an import (SPEC §8.1, D3): encrypted secrets
//! of the source cannot be decrypted, so the user may type them.

// Stub (WP0): the parameters are used once WP5 implements the bodies.
#![expect(unused_variables)]

use portolano_core::Secret;

use crate::ImportError;

/// Asks the user for the secrets an importer cannot read.
pub trait SecretPrompter {
    /// The secret for `req`, or `None` to skip it.
    ///
    /// # Errors
    /// [`ImportError::Prompt`] if the input cannot be read,
    /// [`ImportError::Cancelled`] if the user aborts.
    fn ask(&mut self, req: &SecretRequest) -> Result<Option<Secret>, ImportError>;
}

/// A secret an importer needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretRequest {
    /// Path of the node in the imported tree.
    pub node_path: String,
    /// User name of the node, to help the user recognize it.
    pub username: Option<String>,
    /// Which secret is requested.
    pub field: SecretField,
}

/// The kind of secret requested.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecretField {
    /// A password.
    Password,
    /// The passphrase of a private key.
    Passphrase,
    /// The content of a private key.
    PrivateKey,
    /// A protected custom property, by name.
    CustomProperty(String),
}

/// A prompter that never asks and always skips.
#[derive(Debug, Clone, Copy, Default)]
pub struct NonInteractive;

impl SecretPrompter for NonInteractive {
    fn ask(&mut self, req: &SecretRequest) -> Result<Option<Secret>, ImportError> {
        todo!()
    }
}
