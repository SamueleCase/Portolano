//! The inputs of moves, deletions and the secret commands.

use portolano_ops::delete::DeleteOptions;
use portolano_ops::moves::Placement;
use portolano_ops::secrets::SecretSlot;
use serde::Deserialize;
use uuid::Uuid;

/// Where a moved node goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, specta::Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PlacementInput {
    /// Last.
    End,
    /// At this index of the final list.
    Index {
        /// From 0.
        index: u32,
    },
    /// Before this child of the destination.
    Before {
        /// The sibling.
        id: Uuid,
    },
    /// After this child of the destination.
    After {
        /// The sibling.
        id: Uuid,
    },
}

impl From<PlacementInput> for Placement {
    fn from(placement: PlacementInput) -> Self {
        match placement {
            PlacementInput::End => Placement::End,
            PlacementInput::Index { index } => {
                Placement::Index(usize::try_from(index).unwrap_or(usize::MAX))
            }
            PlacementInput::Before { id } => Placement::Before(id),
            PlacementInput::After { id } => Placement::After(id),
        }
    }
}

/// The confirmation of a deletion.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", default)]
pub struct DeleteOptionsInput {
    /// Delete a folder with its content.
    pub recursive: bool,
    /// "Elimina comunque": delete even if other nodes use it.
    pub force: bool,
}

impl From<DeleteOptionsInput> for DeleteOptions {
    fn from(options: DeleteOptionsInput) -> Self {
        DeleteOptions {
            recursive: options.recursive,
            force: options.force,
        }
    }
}

/// A place of a node that holds a secret, as the secret commands name it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, specta::Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SecretSlotInput {
    /// The password.
    Password,
    /// The passphrase of the private key.
    Passphrase,
    /// The private key in the vault.
    PrivateKey,
    /// A secret property.
    Property {
        /// Its name.
        name: String,
    },
}

impl From<SecretSlotInput> for SecretSlot {
    fn from(slot: SecretSlotInput) -> Self {
        match slot {
            SecretSlotInput::Password => SecretSlot::Password,
            SecretSlotInput::Passphrase => SecretSlot::Passphrase,
            SecretSlotInput::PrivateKey => SecretSlot::PrivateKey,
            SecretSlotInput::Property { name } => SecretSlot::Property(name),
        }
    }
}
