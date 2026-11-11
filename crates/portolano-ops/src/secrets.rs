//! The secrets of one node: `secret set`, `secret clear`, `secret status`
//! (SPEC §10.2).

use std::fmt;

use portolano_core::{
    CredentialSpec, Document, Node, NodeKind, PrivateKey, PropertyValue, Secret, SecretRef, Store,
    Vault,
};
use serde::Serialize;
use uuid::Uuid;

use crate::commit::Change;
use crate::nodes::{EditOutcome, credential_spec, credential_spec_mut};
use crate::tree_util::node_path;
use crate::{OpsError, Result};

/// A place of a node that holds a secret.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SecretSlot {
    /// The password of a credential, or of the inline credential of a folder
    /// or a connection.
    Password,
    /// The passphrase of a credential's private key.
    Passphrase,
    /// The private key of a credential, stored in the vault.
    PrivateKey,
    /// A secret custom property, by name.
    Property(String),
}

impl fmt::Display for SecretSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecretSlot::Password => f.write_str("la password"),
            SecretSlot::Passphrase => f.write_str("la passphrase"),
            SecretSlot::PrivateKey => f.write_str("la chiave privata"),
            SecretSlot::Property(name) => write!(f, "la proprietà segreta '{name}'"),
        }
    }
}

/// Whether one secret of a node is in the vault.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SecretStatus {
    /// Where the secret is.
    pub slot: SecretSlot,
    /// Its reference.
    pub reference: SecretRef,
    /// Whether the vault has it.
    pub present: bool,
}

/// Stores `secret` in `slot` of node `id`.
///
/// If the slot already references a secret, the vault entry is overwritten
/// in place and the document does not change, so that a document shared by
/// machines with different vaults keeps working on all of them
/// (DEC-022.1). Otherwise a new reference is written to the vault and to
/// the document, which is saved; the secret is removed again if the save
/// fails. A key file of a credential is replaced by the key in the vault.
///
/// Returns the reference of the secret.
///
/// # Errors
/// [`portolano_core::Error::NodeNotFound`],
/// [`OpsError::SecretNotApplicable`] (a password on a connection whose
/// credential is not inline, a passphrase on a connection, …),
/// [`OpsError::PropertyIsText`], errors of the vault and the store.
pub fn set_secret(
    store: &Store,
    doc: &mut Document,
    vault: &dyn Vault,
    id: Uuid,
    slot: SecretSlot,
    secret: Secret,
) -> Result<SecretRef> {
    let path = node_path(doc, id);
    if let Some(reference) = current_ref(find(doc, id)?, &slot, &path)? {
        vault.set(&reference, &secret)?;
        return Ok(reference);
    }
    let mut change = Change::from(doc);
    let reference = change.add_secret(secret);
    let node = change
        .doc
        .find_mut(id)
        .ok_or(portolano_core::Error::NodeNotFound { id })?;
    assign(node, &slot, Some(reference.clone()));
    node.touch();
    change.commit(store, vault, doc, |_| Ok(()))?;
    Ok(reference)
}

/// Removes the secret in `slot` of node `id` from the document, saves it,
/// then deletes the secret from the vault. A secret property is removed
/// altogether.
///
/// # Errors
/// [`portolano_core::Error::NodeNotFound`],
/// [`OpsError::SecretNotApplicable`], [`OpsError::PropertyIsText`],
/// [`OpsError::SecretNotSet`], errors of the vault and the store.
pub fn clear_secret(
    store: &Store,
    doc: &mut Document,
    vault: &dyn Vault,
    id: Uuid,
    slot: SecretSlot,
) -> Result<EditOutcome> {
    let path = node_path(doc, id);
    if current_ref(find(doc, id)?, &slot, &path)?.is_none() {
        return Err(OpsError::SecretNotSet { path, slot });
    }
    let mut change = Change::from(doc);
    let node = change
        .doc
        .find_mut(id)
        .ok_or(portolano_core::Error::NodeNotFound { id })?;
    assign(node, &slot, None);
    node.touch();
    let committed = change.commit(store, vault, doc, |_| Ok(()))?;
    Ok(EditOutcome {
        path,
        secrets_written: 0,
        secrets_deleted: committed.secrets_deleted,
        cleanup_failures: committed.cleanup_failures,
    })
}

/// The secrets of node `id` itself (not of its descendants) and whether the
/// vault has them; never their values.
///
/// # Errors
/// [`portolano_core::Error::NodeNotFound`], errors of the vault.
pub fn secret_status(doc: &Document, vault: &dyn Vault, id: Uuid) -> Result<Vec<SecretStatus>> {
    let node = find(doc, id)?;
    let mut slots: Vec<(SecretSlot, SecretRef)> = Vec::new();
    match &node.kind {
        NodeKind::Credential(credential) => {
            slots.extend(
                credential
                    .password
                    .clone()
                    .map(|r| (SecretSlot::Password, r)),
            );
            if let Some(PrivateKey::Vault { secret }) = &credential.private_key {
                slots.push((SecretSlot::PrivateKey, secret.clone()));
            }
            slots.extend(
                credential
                    .passphrase
                    .clone()
                    .map(|r| (SecretSlot::Passphrase, r)),
            );
        }
        _ => slots.extend(inline_password(node).map(|r| (SecretSlot::Password, r))),
    }
    for property in &node.custom_properties {
        if let PropertyValue::Secret { secret } = &property.value {
            slots.push((SecretSlot::Property(property.name.clone()), secret.clone()));
        }
    }
    slots
        .into_iter()
        .map(|(slot, reference)| {
            Ok(SecretStatus {
                present: vault.contains(&reference)?,
                slot,
                reference,
            })
        })
        .collect()
}

fn find(doc: &Document, id: Uuid) -> Result<&Node> {
    Ok(doc
        .find(id)
        .ok_or(portolano_core::Error::NodeNotFound { id })?)
}

/// The reference currently in `slot`, if the node can hold that secret.
fn current_ref(node: &Node, slot: &SecretSlot, path: &str) -> Result<Option<SecretRef>> {
    let not_applicable = || OpsError::SecretNotApplicable {
        path: path.to_owned(),
        slot: slot.clone(),
    };
    match (slot, &node.kind) {
        (SecretSlot::Property(name), _) => {
            match node.custom_properties.iter().find(|p| &p.name == name) {
                None => Ok(None),
                Some(property) => match &property.value {
                    PropertyValue::Secret { secret } => Ok(Some(secret.clone())),
                    PropertyValue::Text { .. } => Err(OpsError::PropertyIsText {
                        path: path.to_owned(),
                        name: name.clone(),
                    }),
                },
            }
        }
        (SecretSlot::Password, NodeKind::Credential(credential)) => Ok(credential.password.clone()),
        (SecretSlot::Password, NodeKind::Folder(_) | NodeKind::Connection(_)) => {
            match credential_spec(node) {
                Some(CredentialSpec::Inline { password, .. }) => Ok(password.clone()),
                _ => Err(not_applicable()),
            }
        }
        (SecretSlot::Passphrase, NodeKind::Credential(credential)) => {
            Ok(credential.passphrase.clone())
        }
        (SecretSlot::PrivateKey, NodeKind::Credential(credential)) => {
            Ok(match &credential.private_key {
                Some(PrivateKey::Vault { secret }) => Some(secret.clone()),
                Some(PrivateKey::File { .. }) | None => None,
            })
        }
        _ => Err(not_applicable()),
    }
}

/// Puts `reference` in `slot`, or empties it; the slot was checked by
/// [`current_ref`].
fn assign(node: &mut Node, slot: &SecretSlot, reference: Option<SecretRef>) {
    if let SecretSlot::Property(name) = slot {
        match reference {
            Some(secret) => {
                crate::nodes::set_secret_properties(node, vec![(name.clone(), secret)]);
            }
            None => node.custom_properties.retain(|p| &p.name != name),
        }
        return;
    }
    if let NodeKind::Credential(credential) = &mut node.kind {
        match slot {
            SecretSlot::Password => credential.password = reference,
            SecretSlot::Passphrase => credential.passphrase = reference,
            SecretSlot::PrivateKey => {
                credential.private_key = reference.map(|secret| PrivateKey::Vault { secret });
            }
            SecretSlot::Property(_) => {}
        }
        return;
    }
    if let Some(CredentialSpec::Inline { password, .. }) = credential_spec_mut(node) {
        *password = reference;
    }
}

fn inline_password(node: &Node) -> Option<SecretRef> {
    match credential_spec(node) {
        Some(CredentialSpec::Inline { password, .. }) => password.clone(),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
