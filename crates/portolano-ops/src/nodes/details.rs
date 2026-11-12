//! The fields every node has: description, notes, icon, color, favorite,
//! tags and custom properties.

use portolano_core::{CustomProperty, Document, Icon, Node, PropertyValue, SecretRef, Store};
use uuid::Uuid;

use super::non_empty;
use crate::{OpsError, Result};

/// A new icon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IconInput {
    /// An icon of the Portolano set.
    Builtin(String),
    /// An icon of the Royal library, such as `Flat/Hardware/Server`.
    Royal(String),
    /// An image already in the icon store of the document, as listed by
    /// [`IconStore::list`](portolano_core::IconStore::list).
    Stored {
        /// Its hash.
        sha256: String,
        /// Its type.
        mime: String,
    },
    /// A custom image, stored in the icon store of the document.
    Image {
        /// The image bytes.
        bytes: Vec<u8>,
        /// The MIME type declared by the caller; the stored type is
        /// recognized from the bytes.
        declared_mime: String,
    },
    /// No icon.
    Remove,
}

/// Changes to the common fields; `None` and empty lists leave a field as it
/// is, an empty string clears an optional text field.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NodeDetails {
    /// Description.
    pub description: Option<String>,
    /// Notes.
    pub notes: Option<String>,
    /// Icon.
    pub icon: Option<IconInput>,
    /// Color `#RRGGBB`.
    pub color: Option<String>,
    /// Favorite.
    pub favorite: Option<bool>,
    /// Tags to add.
    pub add_tags: Vec<String>,
    /// Tags to remove.
    pub remove_tags: Vec<String>,
    /// Text properties to set, replacing a property with the same name.
    pub set_properties: Vec<(String, String)>,
    /// Properties to remove, by name.
    pub remove_properties: Vec<String>,
}

/// A custom image to store, or to find in the store, before the document is
/// saved.
pub(crate) enum PendingImage {
    /// New bytes.
    New {
        bytes: Vec<u8>,
        declared_mime: String,
    },
    /// An image the store should already have.
    Stored(Icon),
}

impl PendingImage {
    /// Stores the image, or checks that the store has it, and sets it as the
    /// icon of node `id`.
    ///
    /// # Errors
    /// Errors of the icon store, such as an image type that is not accepted;
    /// [`OpsError::IconNotStored`] for an image the store does not have.
    pub(crate) fn store(self, store: &Store, doc: &mut Document, id: Uuid) -> Result<()> {
        let icons = store.icons();
        let icon = match self {
            PendingImage::New {
                bytes,
                declared_mime,
            } => icons.add(&bytes, &declared_mime)?,
            PendingImage::Stored(icon) => {
                if !icons.path_for(&icon).is_some_and(|path| path.is_file()) {
                    return Err(OpsError::IconNotStored);
                }
                icon
            }
        };
        if let Some(node) = doc.find_mut(id) {
            node.icon = Some(icon);
        }
        Ok(())
    }
}

/// Applies `details` to `node`; a custom image is returned to be stored
/// later, in the order of the commit (secrets, icons, document).
///
/// # Errors
/// [`OpsError::InvalidColor`].
pub(crate) fn apply_details(node: &mut Node, details: NodeDetails) -> Result<Option<PendingImage>> {
    if let Some(color) = details.color {
        node.color = non_empty(color).map(checked_color).transpose()?;
    }
    if let Some(description) = details.description {
        node.description = non_empty(description);
    }
    if let Some(notes) = details.notes {
        node.notes = non_empty(notes);
    }
    if let Some(favorite) = details.favorite {
        node.favorite = favorite;
    }
    node.tags.retain(|tag| !details.remove_tags.contains(tag));
    for tag in details.add_tags {
        if !node.tags.contains(&tag) {
            node.tags.push(tag);
        }
    }
    node.custom_properties
        .retain(|property| !details.remove_properties.contains(&property.name));
    for (name, value) in details.set_properties {
        set_property(node, name, PropertyValue::Text { value });
    }
    Ok(match details.icon {
        None => None,
        Some(IconInput::Builtin(name)) => replace_icon(node, Some(Icon::Builtin { name })),
        Some(IconInput::Royal(name)) => replace_icon(node, Some(Icon::Royal { name })),
        Some(IconInput::Remove) => replace_icon(node, None),
        Some(IconInput::Stored { sha256, mime }) => {
            Some(PendingImage::Stored(Icon::Custom { sha256, mime }))
        }
        Some(IconInput::Image {
            bytes,
            declared_mime,
        }) => Some(PendingImage::New {
            bytes,
            declared_mime,
        }),
    })
}

fn replace_icon(node: &mut Node, icon: Option<Icon>) -> Option<PendingImage> {
    node.icon = icon;
    None
}

/// Sets the secret properties of `node` to their new references.
pub(crate) fn set_secret_properties(node: &mut Node, properties: Vec<(String, SecretRef)>) {
    for (name, secret) in properties {
        set_property(node, name, PropertyValue::Secret { secret });
    }
}

/// Replaces the property called `name`, or appends it.
fn set_property(node: &mut Node, name: String, value: PropertyValue) {
    match node
        .custom_properties
        .iter_mut()
        .find(|property| property.name == name)
    {
        Some(property) => property.value = value,
        None => node.custom_properties.push(CustomProperty { name, value }),
    }
}

fn checked_color(color: String) -> Result<String> {
    let valid = color.len() == 7
        && color.starts_with('#')
        && color[1..].chars().all(|c| c.is_ascii_hexdigit());
    if valid {
        Ok(color)
    } else {
        Err(OpsError::InvalidColor { value: color })
    }
}
