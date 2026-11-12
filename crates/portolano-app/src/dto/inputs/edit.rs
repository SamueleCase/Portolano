//! The changes of the editor to an existing node (`edit_node`), with the
//! secrets staged until "Salva".

use portolano_core::{CredentialSpec, Node, NodeKind};
use portolano_ops::nodes::{CredentialInput, EditorChanges, NodeEdit, SecretChange};
use serde::Deserialize;

use super::{CredentialChoice, DetailsInput, SecretProperty, check_key_file};
use crate::dto::json::JsonObject;
use crate::dto::secret::SecretText;
use crate::error::IpcError;

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
    ///
    /// # Errors
    /// A key file that is not a usable path (DEC-031.4).
    pub fn into_edit(self, current: &Node) -> Result<(NodeEdit, EditorChanges), IpcError> {
        if let Some(key_file) = &self.key_file {
            check_key_file(key_file)?;
        }
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
        Ok((edit, changes))
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
