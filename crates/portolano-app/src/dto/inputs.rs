//! What the editor and the dialogs send to the commands that change the
//! document (SPEC-UI §4.3 "Modifiche"), and its translation into the inputs
//! of `portolano-ops`. Secrets arrive as [`SecretText`] and stay secrets.

use std::collections::BTreeMap;

use portolano_core::{CredentialSpec, Document, Node, NodeKind, PropertyValue};
use portolano_ops::delete::DeleteOptions;
use portolano_ops::moves::Placement;
use portolano_ops::nodes::{
    CredentialInput, EditorChanges, IconInput, NewNode, NewNodeKind, NodeDetails, NodeEdit,
    NodeSecrets, SecretChange,
};
use portolano_ops::secrets::SecretSlot;
use serde::Deserialize;
use uuid::Uuid;

use super::json::JsonObject;
use super::secret::SecretText;
use crate::error::IpcError;

/// The credential chosen in the editor.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, specta::Type)]
#[serde(tag = "mode", rename_all = "camelCase")]
pub enum CredentialChoice {
    /// From the parent folder.
    Inherit,
    /// None.
    None,
    /// Asked at connection time.
    Prompt,
    /// A credential node by name, looked up at each connection.
    ByName {
        /// The name.
        name: String,
    },
    /// A credential node.
    ById {
        /// The credential node.
        id: Uuid,
    },
    /// Login data on the node itself; its password is a staged secret.
    Inline {
        /// User name.
        username: Option<String>,
        /// Windows domain.
        domain: Option<String>,
    },
}

impl CredentialChoice {
    fn into_input(self) -> CredentialInput {
        match self {
            CredentialChoice::Inherit => CredentialInput::Inherit,
            CredentialChoice::None => CredentialInput::None,
            CredentialChoice::Prompt => CredentialInput::Prompt,
            CredentialChoice::ByName { name } => CredentialInput::ByName(name),
            // A path of the form `@<uuid>` names the node by id.
            CredentialChoice::ById { id } => CredentialInput::ByPath(format!("@{id}")),
            CredentialChoice::Inline { username, domain } => CredentialInput::Inline {
                username: username.and_then(non_blank),
                domain: domain.and_then(non_blank),
            },
        }
    }
}

/// The icon chosen in the editor.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, specta::Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum IconChoice {
    /// A builtin icon.
    Builtin {
        /// Its name.
        name: String,
    },
    /// An image of the document (`custom_icons`, or `add_icon`).
    Custom {
        /// Its hash.
        sha256: String,
        /// Its type.
        mime: String,
    },
    /// The icon of the kind of node.
    None,
}

impl IconChoice {
    fn into_input(self) -> IconInput {
        match self {
            IconChoice::Builtin { name } => IconInput::Builtin(name),
            IconChoice::Custom { sha256, mime } => IconInput::Stored { sha256, mime },
            IconChoice::None => IconInput::Remove,
        }
    }
}

/// A text property.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, specta::Type)]
pub struct TextProperty {
    /// Its name.
    pub name: String,
    /// Its text.
    pub value: String,
}

/// A secret property typed in the editor.
#[derive(Debug, Deserialize, specta::Type)]
pub struct SecretProperty {
    /// Its name.
    pub name: String,
    /// Its value.
    pub value: SecretText,
}

/// The common fields of a node; an absent field is left as it is.
#[derive(Debug, Default, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", default)]
pub struct DetailsInput {
    /// Description; empty clears it.
    pub description: Option<String>,
    /// Notes; empty clears them.
    pub notes: Option<String>,
    /// Icon.
    pub icon: Option<IconChoice>,
    /// Color `#RRGGBB`; empty clears it.
    pub color: Option<String>,
    /// Favorite.
    pub favorite: Option<bool>,
    /// All the tags, in order.
    pub tags: Option<Vec<String>>,
    /// All the text properties, in order; the secret ones stay.
    pub properties: Option<Vec<TextProperty>>,
    /// Secret properties to remove.
    pub remove_secret_properties: Vec<String>,
}

impl DetailsInput {
    /// The changes to `current` (none for a new node).
    fn into_details(self, current: Option<&Node>) -> NodeDetails {
        let current_tags = current.map(|node| node.tags.clone()).unwrap_or_default();
        let (add_tags, remove_tags) = match self.tags {
            Some(tags) => (
                tags.iter()
                    .filter(|t| !current_tags.contains(t))
                    .cloned()
                    .collect(),
                current_tags
                    .into_iter()
                    .filter(|t| !tags.contains(t))
                    .collect(),
            ),
            None => (Vec::new(), Vec::new()),
        };
        let mut remove_properties = self.remove_secret_properties;
        let mut set_properties = Vec::new();
        if let Some(properties) = self.properties {
            let wanted: BTreeMap<&str, ()> =
                properties.iter().map(|p| (p.name.as_str(), ())).collect();
            remove_properties.extend(
                current
                    .into_iter()
                    .flat_map(|node| &node.custom_properties)
                    .filter(|p| matches!(p.value, PropertyValue::Text { .. }))
                    .filter(|p| !wanted.contains_key(p.name.as_str()))
                    .map(|p| p.name.clone()),
            );
            set_properties = properties.into_iter().map(|p| (p.name, p.value)).collect();
        }
        NodeDetails {
            description: self.description,
            notes: self.notes,
            icon: self.icon.map(IconChoice::into_input),
            color: self.color,
            favorite: self.favorite,
            add_tags,
            remove_tags,
            set_properties,
            remove_properties,
        }
    }
}

/// The secrets typed for a new node.
#[derive(Debug, Default, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", default)]
pub struct NewSecretsInput {
    /// Password.
    pub password: Option<SecretText>,
    /// Passphrase of the private key (credentials).
    pub passphrase: Option<SecretText>,
    /// Private key kept in the vault (credentials).
    pub private_key: Option<SecretText>,
    /// Secret properties.
    pub properties: Vec<SecretProperty>,
}

/// What kind of node to create.
#[derive(Debug, Deserialize, specta::Type)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum NewNodeKindInput {
    /// A folder.
    #[serde(rename_all = "camelCase")]
    Folder {
        /// The credential its descendants inherit.
        credential: CredentialChoice,
    },
    /// A connection.
    #[serde(rename_all = "camelCase")]
    Connection {
        /// Protocol id: `ssh`, `rdp`.
        protocol: String,
        /// Host name or address.
        host: String,
        /// Port; none is the default of the protocol.
        port: Option<u16>,
        /// Its credential.
        credential: CredentialChoice,
        /// Protocol settings, validated by the protocol.
        #[specta(type = JsonObject)]
        settings: serde_json::Map<String, serde_json::Value>,
    },
    /// A credential.
    #[serde(rename_all = "camelCase")]
    Credential {
        /// User name.
        username: Option<String>,
        /// Windows domain.
        domain: Option<String>,
        /// Path of a private key file.
        key_file: Option<String>,
    },
}

/// A node to create.
#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct NewNodeInput {
    /// The folder to create it in; none is the root.
    pub parent: Option<Uuid>,
    /// Its name.
    pub name: String,
    /// Its kind.
    pub kind: NewNodeKindInput,
    /// The common fields.
    #[serde(default)]
    pub details: DetailsInput,
    /// Its secrets.
    #[serde(default)]
    pub secrets: NewSecretsInput,
}

impl NewNodeInput {
    /// The node to create in `doc`.
    ///
    /// # Errors
    /// A parent that does not exist or is not a folder.
    pub fn into_new_node(self, doc: &Document) -> Result<NewNode, IpcError> {
        let parent_path = match self.parent {
            Some(parent) => doc.path_of(parent).ok_or(portolano_ops::OpsError::Core(
                portolano_core::Error::NodeNotFound { id: parent },
            ))?,
            None => String::new(),
        };
        let kind = match self.kind {
            NewNodeKindInput::Folder { credential } => NewNodeKind::FolderWithCredential {
                credential: credential.into_input(),
            },
            NewNodeKindInput::Connection {
                protocol,
                host,
                port,
                credential,
                settings,
            } => NewNodeKind::Connection {
                protocol,
                host,
                port,
                credential: credential.into_input(),
                settings,
            },
            NewNodeKindInput::Credential {
                username,
                domain,
                key_file,
            } => NewNodeKind::Credential {
                username,
                domain,
                key_file,
            },
        };
        Ok(NewNode {
            path: format!(
                "{parent_path}/{}",
                portolano_core::escape_path_segment(&self.name)
            ),
            create_parents: false,
            kind,
            details: self.details.into_details(None),
            secrets: NodeSecrets {
                password: self.secrets.password.map(SecretText::into_secret),
                passphrase: self.secrets.passphrase.map(SecretText::into_secret),
                private_key: self.secrets.private_key.map(SecretText::into_secret),
                properties: self
                    .secrets
                    .properties
                    .into_iter()
                    .map(|property| (property.name, property.value.into_secret()))
                    .collect(),
            },
        })
    }
}

/// A secret of the editor changed by "Imposta", "Sostituisci" or "Cancella",
/// applied by "Salva" (SPEC-UI §5.4).
#[derive(Debug, Deserialize, specta::Type)]
#[serde(tag = "action", rename_all = "camelCase")]
pub enum StagedSecret {
    /// The new value.
    Set {
        /// The secret.
        value: SecretText,
    },
    /// Remove it.
    Clear,
}

impl StagedSecret {
    fn into_change(self) -> SecretChange {
        match self {
            StagedSecret::Set { value } => SecretChange::Set(value.into_secret()),
            StagedSecret::Clear => SecretChange::Clear,
        }
    }
}

/// The secrets staged in the editor.
#[derive(Debug, Default, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", default)]
pub struct StagedSecretsInput {
    /// Password.
    pub password: Option<StagedSecret>,
    /// Passphrase of the private key.
    pub passphrase: Option<StagedSecret>,
    /// Private key kept in the vault.
    pub private_key: Option<StagedSecret>,
    /// Secret properties to set, each with a new reference.
    pub properties: Vec<SecretProperty>,
}

/// The changes of the editor to an existing node; an absent field is left
/// as it is.
#[derive(Debug, Default, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", default)]
pub struct NodeEditInput {
    /// New name.
    pub name: Option<String>,
    /// The common fields.
    pub details: DetailsInput,
    /// Host (connections).
    pub host: Option<String>,
    /// Port (connections).
    pub port: Option<u16>,
    /// Port back to the default of the protocol (connections).
    pub default_port: bool,
    /// All the settings of the protocol (connections), validated by it.
    #[specta(type = Option<JsonObject>)]
    pub settings: Option<serde_json::Map<String, serde_json::Value>>,
    /// Credential (folders and connections).
    pub credential: Option<CredentialChoice>,
    /// User name (credentials).
    pub username: Option<String>,
    /// Windows domain (credentials).
    pub domain: Option<String>,
    /// Private key file (credentials); empty removes it.
    pub key_file: Option<String>,
    /// The staged secrets.
    pub secrets: StagedSecretsInput,
}

impl NodeEditInput {
    /// The changes to `current`, in the terms of `portolano-ops`.
    pub fn into_edit(self, current: &Node) -> (NodeEdit, EditorChanges) {
        let (settings_set, settings_unset) = match (self.settings, current.as_connection()) {
            (Some(settings), Some(connection)) => (
                settings.clone().into_iter().collect(),
                connection
                    .settings
                    .keys()
                    .filter(|key| !settings.contains_key(*key))
                    .cloned()
                    .collect(),
            ),
            (Some(settings), None) => (settings.into_iter().collect(), Vec::new()),
            (None, _) => (Vec::new(), Vec::new()),
        };
        let (credential, username, domain) =
            login_change(self.credential, self.username, self.domain, current);
        let edit = NodeEdit {
            name: self.name,
            details: self.details.into_details(Some(current)),
            host: self.host,
            port: self.port,
            settings_set,
            settings_unset,
            credential,
            username,
            domain,
            secret_properties: self
                .secrets
                .properties
                .into_iter()
                .map(|property| (property.name, property.value.into_secret()))
                .collect(),
        };
        let changes = EditorChanges {
            password: self.secrets.password.map(StagedSecret::into_change),
            passphrase: self.secrets.passphrase.map(StagedSecret::into_change),
            private_key: self.secrets.private_key.map(StagedSecret::into_change),
            key_file: self.key_file,
            default_port: self.default_port,
        };
        (edit, changes)
    }
}

/// The credential, user and domain of an edit. An inline choice on a node
/// whose credential is already inline changes only user and domain, so
/// that its password stays.
fn login_change(
    choice: Option<CredentialChoice>,
    username: Option<String>,
    domain: Option<String>,
    current: &Node,
) -> (Option<CredentialInput>, Option<String>, Option<String>) {
    let spec = match &current.kind {
        NodeKind::Folder(folder) => Some(&folder.credential),
        NodeKind::Connection(connection) => Some(&connection.credential),
        NodeKind::Credential(_) | NodeKind::Unsupported(_) => None,
    };
    let already_inline = matches!(spec, Some(CredentialSpec::Inline { .. }));
    match choice {
        Some(CredentialChoice::Inline {
            username: new_user,
            domain: new_domain,
        }) if already_inline => (
            None,
            Some(new_user.unwrap_or_default()),
            Some(new_domain.unwrap_or_default()),
        ),
        Some(choice) => (Some(choice.into_input()), None, None),
        None => (None, username, domain),
    }
}

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

fn non_blank(value: String) -> Option<String> {
    (!value.trim().is_empty()).then_some(value)
}

#[cfg(test)]
mod tests;
