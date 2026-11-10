//! Credential resolution (SPEC §4.3).

// Stub (WP0): the parameters are used once WP1 implements the bodies.
#![expect(unused_variables)]

use uuid::Uuid;

use crate::error::Result;
use crate::model::{CredentialSpec, Document, PrivateKey};
use crate::secret::SecretRef;

/// Where the effective login data of a node comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialOrigin {
    /// No credential: the inheritance chain reached the root, or a spec says
    /// [`CredentialSpec::None`].
    None,
    /// The user is asked at connection time.
    Prompt,
    /// Inline data of a node.
    Inline {
        /// The node that holds the inline data.
        node: Uuid,
    },
    /// A credential node.
    Credential {
        /// The credential node.
        node: Uuid,
    },
}

/// The effective login data of a node, after following inheritance and
/// references. Secrets are only referenced, never included.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedCredential {
    /// Where the data comes from.
    pub origin: CredentialOrigin,
    /// User name.
    pub username: Option<String>,
    /// Windows domain.
    pub domain: Option<String>,
    /// Password in the vault.
    pub password: Option<SecretRef>,
    /// Private key.
    pub private_key: Option<PrivateKey>,
    /// Passphrase of the private key, in the vault.
    pub passphrase: Option<SecretRef>,
}

impl Document {
    /// Resolves the credential of a node from its own spec (connection or
    /// folder).
    ///
    /// # Errors
    /// [`Error::NodeNotFound`](crate::Error::NodeNotFound) if the node does
    /// not exist, plus the errors of [`Document::resolve_spec`].
    pub fn resolve_credential(&self, node_id: Uuid) -> Result<ResolvedCredential> {
        todo!()
    }

    /// Resolves `spec` as if it belonged to the node `context`: `Inherit`
    /// continues from the parent of `context`.
    ///
    /// # Errors
    /// [`Error::DanglingReference`](crate::Error::DanglingReference) or
    /// [`Error::NotACredential`](crate::Error::NotACredential) for a bad
    /// `ById`; [`Error::NotFound`](crate::Error::NotFound) or
    /// [`Error::Ambiguous`](crate::Error::Ambiguous) for a bad `ByName`.
    pub fn resolve_spec(&self, context: Uuid, spec: &CredentialSpec) -> Result<ResolvedCredential> {
        todo!()
    }
}
