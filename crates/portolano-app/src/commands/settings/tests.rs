use std::path::Path;

use pretty_assertions::assert_eq;

use super::*;
use crate::commands::test_support::TestApp;
use crate::error::{ErrorCode, IpcErrorDetails};
use crate::prefs::{Density, UiPreferences};

fn value_of(entries: &[ConfigEntry], key: &str) -> Option<String> {
    entries
        .iter()
        .find(|entry| entry.key == key)
        .map(|entry| entry.value.clone())
}

#[test]
fn set_value_is_saved_in_the_configuration() {
    let app = TestApp::new();

    let entries = set(&app.state, "terminal.app", "iterm").unwrap();

    assert_eq!(value_of(&entries, "terminal.app").as_deref(), Some("iterm"));
    let saved = portolano_core::AppConfig::load(&app.state.paths().config).unwrap();
    assert_eq!(
        saved.terminal.app,
        portolano_core::config::TerminalApp::Iterm
    );
}

#[test]
fn value_refused_by_the_protocol_names_its_key() {
    let app = TestApp::new();

    let error = set(&app.state, "rdp.client", "telepatia").unwrap_err();

    assert_eq!(error.code, ErrorCode::Invalid);
    assert_eq!(
        error.details,
        Some(IpcErrorDetails::Invalid {
            field: "rdp.client".to_owned()
        })
    );
}

#[test]
fn file_keys_cannot_be_typed() {
    let app = TestApp::new();

    for key in FILE_KEYS {
        let error = set(&app.state, key, "/tmp/altro").unwrap_err();

        assert_eq!(error.code, ErrorCode::Invalid, "{key}");
    }
}

#[test]
fn chosen_file_is_set_as_a_string() {
    let app = TestApp::new();

    let entries = set_file(&app.state, "ssh.program", Path::new("/usr/bin/ssh")).unwrap();

    assert_eq!(
        value_of(&entries, "ssh.program").as_deref(),
        Some("/usr/bin/ssh")
    );
}

#[test]
fn reset_key_takes_its_default_again() {
    let app = TestApp::new();
    set(&app.state, "terminal.app", "iterm").unwrap();

    let entries = update(&app.state, |config| {
        Ok(unset_config_value(
            config,
            "terminal.app",
            app.state.protocols(),
        )?)
    })
    .unwrap();

    assert_eq!(
        value_of(&entries, "terminal.app").as_deref(),
        Some("terminal")
    );
}

#[test]
fn interface_preferences_are_saved_in_their_own_file() {
    let app = TestApp::new();

    app.state
        .update_ui_preferences(|preferences| preferences.interface.density = Density::Compact)
        .unwrap();

    let saved = UiPreferences::load(&app.state.paths().ui_preferences).unwrap();
    assert_eq!(saved.interface.density, Density::Compact);
    let config = std::fs::read_to_string(&app.state.paths().config).unwrap();
    assert!(!config.contains("density"));
}
