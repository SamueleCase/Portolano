//! The credential of a folder or a connection, as chosen by the user.

use portolano_core::{CredentialSpec, Document, Node, NodeKind, SecretRef};

use crate::secrets::SecretSlot;
use crate::{OpsError, Result};

/// A credential choice before it is resolved against the document
/// (`--cred inherit|none|prompt|name:<NOME>|<PATH>`, `--user`, `--domain`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialInput {
    /// Inherit from the parent folders.
    Inherit,
    /// No credential.
    None,
    /// Ask at connection time.
    Prompt,
    /// A credential node by name, resolved at use time.
    ByName(String),
    /// A credential node by path, stored by id.
    ByPath(String),
    /// Login data on the node itself.
    Inline {
        /// User name.
        username: Option<String>,
        /// Windows domain.
        domain: Option<String>,
    },
}

impl CredentialInput {
    /// The spec to store; a path must lead to a credential node.
    ///
    /// # Errors
    /// Path resolution errors, [`OpsError::NotACredential`].
    pub fn resolve(self, doc: &Document) -> Result<CredentialSpec> {
        Ok(match self {
            CredentialInput::Inherit => CredentialSpec::Inherit,
            CredentialInput::None => CredentialSpec::None,
            CredentialInput::Prompt => CredentialSpec::Prompt,
            CredentialInput::ByName(name) => CredentialSpec::ByName { name },
            CredentialInput::ByPath(path) => {
                let id = doc.resolve_path(&path)?;
                if doc.find(id).and_then(Node::as_credential).is_none() {
                    return Err(OpsError::NotACredential { path });
                }
                CredentialSpec::ById { id }
            }
            CredentialInput::Inline { username, domain } => CredentialSpec::Inline {
                username,
                domain,
                password: None,
            },
        })
    }
}

/// Puts `password` in `spec`: an inline spec takes it, an inherited one
/// becomes inline with the password only, any other spec refuses it.
///
/// # Errors
/// [`OpsError::SecretNotApplicable`] naming `path`.
pub(crate) fn with_password(
    spec: CredentialSpec,
    password: Option<SecretRef>,
    path: &str,
) -> Result<CredentialSpec> {
    let Some(password) = password else {
        return Ok(spec);
    };
    match spec {
        CredentialSpec::Inline {
            username, domain, ..
        } => Ok(CredentialSpec::Inline {
            username,
            domain,
            password: Some(password),
        }),
        CredentialSpec::Inherit => Ok(CredentialSpec::Inline {
            username: None,
            domain: None,
            password: Some(password),
        }),
        _ => Err(OpsError::SecretNotApplicable {
            path: path.to_owned(),
            slot: SecretSlot::Password,
        }),
    }
}

/// The credential spec of a folder or a connection.
pub(crate) fn credential_spec(node: &Node) -> Option<&CredentialSpec> {
    match &node.kind {
        NodeKind::Folder(folder) => Some(&folder.credential),
        NodeKind::Connection(connection) => Some(&connection.credential),
        NodeKind::Credential(_) | NodeKind::Unsupported(_) => None,
    }
}

/// The credential spec of a folder or a connection, to change it.
pub(crate) fn credential_spec_mut(node: &mut Node) -> Option<&mut CredentialSpec> {
    match &mut node.kind {
        NodeKind::Folder(folder) => Some(&mut folder.credential),
        NodeKind::Connection(connection) => Some(&mut connection.credential),
        NodeKind::Credential(_) | NodeKind::Unsupported(_) => None,
    }
}
