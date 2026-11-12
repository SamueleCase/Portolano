//! The detail of a node (SPEC-UI §4.3, §5.4 "Dettaglio").

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use portolano_connectors::Registry;
use portolano_core::vault::MemoryVault;
use portolano_core::{Document, Node, NodeKind, PrivateKey, PropertyValue, Vault};
use portolano_ops::inspect::{CredentialView, SecretState, resolved_credential, summary};
use portolano_ops::secrets::{SecretSlot, SecretStatus, secret_status};
use portolano_ops::trace::{CredentialTrace, SpecView, resolve_trace};
use portolano_ops::usage::{CredentialUsage, credential_usage};
use portolano_ops::{ErrorCategory, OpsError};
use serde::Serialize;
use uuid::Uuid;

use super::icon::IconView;
use super::json::JsonObject;
use super::tree::{NodeKindView, SecretsBadge, badge_of};
use crate::error::IpcError;

/// Everything the detail and the editor show about a node, without secrets.
#[derive(Debug, Clone, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct NodeDetailsView {
    /// The node.
    pub id: Uuid,
    /// Its name.
    pub name: String,
    /// Its path.
    pub path: String,
    /// The folder that contains it; none for the root.
    pub parent: Option<Uuid>,
    /// Its kind.
    pub kind: NodeKindView,
    /// One-line summary.
    pub summary: String,
    /// Description.
    pub description: Option<String>,
    /// Notes, as text.
    pub notes: Option<String>,
    /// Color `#RRGGBB`.
    pub color: Option<String>,
    /// Icon; none means the icon of the kind.
    pub icon: Option<IconView>,
    /// Favorite.
    pub favorite: bool,
    /// Tags.
    pub tags: Vec<String>,
    /// Custom properties, secret ones as a state.
    pub properties: Vec<PropertyView>,
    /// Creation time.
    pub created: DateTime<Utc>,
    /// Last change.
    pub modified: DateTime<Utc>,
    /// Host, port and settings of a connection.
    pub connection: Option<ConnectionView>,
    /// The own credential spec of a folder or a connection.
    pub credential_spec: Option<SpecView>,
    /// The fields of a credential node.
    pub credential: Option<CredentialFields>,
    /// The effective credential, or why it cannot be resolved.
    pub effective: Option<EffectiveCredential>,
    /// How the effective credential is reached (`resolve_trace`).
    pub trace: Option<CredentialTrace>,
    /// The secrets of the node itself.
    pub secrets: Vec<SecretView>,
    /// Whether the vault is locked: the states of the secrets are unknown
    /// and shown as "Vault bloccato".
    pub vault_locked: bool,
    /// Who uses the node.
    pub usage: CredentialUsage,
    /// Where it was imported from.
    pub source: Option<SourceView>,
    /// Imported fields Portolano does not map, read-only.
    #[specta(type = JsonObject)]
    pub extra: BTreeMap<String, serde_json::Value>,
    /// The original of an unsupported node, read-only.
    pub unsupported: Option<UnsupportedView>,
}

/// A custom property.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PropertyView {
    /// Its name.
    pub name: String,
    /// The text of a text property; none for a secret one.
    pub text: Option<String>,
    /// Whether it is secret.
    pub secret: bool,
}

/// Where a connection points.
#[derive(Debug, Clone, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionView {
    /// The protocol.
    pub protocol: String,
    /// Host name or address.
    pub host: String,
    /// The port; none is the default of the protocol.
    pub port: Option<u16>,
    /// The default port of the protocol, if it is registered.
    pub default_port: Option<u16>,
    /// The settings of the protocol, as stored.
    #[specta(type = JsonObject)]
    pub settings: serde_json::Map<String, serde_json::Value>,
}

/// The fields of a credential node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CredentialFields {
    /// User name.
    pub username: Option<String>,
    /// Windows domain.
    pub domain: Option<String>,
    /// Path of the private key file.
    pub key_file: Option<String>,
}

/// The effective credential of a node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum EffectiveCredential {
    /// Resolved.
    Resolved {
        /// The credential and the state of its secrets.
        credential: CredentialView,
    },
    /// Not resolvable: a missing or ambiguous name, a dangling reference.
    Unresolved {
        /// Why, in Italian.
        reason: String,
    },
}

/// A secret of the node itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SecretView {
    /// Where it is.
    pub slot: SecretSlot,
    /// Whether the vault has it.
    pub state: SecretsBadge,
}

impl From<SecretStatus> for SecretView {
    fn from(status: SecretStatus) -> Self {
        SecretView {
            slot: status.slot,
            state: badge_of(if status.present {
                SecretState::Present
            } else {
                SecretState::Absent
            }),
        }
    }
}

/// Where an imported node comes from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SourceView {
    /// The source system, such as `royal`.
    pub system: String,
    /// Its type there.
    pub source_type: String,
    /// Its id there, to find it in an import report.
    pub source_id: Option<String>,
    /// The imported file name.
    pub file: Option<String>,
    /// When it was imported.
    pub imported_at: DateTime<Utc>,
}

/// A node Portolano does not support.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct UnsupportedView {
    /// Its type in the source system.
    pub source_type: String,
    /// The original fragment, as text.
    pub raw: String,
}

/// The detail of node `id`, with the state of the secrets in `vault`.
///
/// # Errors
/// [`portolano_core::Error::NodeNotFound`], errors of the vault other than a
/// locked vault.
pub fn node_details(
    doc: &Document,
    protocols: &Registry,
    vault: &dyn Vault,
    id: Uuid,
) -> Result<NodeDetailsView, IpcError> {
    let node = doc
        .find(id)
        .ok_or(OpsError::Core(portolano_core::Error::NodeNotFound { id }))?;
    let (secrets, effective, vault_locked) = match secret_views(doc, node, vault) {
        Ok((secrets, effective)) => (secrets, effective, false),
        Err(error) if error.category() == ErrorCategory::VaultLocked => {
            // States are computed against an empty vault and shown as
            // "Vault bloccato" by the frontend wherever a secret is set.
            let (secrets, effective) = secret_views(doc, node, &MemoryVault::default())?;
            let secrets = secrets
                .into_iter()
                .map(|secret| SecretView {
                    state: SecretsBadge::VaultLocked,
                    ..secret
                })
                .collect();
            (secrets, effective, true)
        }
        Err(error) => return Err(error.into()),
    };
    Ok(NodeDetailsView {
        id,
        name: node.name.clone(),
        path: doc.path_of(id).unwrap_or_default(),
        parent: doc.parent_of(id).flatten(),
        kind: NodeKindView::of(node),
        summary: summary(doc, node, protocols),
        description: node.description.clone(),
        notes: node.notes.clone(),
        color: node.color.clone(),
        icon: node.icon.as_ref().and_then(IconView::of),
        favorite: node.favorite,
        tags: node.tags.clone(),
        properties: node.custom_properties.iter().map(property_view).collect(),
        created: node.created,
        modified: node.modified,
        connection: connection_view(node, protocols),
        credential_spec: own_spec(node),
        credential: credential_fields(node),
        effective,
        trace: resolve_trace(doc, id).ok(),
        secrets,
        vault_locked,
        usage: credential_usage(doc, id)?,
        source: node.source.as_ref().map(|source| SourceView {
            system: source.system.clone(),
            source_type: source.source_type.clone(),
            source_id: source.source_id.clone(),
            file: source.file.clone(),
            imported_at: source.imported_at,
        }),
        extra: node.extra.clone(),
        unsupported: match &node.kind {
            NodeKind::Unsupported(unsupported) => Some(UnsupportedView {
                source_type: unsupported.source_type.clone(),
                raw: unsupported.raw.clone(),
            }),
            _ => None,
        },
    })
}

/// The secrets of the node and its effective credential, with their states
/// in `vault`.
fn secret_views(
    doc: &Document,
    node: &Node,
    vault: &dyn Vault,
) -> Result<(Vec<SecretView>, Option<EffectiveCredential>), OpsError> {
    let secrets = secret_status(doc, vault, node.id)?
        .into_iter()
        .map(SecretView::from)
        .collect();
    let effective = match &node.kind {
        NodeKind::Unsupported(_) => None,
        _ => Some(match resolved_credential(doc, node.id, vault) {
            Ok(credential) => EffectiveCredential::Resolved { credential },
            Err(OpsError::Core(error)) => EffectiveCredential::Unresolved {
                reason: error.to_string(),
            },
            Err(error) => return Err(error),
        }),
    };
    Ok((secrets, effective))
}

fn property_view(property: &portolano_core::CustomProperty) -> PropertyView {
    match &property.value {
        PropertyValue::Text { value } => PropertyView {
            name: property.name.clone(),
            text: Some(value.clone()),
            secret: false,
        },
        PropertyValue::Secret { .. } => PropertyView {
            name: property.name.clone(),
            text: None,
            secret: true,
        },
    }
}

fn connection_view(node: &Node, protocols: &Registry) -> Option<ConnectionView> {
    let connection = node.as_connection()?;
    Some(ConnectionView {
        protocol: connection.protocol.clone(),
        host: connection.host.clone(),
        port: connection.port,
        default_port: protocols
            .get(&connection.protocol)
            .map(|protocol| protocol.default_port()),
        settings: connection.settings.clone(),
    })
}

fn own_spec(node: &Node) -> Option<SpecView> {
    match &node.kind {
        NodeKind::Folder(folder) => Some(SpecView::of(&folder.credential)),
        NodeKind::Connection(connection) => Some(SpecView::of(&connection.credential)),
        NodeKind::Credential(_) | NodeKind::Unsupported(_) => None,
    }
}

fn credential_fields(node: &Node) -> Option<CredentialFields> {
    let credential = node.as_credential()?;
    Some(CredentialFields {
        username: credential.username.clone(),
        domain: credential.domain.clone(),
        key_file: match &credential.private_key {
            Some(PrivateKey::File { path }) => Some(path.clone()),
            Some(PrivateKey::Vault { .. }) | None => None,
        },
    })
}
