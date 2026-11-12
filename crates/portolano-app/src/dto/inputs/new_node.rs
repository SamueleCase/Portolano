//! A node to create (`add_node`).

use portolano_core::Document;
use portolano_ops::nodes::{NewNode, NewNodeKind, NodeSecrets};
use serde::Deserialize;
use uuid::Uuid;

use super::{CredentialChoice, DetailsInput, SecretProperty};
use crate::dto::json::JsonObject;
use crate::dto::secret::SecretText;
use crate::error::IpcError;

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
