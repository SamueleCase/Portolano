//! The import wizard (SPEC-UI §5.4 "Wizard d'import").

use portolano_import::{SecretField, SecretRequest};
use portolano_ops::import::{ConflictPolicy, ImportResult, SecretAnswer};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::secret::SecretText;

/// What to do with nodes of the import that are already in the document.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum ConflictChoice {
    /// Refuse the import.
    #[default]
    Fail,
    /// Replace them, keeping their secrets.
    Update,
    /// Import them as new nodes.
    NewIds,
}

impl From<ConflictChoice> for ConflictPolicy {
    fn from(choice: ConflictChoice) -> Self {
        match choice {
            ConflictChoice::Fail => ConflictPolicy::Fail,
            ConflictChoice::Update => ConflictPolicy::Update,
            ConflictChoice::NewIds => ConflictPolicy::NewIds,
        }
    }
}

/// The options of step 2 of the wizard.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", default)]
pub struct ImportOptionsInput {
    /// The destination folder; none is the root.
    pub into: Option<Uuid>,
    /// Conflicts with existing nodes.
    pub on_conflict: ConflictChoice,
    /// Import the trash of the source too.
    pub include_trash: bool,
}

/// The secret a request is for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SecretFieldView {
    /// A password.
    Password,
    /// The passphrase of a key.
    Passphrase,
    /// The content of a key.
    PrivateKey,
    /// A protected custom property.
    Property {
        /// Its name.
        name: String,
    },
}

impl From<SecretField> for SecretFieldView {
    fn from(field: SecretField) -> Self {
        match field {
            SecretField::Password => SecretFieldView::Password,
            SecretField::Passphrase => SecretFieldView::Passphrase,
            SecretField::PrivateKey => SecretFieldView::PrivateKey,
            SecretField::CustomProperty(name) => SecretFieldView::Property { name },
        }
    }
}

impl From<SecretFieldView> for SecretField {
    fn from(field: SecretFieldView) -> Self {
        match field {
            SecretFieldView::Password => SecretField::Password,
            SecretFieldView::Passphrase => SecretField::Passphrase,
            SecretFieldView::PrivateKey => SecretField::PrivateKey,
            SecretFieldView::Property { name } => SecretField::CustomProperty(name),
        }
    }
}

/// A secret the import asks for ("N / M" of step 3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SecretRequestView {
    /// The node in the imported tree.
    pub node_path: String,
    /// Its user, to recognize it.
    pub username: Option<String>,
    /// The secret.
    pub field: SecretFieldView,
}

impl From<SecretRequest> for SecretRequestView {
    fn from(request: SecretRequest) -> Self {
        Self {
            node_path: request.node_path,
            username: request.username,
            field: request.field.into(),
        }
    }
}

/// What the import would do and the secrets it asks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreviewView {
    /// The result with every secret skipped; nothing was written.
    pub result: ImportResult,
    /// The secrets to ask, in order.
    pub requests: Vec<SecretRequestView>,
}

/// The answer to one request, in the order of the requests; a missing value
/// skips it ("Salta").
#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ImportAnswerInput {
    /// The node of the request.
    pub node_path: String,
    /// The secret of the request.
    pub field: SecretFieldView,
    /// The secret typed, if any.
    pub value: Option<SecretText>,
}

impl From<ImportAnswerInput> for SecretAnswer {
    fn from(answer: ImportAnswerInput) -> Self {
        SecretAnswer {
            node_path: answer.node_path,
            field: answer.field.into(),
            secret: answer.value.map(SecretText::into_secret),
        }
    }
}
