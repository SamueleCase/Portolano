//! The events the backend sends to the frontend (SPEC-UI §4.4): small and
//! rare, typed in the bindings. `session-ended` belongs to the sessions.

use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager, Runtime};
use tauri_specta::Event;

use crate::dto::VaultKind;

/// The app saved the document: the frontend reloads what it shows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct DocumentChanged {
    /// The file name of the document.
    pub file_name: String,
}

/// Another process changed the document file: the frontend offers
/// "Ricarica".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct DocumentModifiedExternally {
    /// The file name of the document.
    pub file_name: String,
}

/// The document file was moved or deleted: the frontend offers "Ricrea" or
/// "Apri altro".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct DocumentMissing {
    /// The file name of the document.
    pub file_name: String,
}

/// The vault was locked: secrets show as "Vault bloccato" until it is
/// unlocked again.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct VaultLocked {
    /// The backend of the vault.
    pub kind: VaultKind,
}

/// Emits `event` to the frontend; a failure only means that no window is
/// listening, so it is logged.
pub fn emit<R: Runtime, E: Event + Serialize + Clone>(
    app: &(impl Emitter<R> + Manager<R>),
    event: E,
) {
    if let Err(error) = event.emit(app) {
        tracing::warn!(event = E::NAME, %error, "evento non inviato");
    }
}
