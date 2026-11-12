//! Commands about the document.

use std::path::Path;

use portolano_core::AppConfig;
use portolano_ops::OpsError;
use portolano_ops::locations::{GlobalOptions, Locations};
use tauri::State;

use crate::dto::{AppStatus, DocumentLocation, VaultKind};
use crate::error::IpcError;
use crate::state::AppState;

/// The default document and the vault backend of the configuration.
#[tauri::command]
#[specta::specta]
pub async fn app_status(state: State<'_, AppState>) -> Result<AppStatus, IpcError> {
    status(state.config_path())
}

/// The status of an app configured by the file at `config_path`, which may
/// be missing (defaults apply).
fn status(config_path: &Path) -> Result<AppStatus, IpcError> {
    let config = AppConfig::load(config_path).map_err(OpsError::from)?;
    let locations = Locations::resolve(&GlobalOptions::default(), &config)?;
    Ok(AppStatus {
        default_document: DocumentLocation {
            path: locations.document.display().to_string(),
            exists: locations.document.is_file(),
        },
        vault: VaultKind::from(&locations.vault),
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use pretty_assertions::assert_eq;
    use tempfile::TempDir;

    use super::*;
    use crate::error::ErrorCode;

    fn config_with(dir: &TempDir, text: &str) -> std::path::PathBuf {
        let path = dir.path().join("config.toml");
        fs::write(&path, text).unwrap();
        path
    }

    #[test]
    fn status_reports_the_configured_document_and_vault() {
        let dir = TempDir::new().unwrap();
        let document = dir.path().join("portolano.json");
        fs::write(&document, "{}").unwrap();
        let config = config_with(
            &dir,
            &format!(
                "default_document = {:?}\n[vault]\nbackend = \"file\"\n",
                document.display().to_string()
            ),
        );

        let status = status(&config).unwrap();

        assert_eq!(
            status,
            AppStatus {
                default_document: DocumentLocation {
                    path: document.display().to_string(),
                    exists: true,
                },
                vault: VaultKind::File,
            }
        );
    }

    #[test]
    fn missing_document_is_reported_as_absent() {
        let dir = TempDir::new().unwrap();
        let document = dir.path().join("assente.json");
        let config = config_with(
            &dir,
            &format!("default_document = {:?}\n", document.display().to_string()),
        );

        let status = status(&config).unwrap();

        assert!(!status.default_document.exists);
        assert_eq!(status.vault, VaultKind::Keychain);
    }

    #[test]
    fn invalid_configuration_is_an_invalid_error() {
        let dir = TempDir::new().unwrap();
        let config = config_with(&dir, "default_documnt = \"refuso\"\n");

        let error = status(&config).unwrap_err();

        assert_eq!(error.code, ErrorCode::Invalid);
        assert!(
            error.message.contains("default_documnt"),
            "{}",
            error.message
        );
    }
}
