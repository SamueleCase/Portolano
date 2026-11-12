//! The preferences of the interface (SPEC-UI §4.1), in `ui.toml` in the
//! data directory: not in `config.toml`, which the CLI shares and validates
//! strictly (DEC-011). The theme is fixed (ambiguity 19), so it is not a
//! preference.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::IpcError;

/// File name of the preferences inside the data directory.
pub const UI_PREFERENCES_FILE_NAME: &str = "ui.toml";

/// How many recent documents are remembered.
const MAX_RECENT_DOCUMENTS: usize = 10;

/// Row height and text size of the lists (ambiguity 8).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum Density {
    /// The size of the mock.
    #[default]
    Normal,
    /// Rows of 26 px and text of 12.5 px.
    Compact,
}

/// The animated background (ambiguity 19).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum Background {
    /// Animated.
    #[default]
    On,
    /// Slower and fainter.
    Reduced,
    /// Still.
    Off,
}

/// The preferences the frontend reads and changes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", default)]
pub struct InterfacePreferences {
    /// Density of the lists.
    pub density: Density,
    /// Animated background.
    pub background: Background,
    /// Reduce motion, besides the setting of the system.
    pub reduce_motion: bool,
}

/// Size and position of the main window, in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WindowGeometry {
    /// Width.
    pub width: f64,
    /// Height.
    pub height: f64,
    /// Left edge.
    pub x: f64,
    /// Top edge.
    pub y: f64,
}

/// Everything `ui.toml` holds.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiPreferences {
    /// What the frontend shows and changes.
    #[serde(flatten)]
    pub interface: InterfacePreferences,
    /// The documents opened last, most recent first.
    pub recent_documents: Vec<PathBuf>,
    /// The main window when the app last closed.
    pub window: Option<WindowGeometry>,
}

impl UiPreferences {
    /// The preferences in `path`; the defaults if the file does not exist.
    ///
    /// # Errors
    /// The file cannot be read or is not valid.
    pub fn load(path: &Path) -> Result<UiPreferences, IpcError> {
        match fs::read_to_string(path) {
            Ok(text) => {
                toml::from_str(&text).map_err(|error| IpcError::invalid_file(path, error.message()))
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(UiPreferences::default()),
            Err(error) => Err(IpcError::io(path, &error)),
        }
    }

    /// Writes the preferences to `path`, replacing the file in one step.
    ///
    /// # Errors
    /// The file cannot be written.
    pub fn save(&self, path: &Path) -> Result<(), IpcError> {
        let text = toml::to_string(self).map_err(|error| {
            IpcError::internal(&format!("preferenze non serializzabili: {error}"))
        })?;
        let temporary = path.with_extension("toml.tmp");
        let write = || -> io::Result<()> {
            if let Some(dir) = path.parent() {
                fs::create_dir_all(dir)?;
            }
            fs::write(&temporary, text)?;
            fs::rename(&temporary, path)
        };
        write().map_err(|error| IpcError::io(path, &error))
    }

    /// Puts `document` first among the recent documents.
    pub fn remember_document(&mut self, document: &Path) {
        self.recent_documents.retain(|recent| recent != document);
        self.recent_documents.insert(0, document.to_owned());
        self.recent_documents.truncate(MAX_RECENT_DOCUMENTS);
    }

    /// The document opened last, if any.
    pub fn last_document(&self) -> Option<&Path> {
        self.recent_documents.first().map(PathBuf::as_path)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn missing_file_gives_the_defaults() {
        let dir = tempfile::tempdir().unwrap();

        let preferences = UiPreferences::load(&dir.path().join("ui.toml")).unwrap();

        assert_eq!(preferences, UiPreferences::default());
    }

    #[test]
    fn saved_preferences_load_back() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ui.toml");
        let mut preferences = UiPreferences::default();
        preferences.interface.density = Density::Compact;
        preferences.interface.background = Background::Off;
        preferences.remember_document(Path::new("/Users/utente-di-prova/infra.json"));
        preferences.window = Some(WindowGeometry {
            width: 1280.0,
            height: 800.0,
            x: 10.0,
            y: 20.0,
        });

        preferences.save(&path).unwrap();

        assert_eq!(UiPreferences::load(&path).unwrap(), preferences);
    }

    #[test]
    fn reopened_document_moves_first_without_duplicates() {
        let mut preferences = UiPreferences::default();
        preferences.remember_document(Path::new("/a.json"));
        preferences.remember_document(Path::new("/b.json"));

        preferences.remember_document(Path::new("/a.json"));

        assert_eq!(
            preferences.recent_documents,
            vec![PathBuf::from("/a.json"), PathBuf::from("/b.json")]
        );
    }

    #[test]
    fn only_the_last_ten_documents_are_remembered() {
        let mut preferences = UiPreferences::default();

        for index in 0..12 {
            preferences.remember_document(Path::new(&format!("/doc{index}.json")));
        }

        assert_eq!(preferences.recent_documents.len(), MAX_RECENT_DOCUMENTS);
        assert_eq!(preferences.last_document(), Some(Path::new("/doc11.json")));
    }

    #[test]
    fn invalid_file_is_an_invalid_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ui.toml");
        fs::write(&path, "density = 3").unwrap();

        let error = UiPreferences::load(&path).unwrap_err();

        assert_eq!(error.code, crate::error::ErrorCode::Invalid);
    }
}
