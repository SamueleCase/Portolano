//! Credential resolution (SPEC §4.3, DEC-010).

use std::collections::HashMap;

use uuid::Uuid;

use crate::error::{Error, Result};
use crate::model::{Credential, CredentialSpec, Document, Node, NodeKind, PrivateKey};
use crate::path::lowercase_name;
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
    /// Resolves the credential of a node, by kind (DEC-010):
    /// - a connection or a folder resolves its own `credential` spec, as
    ///   [`Document::resolve_spec`] with the node as context;
    /// - a credential returns its own fields, with
    ///   `origin = CredentialOrigin::Credential { node: <itself> }`;
    /// - an unsupported node has nothing to resolve.
    ///
    /// # Errors
    /// [`Error::NodeNotFound`](crate::Error::NodeNotFound) if the node does
    /// not exist, [`Error::NoCredential`](crate::Error::NoCredential) for an
    /// unsupported node, plus the errors of [`Document::resolve_spec`].
    pub fn resolve_credential(&self, node_id: Uuid) -> Result<ResolvedCredential> {
        let node = self
            .find(node_id)
            .ok_or(Error::NodeNotFound { id: node_id })?;
        match &node.kind {
            NodeKind::Folder(folder) => self.resolve_spec(node_id, &folder.credential),
            NodeKind::Connection(connection) => self.resolve_spec(node_id, &connection.credential),
            NodeKind::Credential(credential) => {
                Ok(ResolvedCredential::from_credential(node_id, credential))
            }
            NodeKind::Unsupported(_) => Err(Error::NoCredential { id: node_id }),
        }
    }

    /// Resolves `spec` as if it belonged to the node `context`: `Inherit`
    /// continues from the parent of `context`.
    ///
    /// # Errors
    /// [`Error::DanglingReference`](crate::Error::DanglingReference) or
    /// [`Error::NotACredential`](crate::Error::NotACredential) for a bad
    /// `ById`; [`Error::NotFound`](crate::Error::NotFound) or
    /// [`Error::Ambiguous`](crate::Error::Ambiguous) for a bad `ByName`;
    /// [`Error::NodeNotFound`](crate::Error::NodeNotFound) if `Inherit` has to
    /// look up a `context` that does not exist.
    pub fn resolve_spec(&self, context: Uuid, spec: &CredentialSpec) -> Result<ResolvedCredential> {
        let mut context = context;
        let mut spec = spec;
        while spec.is_inherit() {
            let parent = self
                .parent_of(context)
                .ok_or(Error::NodeNotFound { id: context })?;
            let Some(parent) = parent else {
                return Ok(ResolvedCredential::empty(CredentialOrigin::None));
            };
            // Only folders have children, so a parent always has a spec; the
            // error covers a broken invariant, not bad input.
            spec = self
                .find(parent)
                .and_then(|node| node.as_folder())
                .map(|folder| &folder.credential)
                .ok_or(Error::NotAFolder { id: parent })?;
            context = parent;
        }
        self.resolve_own_spec(context, spec)
    }

    /// The credential node a `ByName` spec refers to: the only credential with
    /// exactly that name or, if there is none, the only one with that name
    /// ignoring case.
    ///
    /// # Errors
    /// [`Error::NotFound`](crate::Error::NotFound) if no credential has the
    /// name, [`Error::Ambiguous`](crate::Error::Ambiguous) if several do.
    pub(crate) fn credential_named(&self, name: &str) -> Result<(Uuid, &Credential)> {
        match CredentialNames::new(self).matching(name) {
            [] => Err(Error::NotFound {
                query: name.to_owned(),
            }),
            [(node, credential)] => Ok((node.id, credential)),
            found => {
                let nodes: Vec<&Node> = found.iter().map(|(node, _)| *node).collect();
                Err(self.ambiguous(name.to_owned(), &nodes))
            }
        }
    }

    /// Resolves a spec that is not `Inherit`, owned by the node `context`.
    fn resolve_own_spec(&self, context: Uuid, spec: &CredentialSpec) -> Result<ResolvedCredential> {
        match spec {
            // `resolve_spec` follows `Inherit` up to a real spec or the root,
            // and inheriting from the root means no credential.
            CredentialSpec::Inherit | CredentialSpec::None => {
                Ok(ResolvedCredential::empty(CredentialOrigin::None))
            }
            CredentialSpec::Prompt => Ok(ResolvedCredential::empty(CredentialOrigin::Prompt)),
            CredentialSpec::Inline {
                username,
                domain,
                password,
            } => Ok(ResolvedCredential {
                origin: CredentialOrigin::Inline { node: context },
                username: username.clone(),
                domain: domain.clone(),
                password: password.clone(),
                private_key: None,
                passphrase: None,
            }),
            CredentialSpec::ById { id } => {
                let target = self.find(*id).ok_or(Error::DanglingReference {
                    from: context,
                    target: *id,
                })?;
                let credential = target.as_credential().ok_or(Error::NotACredential {
                    from: context,
                    target: *id,
                })?;
                Ok(ResolvedCredential::from_credential(*id, credential))
            }
            CredentialSpec::ByName { name } => {
                let (id, credential) = self.credential_named(name)?;
                Ok(ResolvedCredential::from_credential(id, credential))
            }
        }
    }
}

/// The credentials of a document indexed by name, so that many `ByName`
/// specs are resolved in one pass over the document (SPEC §4.3).
pub(crate) struct CredentialNames<'a> {
    exact: HashMap<&'a str, Vec<(&'a Node, &'a Credential)>>,
    lowercase: HashMap<String, Vec<(&'a Node, &'a Credential)>>,
}

impl<'a> CredentialNames<'a> {
    /// Indexes every credential of `document`.
    pub(crate) fn new(document: &'a Document) -> Self {
        let mut names = Self {
            exact: HashMap::new(),
            lowercase: HashMap::new(),
        };
        for (_, node) in document.walk() {
            if let Some(credential) = node.as_credential() {
                names
                    .exact
                    .entry(node.name.as_str())
                    .or_default()
                    .push((node, credential));
                names
                    .lowercase
                    .entry(lowercase_name(&node.name))
                    .or_default()
                    .push((node, credential));
            }
        }
        names
    }

    /// The credentials a `ByName` spec with `name` matches: those with
    /// exactly that name or, if there is none, those with that name ignoring
    /// case. The spec resolves only when there is exactly one.
    pub(crate) fn matching(&self, name: &str) -> &[(&'a Node, &'a Credential)] {
        self.exact
            .get(name)
            .or_else(|| self.lowercase.get(&lowercase_name(name)))
            .map_or(&[], Vec::as_slice)
    }

    /// Whether a `ByName` spec with `name` resolves to the node `target`.
    pub(crate) fn resolves_to(&self, name: &str, target: Uuid) -> bool {
        matches!(self.matching(name), [(node, _)] if node.id == target)
    }
}

impl ResolvedCredential {
    fn empty(origin: CredentialOrigin) -> Self {
        Self {
            origin,
            username: None,
            domain: None,
            password: None,
            private_key: None,
            passphrase: None,
        }
    }

    fn from_credential(node: Uuid, credential: &Credential) -> Self {
        Self {
            origin: CredentialOrigin::Credential { node },
            username: credential.username.clone(),
            domain: credential.domain.clone(),
            password: credential.password.clone(),
            private_key: credential.private_key.clone(),
            passphrase: credential.passphrase.clone(),
        }
    }
}

#[cfg(test)]
mod tests;
