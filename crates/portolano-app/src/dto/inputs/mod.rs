//! What the editor and the dialogs send to the commands that change the
//! document (SPEC-UI §4.3 "Modifiche"), and its translation into the inputs
//! of `portolano-ops`. Secrets arrive as [`SecretText`] and stay secrets.

mod actions;
mod edit;
mod new_node;

use std::collections::BTreeMap;

use portolano_core::{Node, PropertyValue};
use portolano_ops::nodes::{CredentialInput, IconInput, NodeDetails};
use serde::Deserialize;
use uuid::Uuid;

use super::secret::SecretText;
use crate::error::IpcError;

pub use actions::{DeleteOptionsInput, PlacementInput, SecretSlotInput};
pub use edit::NodeEditInput;
pub use new_node::NewNodeInput;

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
    pub(super) fn into_input(self) -> CredentialInput {
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
    pub(super) fn into_input(self) -> IconInput {
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
    pub(super) fn into_details(self, current: Option<&Node>) -> NodeDetails {
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

/// Refuses a private key file that is not an absolute path or a path under
/// the home directory (`~/…`), or that holds control characters (DEC-031.4).
/// The app never opens it: it is a datum of the document for `ssh`, typed
/// or chosen with `choose_key_file`. Empty clears the key file.
fn check_key_file(path: &str) -> Result<(), IpcError> {
    let usable = path.is_empty() || path.starts_with('/') || path.starts_with("~/");
    if !usable || path.chars().any(char::is_control) {
        return Err(IpcError::invalid(
            "key_file",
            format!(
                "file della chiave non valido: indica un percorso assoluto o che inizia con ~/ ({path:?})"
            ),
        ));
    }
    Ok(())
}

fn non_blank(value: String) -> Option<String> {
    (!value.trim().is_empty()).then_some(value)
}

#[cfg(test)]
mod tests;
