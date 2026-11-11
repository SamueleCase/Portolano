//! Data model of a Portolano document (SPEC §3).
//!
//! These types are the on-disk contract: field names, tags and omission rules
//! define the JSON format. The tree is nested both in memory and on disk, and
//! the order of children is the order of the arrays.
//!
//! Every reference from a node to another node, wherever it appears (also
//! inside protocol `settings`), is serialized as an object with the key
//! `"$ref"`, so that the core can find references without knowing the
//! protocols.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::secret::SecretRef;

/// Value of [`Document::format`] for every Portolano document.
pub const FORMAT_ID: &str = "portolano";

/// Schema version written by this build. Older documents are migrated on load.
pub const SCHEMA_VERSION: u32 = 1;

/// Maximum depth of the tree, in levels of nodes: the children of the root
/// are at level 1 (DEC-021).
///
/// Each level adds two levels of JSON nesting (the node and its `children`),
/// and `serde_json` refuses input nested more than 128 levels, so a deeper
/// tree would save but not load again; the limit also keeps recursive code
/// far from a stack overflow. Real documents are a few levels deep.
pub const MAX_DEPTH: usize = 50;

/// Serde helper for `skip_serializing_if`: omits boolean fields that are `false`.
pub fn is_false(value: &bool) -> bool {
    !*value
}

/// A Portolano document: a named tree of folders, connections and credentials.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    /// Always [`FORMAT_ID`].
    pub format: String,
    /// Schema version of the serialized form; [`SCHEMA_VERSION`] once loaded.
    pub schema_version: u32,
    /// Identifier of the document.
    pub id: Uuid,
    /// Display name of the document (the root of the tree).
    pub name: String,
    /// Creation time.
    pub created: DateTime<Utc>,
    /// Last modification time, updated on save.
    pub modified: DateTime<Utc>,
    /// Children of the root, in display order.
    #[serde(default)]
    pub nodes: Vec<Node>,
    /// Fields preserved verbatim, for example from an importer.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
}

impl Document {
    /// Creates an empty document with a new id and the current schema version.
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            format: FORMAT_ID.to_owned(),
            schema_version: SCHEMA_VERSION,
            id: Uuid::new_v4(),
            name: name.into(),
            created: now,
            modified: now,
            nodes: Vec::new(),
            extra: BTreeMap::new(),
        }
    }
}

/// A node of the tree. The fields common to every kind are here; the
/// kind-specific ones are flattened from [`NodeKind`], tagged by `"type"`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    /// Identifier, unique in the document.
    pub id: Uuid,
    /// Display name; not necessarily unique among siblings.
    pub name: String,
    /// Short description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Free-form notes, plain text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// Icon shown next to the node.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<Icon>,
    /// Color in `#RRGGBB` form.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Whether the node is marked as favorite.
    #[serde(default, skip_serializing_if = "is_false")]
    pub favorite: bool,
    /// Free-form tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// User-defined properties, in display order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_properties: Vec<CustomProperty>,
    /// Creation time.
    pub created: DateTime<Utc>,
    /// Last modification time.
    pub modified: DateTime<Utc>,
    /// Imported fields without a mapping, with keys such as `"royal.<Field>"`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
    /// Origin of an imported node.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceInfo>,
    /// Kind-specific data.
    #[serde(flatten)]
    pub kind: NodeKind,
}

impl Node {
    /// Creates an empty folder whose descendants inherit their credential.
    pub fn folder(name: impl Into<String>) -> Self {
        Self::with_kind(
            name.into(),
            NodeKind::Folder(Folder {
                children: Vec::new(),
                credential: CredentialSpec::Inherit,
            }),
        )
    }

    /// Creates a connection with the protocol's default port, an inherited
    /// credential and no settings.
    pub fn connection(
        name: impl Into<String>,
        protocol: impl Into<String>,
        host: impl Into<String>,
    ) -> Self {
        Self::with_kind(
            name.into(),
            NodeKind::Connection(Connection {
                protocol: protocol.into(),
                host: host.into(),
                port: None,
                credential: CredentialSpec::Inherit,
                settings: serde_json::Map::new(),
            }),
        )
    }

    /// Creates a credential with no fields set.
    pub fn credential(name: impl Into<String>) -> Self {
        Self::with_kind(
            name.into(),
            NodeKind::Credential(Credential {
                username: None,
                domain: None,
                password: None,
                private_key: None,
                passphrase: None,
            }),
        )
    }

    fn with_kind(name: String, kind: NodeKind) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description: None,
            notes: None,
            icon: None,
            color: None,
            favorite: false,
            tags: Vec::new(),
            custom_properties: Vec::new(),
            created: now,
            modified: now,
            extra: BTreeMap::new(),
            source: None,
            kind,
        }
    }

    /// Sets `modified` to the current time.
    pub fn touch(&mut self) {
        self.modified = Utc::now();
    }

    /// Whether the node is a folder.
    pub fn is_folder(&self) -> bool {
        matches!(self.kind, NodeKind::Folder(_))
    }

    /// The folder data, if the node is a folder.
    pub fn as_folder(&self) -> Option<&Folder> {
        match &self.kind {
            NodeKind::Folder(folder) => Some(folder),
            _ => None,
        }
    }

    /// The mutable folder data, if the node is a folder.
    pub fn as_folder_mut(&mut self) -> Option<&mut Folder> {
        match &mut self.kind {
            NodeKind::Folder(folder) => Some(folder),
            _ => None,
        }
    }

    /// The connection data, if the node is a connection.
    pub fn as_connection(&self) -> Option<&Connection> {
        match &self.kind {
            NodeKind::Connection(connection) => Some(connection),
            _ => None,
        }
    }

    /// The mutable connection data, if the node is a connection.
    pub fn as_connection_mut(&mut self) -> Option<&mut Connection> {
        match &mut self.kind {
            NodeKind::Connection(connection) => Some(connection),
            _ => None,
        }
    }

    /// The credential data, if the node is a credential.
    pub fn as_credential(&self) -> Option<&Credential> {
        match &self.kind {
            NodeKind::Credential(credential) => Some(credential),
            _ => None,
        }
    }

    /// The mutable credential data, if the node is a credential.
    pub fn as_credential_mut(&mut self) -> Option<&mut Credential> {
        match &mut self.kind {
            NodeKind::Credential(credential) => Some(credential),
            _ => None,
        }
    }
}

/// Kind-specific part of a [`Node`], serialized with the tag `"type"`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NodeKind {
    /// A container of other nodes.
    Folder(Folder),
    /// A remote connection handled by a protocol.
    Connection(Connection),
    /// Reusable login data.
    Credential(Credential),
    /// An imported object that Portolano cannot handle, kept verbatim.
    Unsupported(Unsupported),
}

/// A folder: an ordered list of children and a default credential for them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Folder {
    /// Children in display order.
    #[serde(default)]
    pub children: Vec<Node>,
    /// Credential inherited by descendants that use [`CredentialSpec::Inherit`].
    #[serde(default, skip_serializing_if = "CredentialSpec::is_inherit")]
    pub credential: CredentialSpec,
}

/// A connection to a remote host.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Connection {
    /// Id of the protocol that handles it, such as `"ssh"` or `"rdp"`.
    pub protocol: String,
    /// Host name or address.
    pub host: String,
    /// Port; `None` means the protocol's default port.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    /// How the login data is obtained.
    #[serde(default, skip_serializing_if = "CredentialSpec::is_inherit")]
    pub credential: CredentialSpec,
    /// Protocol-specific settings. The core does not interpret them, except to
    /// find `"$ref"` references.
    #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
    pub settings: serde_json::Map<String, serde_json::Value>,
}

/// Reusable login data. Secrets are stored in the vault and only referenced.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Credential {
    /// User name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// Windows domain.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    /// Password in the vault.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<SecretRef>,
    /// Private key for SSH.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub private_key: Option<PrivateKey>,
    /// Passphrase of the private key, in the vault.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub passphrase: Option<SecretRef>,
}

/// Where a private key is, serialized with the tag `"kind"`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PrivateKey {
    /// A key file on disk.
    File {
        /// Path of the key file; it may start with `~`.
        path: String,
    },
    /// The key content, stored in the vault.
    Vault {
        /// Reference to the key content.
        secret: SecretRef,
    },
}

/// How a connection or a folder obtains its login data, serialized with the
/// tag `"mode"`. [`CredentialSpec::Inherit`] is the default and is omitted.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum CredentialSpec {
    /// Use the credential of the nearest ancestor folder that defines one.
    #[default]
    Inherit,
    /// No credential.
    None,
    /// Ask the user at connection time.
    Prompt,
    /// Login data stored on the node itself.
    Inline {
        /// User name.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        username: Option<String>,
        /// Windows domain.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        domain: Option<String>,
        /// Password in the vault.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        password: Option<SecretRef>,
    },
    /// A credential node referenced by id.
    ById {
        /// Id of the credential node.
        #[serde(rename = "$ref")]
        id: Uuid,
    },
    /// A credential node referenced by name, resolved at use time.
    ByName {
        /// Name of the credential node.
        name: String,
    },
}

impl CredentialSpec {
    /// Whether this is [`CredentialSpec::Inherit`]; serde helper for
    /// `skip_serializing_if`.
    pub fn is_inherit(&self) -> bool {
        matches!(self, Self::Inherit)
    }
}

/// Icon of a node, serialized with the tag `"kind"`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Icon {
    /// An icon of the Portolano set.
    Builtin {
        /// Icon name.
        name: String,
    },
    /// An icon of the Royal icon library, such as `"Flat/Hardware/Server"`.
    Royal {
        /// Path in the Royal icon library.
        name: String,
    },
    /// A custom image stored in the icon store (SPEC §6.4).
    Custom {
        /// SHA-256 of the image bytes, lowercase hex.
        sha256: String,
        /// MIME type of the image.
        mime: String,
    },
}

/// A user-defined property of a node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomProperty {
    /// Property name.
    pub name: String,
    /// Property value, flattened next to the name.
    #[serde(flatten)]
    pub value: PropertyValue,
}

/// Value of a [`CustomProperty`], serialized with the tag `"kind"`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PropertyValue {
    /// Plain text.
    Text {
        /// The text.
        value: String,
    },
    /// A secret stored in the vault.
    Secret {
        /// Reference to the secret.
        secret: SecretRef,
    },
}

/// An imported object of a type Portolano does not support.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Unsupported {
    /// Type name in the source system, such as `"RoyalVNCConnection"`.
    pub source_type: String,
    /// The complete original fragment (for Royal, XML).
    pub raw: String,
}

/// Origin of an imported node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceInfo {
    /// Source system, such as `"royal"`.
    pub system: String,
    /// Type name in the source system, such as `"RoyalSSHConnection"`.
    pub source_type: String,
    /// Id of the object in the source system.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    /// Name of the source file, without its directory.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// Time of the import.
    pub imported_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests;
