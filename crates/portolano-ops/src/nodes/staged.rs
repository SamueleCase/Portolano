//! The changes of the editor of the app that `edit` of the CLI does not
//! make (SPEC-UI §3.6, §5.4): secrets typed in the editor and kept until
//! "Salva", the key file of a credential, and the port back to the default
//! of the protocol.
//!
//! A staged secret always gets a new reference: the previous one is no
//! longer referenced after the save and is deleted from the vault only
//! then, so that cancelling the editor or a failed save never loses it.

use portolano_core::{CredentialSpec, Node, NodeKind, PrivateKey, Secret, SecretRef};

use super::credential::{credential_spec_mut, with_password};
use super::non_empty;
use crate::commit::Change;
use crate::secrets::SecretSlot;
use crate::{OpsError, Result};

/// A secret of the editor to change when the node is saved.
#[derive(Debug)]
pub enum SecretChange {
    /// "Imposta" or "Sostituisci": the new value.
    Set(Secret),
    /// "Cancella".
    Clear,
}

/// The changes of the editor that [`super::NodeEdit`] does not hold.
#[derive(Debug, Default)]
pub struct EditorChanges {
    /// The password: of a credential, or of the inline credential of a
    /// folder or a connection (an inherited credential becomes inline).
    pub password: Option<SecretChange>,
    /// The passphrase of the private key of a credential.
    pub passphrase: Option<SecretChange>,
    /// The private key of a credential, kept in the vault.
    pub private_key: Option<SecretChange>,
    /// The private key file of a credential (it may start with `~`); an
    /// empty value removes the key file.
    pub key_file: Option<String>,
    /// Port back to the default of the protocol (connections).
    pub default_port: bool,
}

/// [`EditorChanges`] whose new secrets are queued in the change: `Some(Some)`
/// sets a new reference, `Some(None)` clears.
pub(super) struct StagedChanges {
    password: Option<Option<SecretRef>>,
    passphrase: Option<Option<SecretRef>>,
    private_key: Option<Option<SecretRef>>,
    key_file: Option<String>,
    default_port: bool,
}

impl StagedChanges {
    /// Queues the new secrets of `changes` in `change`.
    pub(super) fn new(changes: EditorChanges, change: &mut Change) -> StagedChanges {
        let mut stage = |secret: Option<SecretChange>| {
            secret.map(|secret| match secret {
                SecretChange::Set(value) => Some(change.add_secret(value)),
                SecretChange::Clear => None,
            })
        };
        StagedChanges {
            password: stage(changes.password),
            passphrase: stage(changes.passphrase),
            private_key: stage(changes.private_key),
            key_file: changes.key_file,
            default_port: changes.default_port,
        }
    }

    /// Applies the changes to `node`, named `path` in the errors.
    ///
    /// # Errors
    /// [`OpsError::SecretNotApplicable`] for a secret the node cannot hold,
    /// [`OpsError::SecretNotSet`] to clear a secret that is not set,
    /// [`OpsError::KeyFileAndVaultKey`], [`OpsError::NotEditable`] for a key
    /// file or a port on a node without one.
    pub(super) fn apply(self, node: &mut Node, path: &str) -> Result<()> {
        if self.default_port {
            let connection = node
                .as_connection_mut()
                .ok_or_else(|| not_editable(path, "port"))?;
            connection.port = None;
        }
        if let NodeKind::Credential(credential) = &mut node.kind {
            replace(
                &mut credential.password,
                self.password,
                path,
                SecretSlot::Password,
            )?;
            replace(
                &mut credential.passphrase,
                self.passphrase,
                path,
                SecretSlot::Passphrase,
            )?;
            credential.private_key = key_after(
                credential.private_key.take(),
                self.private_key,
                self.key_file,
                path,
            )?;
            return Ok(());
        }
        refuse(self.passphrase.is_some(), path, SecretSlot::Passphrase)?;
        refuse(self.private_key.is_some(), path, SecretSlot::PrivateKey)?;
        if self.key_file.is_some() {
            return Err(not_editable(path, "key_file"));
        }
        let Some(password) = self.password else {
            return Ok(());
        };
        let spec = credential_spec_mut(node).ok_or_else(|| OpsError::SecretNotApplicable {
            path: path.to_owned(),
            slot: SecretSlot::Password,
        })?;
        *spec = match password {
            Some(reference) => with_password(std::mem::take(spec), Some(reference), path)?,
            None => without_password(std::mem::take(spec), path)?,
        };
        Ok(())
    }
}

/// Sets or clears `slot` of a credential.
fn replace(
    slot: &mut Option<SecretRef>,
    change: Option<Option<SecretRef>>,
    path: &str,
    name: SecretSlot,
) -> Result<()> {
    match change {
        None => Ok(()),
        Some(None) if slot.is_none() => Err(OpsError::SecretNotSet {
            path: path.to_owned(),
            slot: name,
        }),
        Some(new) => {
            *slot = new;
            Ok(())
        }
    }
}

/// The private key of a credential after the editor: a key in the vault, a
/// key file, or none. A key in the vault and a key file together are
/// refused, as by `add`.
fn key_after(
    current: Option<PrivateKey>,
    vault_key: Option<Option<SecretRef>>,
    key_file: Option<String>,
    path: &str,
) -> Result<Option<PrivateKey>> {
    let key_file = key_file.map(non_empty);
    match (vault_key, key_file) {
        (Some(Some(_)), Some(Some(_))) => Err(OpsError::KeyFileAndVaultKey),
        (Some(Some(secret)), _) => Ok(Some(PrivateKey::Vault { secret })),
        (Some(None), file) => match current {
            Some(PrivateKey::Vault { .. }) => {
                Ok(file.flatten().map(|path| PrivateKey::File { path }))
            }
            _ => Err(OpsError::SecretNotSet {
                path: path.to_owned(),
                slot: SecretSlot::PrivateKey,
            }),
        },
        (None, Some(Some(file))) => Ok(Some(PrivateKey::File { path: file })),
        // An empty key file removes a key file, not a key in the vault.
        (None, Some(None)) => Ok(match current {
            Some(PrivateKey::File { .. }) => None,
            other => other,
        }),
        (None, None) => Ok(current),
    }
}

/// `spec` without its inline password.
fn without_password(spec: CredentialSpec, path: &str) -> Result<CredentialSpec> {
    match spec {
        CredentialSpec::Inline {
            username,
            domain,
            password: Some(_),
        } => Ok(CredentialSpec::Inline {
            username,
            domain,
            password: None,
        }),
        _ => Err(OpsError::SecretNotSet {
            path: path.to_owned(),
            slot: SecretSlot::Password,
        }),
    }
}

fn refuse(given: bool, path: &str, slot: SecretSlot) -> Result<()> {
    if given {
        return Err(OpsError::SecretNotApplicable {
            path: path.to_owned(),
            slot,
        });
    }
    Ok(())
}

fn not_editable(path: &str, field: &'static str) -> OpsError {
    OpsError::NotEditable {
        path: path.to_owned(),
        field,
    }
}
