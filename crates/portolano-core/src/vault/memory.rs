//! In-memory vault, for tests and the hidden `--vault memory` option.

// Stub (WP0): the parameters are used once WP3 implements the bodies.
#![expect(unused_variables)]

use std::collections::HashMap;
use std::sync::Mutex;

use super::Vault;
use crate::error::VaultError;
use crate::secret::{Secret, SecretRef};

/// A vault that keeps secrets in a map; its content is lost on drop.
#[derive(Debug, Default)]
// Stub (WP0): the field is read once WP3 implements the methods.
#[expect(dead_code)]
pub struct MemoryVault {
    entries: Mutex<HashMap<SecretRef, Secret>>,
}

impl Vault for MemoryVault {
    fn backend(&self) -> &'static str {
        todo!()
    }

    fn get(&self, reference: &SecretRef) -> Result<Option<Secret>, VaultError> {
        todo!()
    }

    fn set(&self, reference: &SecretRef, secret: &Secret) -> Result<(), VaultError> {
        todo!()
    }

    fn delete(&self, reference: &SecretRef) -> Result<(), VaultError> {
        todo!()
    }
}
