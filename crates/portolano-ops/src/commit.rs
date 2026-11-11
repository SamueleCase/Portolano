//! The single way an operation saves its changes (SPEC §8.4, DEC-022):
//! new secrets go to the vault first, then icons, then the document; if
//! anything after the secrets fails, the secrets just written are removed.
//! Only after a successful save are the secrets that the document no longer
//! references deleted from the vault, so a failure never loses a secret that
//! is still in use.

use std::collections::HashSet;

use portolano_core::{Document, Secret, SecretRef, Store, Vault};
use serde::Serialize;

use crate::Result;
use crate::tree_util::document_secret_refs;

/// A secret that could not be deleted from the vault after the document was
/// saved; the document no longer references it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CleanupFailure {
    /// The secret left in the vault.
    pub reference: SecretRef,
    /// Why it could not be deleted.
    pub reason: String,
}

/// What a commit did to the vault.
#[derive(Debug, Default)]
pub(crate) struct Committed {
    /// Secrets written.
    pub(crate) secrets_written: usize,
    /// Obsolete secrets deleted.
    pub(crate) secrets_deleted: usize,
    /// Obsolete secrets that could not be deleted.
    pub(crate) cleanup_failures: Vec<CleanupFailure>,
}

/// A change ready to be saved: the edited copy of the document, the secrets
/// it references that are not in the vault yet, and the references of the
/// document before the change.
pub(crate) struct Change {
    pub(crate) doc: Document,
    pub(crate) new_secrets: Vec<(SecretRef, Secret)>,
    pub(crate) previous_refs: HashSet<SecretRef>,
}

impl Change {
    /// A change starting from a copy of `doc`.
    pub(crate) fn from(doc: &Document) -> Change {
        Change {
            doc: doc.clone(),
            new_secrets: Vec::new(),
            previous_refs: document_secret_refs(doc),
        }
    }

    /// Adds a secret to write and returns its new reference.
    pub(crate) fn add_secret(&mut self, secret: Secret) -> SecretRef {
        let reference = SecretRef::new();
        self.new_secrets.push((reference.clone(), secret));
        reference
    }

    /// Saves the change: secrets, then `store_icons` (which may update the
    /// document), then the document, then the cleanup of obsolete secrets.
    /// On success the saved document replaces `target`.
    ///
    /// # Errors
    /// Errors of the vault, of `store_icons` and of the store; the vault and
    /// `target` are then as before.
    pub(crate) fn commit(
        mut self,
        store: &Store,
        vault: &dyn Vault,
        target: &mut Document,
        store_icons: impl FnOnce(&mut Document) -> Result<()>,
    ) -> Result<Committed> {
        let current = document_secret_refs(&self.doc);
        let obsolete: Vec<SecretRef> = self.previous_refs.difference(&current).cloned().collect();
        // A wrong master password must stop the operation before the
        // document changes, not leave the obsolete secrets behind after.
        if self.new_secrets.is_empty()
            && let Some(first) = obsolete.first()
        {
            vault.contains(first)?;
        }
        write_secrets(vault, &self.new_secrets)?;
        let saved = store_icons(&mut self.doc).and_then(|()| Ok(store.save(&mut self.doc)?));
        if let Err(error) = saved {
            remove_written(vault, &self.new_secrets);
            return Err(error);
        }
        *target = self.doc;
        let mut committed = Committed {
            secrets_written: self.new_secrets.len(),
            ..Committed::default()
        };
        for reference in obsolete {
            match vault.delete(&reference) {
                Ok(()) => committed.secrets_deleted += 1,
                Err(error) => committed.cleanup_failures.push(CleanupFailure {
                    reference,
                    reason: error.to_string(),
                }),
            }
        }
        Ok(committed)
    }

    /// Saves a change that adds and removes no secret, such as a move, so
    /// that the vault is not needed. On success the saved document replaces
    /// `target`.
    ///
    /// # Errors
    /// Errors of the store; `target` is then as before.
    pub(crate) fn save_without_secrets(
        mut self,
        store: &Store,
        target: &mut Document,
    ) -> Result<()> {
        // Programming errors, not user errors: a caller that queued or
        // dropped secrets must use `commit`, or the vault would keep secrets
        // that nothing references (or lose ones still in use). Checked in
        // debug builds and tests only, since walking the document has a cost.
        debug_assert!(self.new_secrets.is_empty());
        debug_assert_eq!(self.previous_refs, document_secret_refs(&self.doc));
        store.save(&mut self.doc)?;
        *target = self.doc;
        Ok(())
    }
}

/// Writes every secret; on the first failure removes those already written.
fn write_secrets(vault: &dyn Vault, secrets: &[(SecretRef, Secret)]) -> Result<()> {
    for (index, (reference, secret)) in secrets.iter().enumerate() {
        if let Err(error) = vault.set(reference, secret) {
            remove_written(vault, &secrets[..index]);
            return Err(error.into());
        }
    }
    Ok(())
}

/// Best-effort rollback: the operation already failed with a more useful
/// error, so a failure here is only logged.
fn remove_written(vault: &dyn Vault, secrets: &[(SecretRef, Secret)]) {
    for (reference, _) in secrets {
        if let Err(error) = vault.delete(reference) {
            tracing::warn!(reference = reference.as_str(), %error, "rollback del segreto non riuscito");
        }
    }
}

#[cfg(test)]
mod tests {
    use portolano_core::vault::MemoryVault;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::OpsError;
    use crate::testutil::*;

    #[test]
    fn secrets_are_written_and_the_document_saved() {
        let mut stored = stored(vec![]);
        let vault = MemoryVault::default();
        let mut change = Change::from(&stored.doc);
        let reference = change.add_secret(secret(SENTINEL));
        change
            .doc
            .nodes
            .push(credential("Admin", "admin", Some(reference.clone())));
        let committed = change
            .commit(&stored.store, &vault, &mut stored.doc, |_| Ok(()))
            .unwrap();
        assert_eq!(committed.secrets_written, 1);
        assert!(vault.contains(&reference).unwrap());
        assert_eq!(stored.reloaded().nodes.len(), 1);
        assert_eq!(stored.doc.nodes.len(), 1);
    }

    #[test]
    fn failed_save_removes_the_new_secrets_and_keeps_the_document() {
        let mut stored = stored(vec![]);
        let vault = MemoryVault::default();
        let mut change = Change::from(&stored.doc);
        let reference = change.add_secret(secret(SENTINEL));
        change
            .doc
            .nodes
            .push(credential("Admin", "admin", Some(reference.clone())));
        stored.modify_externally();
        let result = change.commit(&stored.store, &vault, &mut stored.doc, |_| Ok(()));
        assert!(matches!(
            result,
            Err(OpsError::Core(
                portolano_core::Error::ModifiedExternally { .. }
            ))
        ));
        assert!(!vault.contains(&reference).unwrap());
        assert!(stored.doc.nodes.is_empty());
    }

    #[test]
    fn failed_icons_remove_the_new_secrets() {
        let mut stored = stored(vec![]);
        let vault = MemoryVault::default();
        let mut change = Change::from(&stored.doc);
        let reference = change.add_secret(secret(SENTINEL));
        let result = change.commit(&stored.store, &vault, &mut stored.doc, |_| {
            Err(OpsError::EmptyName)
        });
        assert!(result.is_err());
        assert!(!vault.contains(&reference).unwrap());
    }

    #[test]
    fn unreferenced_secrets_are_deleted_after_the_save() {
        let (vault, reference) = vault_with(SENTINEL);
        let mut stored = stored(vec![credential("Admin", "admin", Some(reference.clone()))]);
        let mut change = Change::from(&stored.doc);
        change.doc.nodes.clear();
        let committed = change
            .commit(&stored.store, &vault, &mut stored.doc, |_| Ok(()))
            .unwrap();
        assert_eq!(committed.secrets_deleted, 1);
        assert!(!vault.contains(&reference).unwrap());
    }

    #[test]
    fn unreferenced_secrets_stay_when_the_save_fails() {
        let (vault, reference) = vault_with(SENTINEL);
        let mut stored = stored(vec![credential("Admin", "admin", Some(reference.clone()))]);
        let mut change = Change::from(&stored.doc);
        change.doc.nodes.clear();
        stored.modify_externally();
        let result = change.commit(&stored.store, &vault, &mut stored.doc, |_| Ok(()));
        assert!(result.is_err());
        assert!(vault.contains(&reference).unwrap());
    }
}
