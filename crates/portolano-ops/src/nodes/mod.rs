//! Creation and editing of nodes with their secrets (SPEC §10.2 `add`,
//! `edit`).

mod add;
mod credential;
mod details;
mod edit;
mod jump;

use portolano_core::{Document, Secret, Store};
use serde::Serialize;
use uuid::Uuid;

pub use add::add_node;
pub use credential::CredentialInput;
pub use details::{IconInput, NodeDetails};
pub use edit::edit_node;
pub use jump::JumpInput;

pub(crate) use credential::{credential_spec, credential_spec_mut};
pub(crate) use details::set_secret_properties;

use crate::commit::CleanupFailure;
use crate::tree_util::location_path;
use crate::{OpsError, Result};

/// Secrets typed for a new node. Which ones a node accepts depends on its
/// kind: a connection only takes a password (in an inline credential), a
/// credential takes all of them, every node takes secret properties.
#[derive(Debug, Default)]
pub struct NodeSecrets {
    /// Password.
    pub password: Option<Secret>,
    /// Passphrase of the private key.
    pub passphrase: Option<Secret>,
    /// Content of the private key, stored in the vault.
    pub private_key: Option<Secret>,
    /// Secret custom properties, by name.
    pub properties: Vec<(String, Secret)>,
}

/// What kind of node to create.
#[derive(Debug, Clone)]
pub enum NewNodeKind {
    /// A folder; its descendants inherit no credential from it.
    Folder,
    /// A connection.
    Connection {
        /// Protocol id, such as `"ssh"`; it must be registered.
        protocol: String,
        /// Host name or address.
        host: String,
        /// Port; `None` is the protocol's default.
        port: Option<u16>,
        /// Credential of the connection.
        credential: CredentialInput,
        /// Protocol settings, validated by the protocol.
        settings: serde_json::Map<String, serde_json::Value>,
    },
    /// A credential.
    Credential {
        /// User name.
        username: Option<String>,
        /// Windows domain.
        domain: Option<String>,
        /// Path of a private key file (it may start with `~`).
        key_file: Option<String>,
    },
}

/// A node to create.
#[derive(Debug)]
pub struct NewNode {
    /// Path of the new node; its last segment is the name.
    pub path: String,
    /// Create the missing parent folders (`add -p`).
    pub create_parents: bool,
    /// The kind of node.
    pub kind: NewNodeKind,
    /// Description, icon, tags and the other common fields.
    pub details: NodeDetails,
    /// Secrets to store in the vault.
    pub secrets: NodeSecrets,
}

/// The result of [`add_node`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct AddOutcome {
    /// Id of the new node.
    pub id: Uuid,
    /// Path of the new node.
    pub path: String,
    /// Secrets written to the vault.
    pub secrets_written: usize,
}

/// Changes to an existing node; `None` and empty lists leave a field as it
/// is, an empty string clears an optional text field.
#[derive(Debug, Default)]
pub struct NodeEdit {
    /// New name.
    pub name: Option<String>,
    /// Description, icon, tags and the other common fields.
    pub details: NodeDetails,
    /// New host (connections).
    pub host: Option<String>,
    /// New port (connections).
    pub port: Option<u16>,
    /// Settings to set, by top-level key (connections).
    pub settings_set: Vec<(String, serde_json::Value)>,
    /// Settings to remove, by top-level key (connections).
    pub settings_unset: Vec<String>,
    /// New credential (folders and connections).
    pub credential: Option<CredentialInput>,
    /// New user name: of a credential, or of the inline credential of a
    /// folder or a connection, which becomes inline if it was not.
    pub username: Option<String>,
    /// New domain, like `username`.
    pub domain: Option<String>,
    /// Secret custom properties to set; each gets a new reference and the
    /// previous value, if any, is deleted after saving.
    pub secret_properties: Vec<(String, Secret)>,
}

/// The result of an operation that edits a node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EditOutcome {
    /// Path of the node after the change.
    pub path: String,
    /// Secrets written to the vault.
    pub secrets_written: usize,
    /// Secrets no longer used, deleted from the vault.
    pub secrets_deleted: usize,
    /// Secrets no longer used that could not be deleted.
    pub cleanup_failures: Vec<CleanupFailure>,
}

/// Refuses an empty name, or a name already used by a sibling other than
/// `exclude` (DEC-022.5).
pub(crate) fn check_name(
    doc: &Document,
    parent: Option<Uuid>,
    name: &str,
    exclude: Option<Uuid>,
) -> Result<()> {
    if name.trim().is_empty() {
        return Err(OpsError::EmptyName);
    }
    let taken = doc
        .children(parent)?
        .iter()
        .any(|sibling| sibling.name == name && Some(sibling.id) != exclude);
    if taken {
        return Err(OpsError::NameTaken {
            parent: location_path(doc, parent),
            name: name.to_owned(),
        });
    }
    Ok(())
}

/// Removes images that no node uses any more. A failure only leaves unused
/// files behind, so it is logged instead of failing the operation.
pub(crate) fn collect_unused_icons(store: &Store, doc: &Document) {
    if let Err(error) = store.icons().gc(doc) {
        tracing::warn!(%error, "pulizia delle icone non più usate non riuscita");
    }
}

/// `None` for an empty or blank value: an empty string clears a field.
pub(crate) fn non_empty(value: String) -> Option<String> {
    (!value.trim().is_empty()).then_some(value)
}

#[cfg(test)]
mod tests;
