//! The master password of the file vault, typed in the unlock dialog
//! (SPEC-UI §4.1, §5.4).

use std::sync::{Arc, Mutex};

use portolano_core::{MasterPasswordSource, Secret, VaultError};

use super::recover;

/// Holds the master password between the unlock command and the opening
/// of the vault, which asks for it. An empty slot means a locked vault: the
/// vault cannot open and every operation on secrets fails with
/// [`VaultError::MasterPasswordUnavailable`] (`VaultLocked` for the
/// frontend).
#[derive(Default)]
pub struct UnlockSlot {
    password: Mutex<Option<Secret>>,
}

impl UnlockSlot {
    /// Puts `password` in the slot for the next opening of the vault.
    pub fn fill(&self, password: Secret) {
        *recover(self.password.lock()) = Some(password);
    }

    /// Empties the slot, wiping the password: once the vault is open it
    /// keeps the derived key, not the password.
    pub fn clear(&self) {
        recover(self.password.lock()).take();
    }
}

impl MasterPasswordSource for UnlockSlot {
    fn master_password(&self, _creating: bool) -> Result<Secret, VaultError> {
        match recover(self.password.lock()).as_ref() {
            Some(password) => Ok(Secret::new(password.expose().to_owned())),
            None => Err(VaultError::MasterPasswordUnavailable {
                reason: "il vault è bloccato: sbloccalo con la master password".to_owned(),
            }),
        }
    }
}

/// The slot shared between the state and the vault that asks it.
pub(super) struct SharedSlot(pub(super) Arc<UnlockSlot>);

impl MasterPasswordSource for SharedSlot {
    fn master_password(&self, creating: bool) -> Result<Secret, VaultError> {
        self.0.master_password(creating)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_slot_means_a_locked_vault() {
        let slot = UnlockSlot::default();

        let result = slot.master_password(false);

        assert!(matches!(
            result,
            Err(VaultError::MasterPasswordUnavailable { .. })
        ));
    }

    #[test]
    fn filled_slot_answers_until_cleared() {
        let slot = UnlockSlot::default();
        slot.fill(Secret::new("test-only-master-password".to_owned()));

        assert_eq!(
            slot.master_password(false).unwrap().expose(),
            "test-only-master-password"
        );
        slot.clear();
        assert!(slot.master_password(false).is_err());
    }
}
