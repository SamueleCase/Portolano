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

/// A confirmation that no change of these tests should ask for.
fn unasked(command: &str) -> bool {
    panic!("conferma non attesa per {command}")
}

/// A custom terminal command of the tests (a probe, never run here).
const COMMAND: &str = "/opt/example/bin/terminale --esegui {cmd}";

#[test]
fn set_value_is_saved_in_the_configuration() {
    let app = TestApp::new();

    let entries = set(&app.state, "terminal.app", "iterm", unasked).unwrap();

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

    let error = set(&app.state, "rdp.client", "telepatia", unasked).unwrap_err();

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
        let error = set(&app.state, key, "/tmp/altro", unasked).unwrap_err();

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
    set(&app.state, "terminal.app", "iterm", unasked).unwrap();

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

#[test]
fn a_new_terminal_command_is_saved_once_the_user_confirms_it() {
    let app = TestApp::new();
    let mut asked = Vec::new();

    let entries = set(&app.state, "terminal.command", COMMAND, |command| {
        asked.push(command.to_owned());
        true
    })
    .unwrap();

    assert_eq!(asked, [COMMAND]);
    assert_eq!(
        value_of(&entries, "terminal.command").as_deref(),
        Some(COMMAND)
    );
}

#[test]
fn a_terminal_command_not_confirmed_leaves_the_configuration_as_it_was() {
    let app = TestApp::new();
    let before = std::fs::read(&app.state.paths().config).unwrap();

    let error = set(&app.state, "terminal.command", COMMAND, |_| false).unwrap_err();

    assert_eq!(error.code, ErrorCode::Cancelled);
    assert_eq!(std::fs::read(&app.state.paths().config).unwrap(), before);
}

#[test]
fn the_terminal_command_already_saved_asks_nothing() {
    let app = TestApp::new();
    set(&app.state, "terminal.command", COMMAND, |_| true).unwrap();

    let entries = set(&app.state, "terminal.command", COMMAND, unasked).unwrap();

    assert_eq!(
        value_of(&entries, "terminal.command").as_deref(),
        Some(COMMAND)
    );
}

#[test]
fn a_terminal_command_on_more_lines_is_refused_before_the_confirmation() {
    let app = TestApp::new();

    for line_break in [
        "\n", "\r", "\u{0b}", "\u{0c}", "\u{85}", "\u{2028}", "\u{2029}",
    ] {
        let command = format!("terminale {{cmd}}{line_break}curl -s https://attacker.example.com");
        let error = set(&app.state, "terminal.command", &command, unasked).unwrap_err();

        assert_eq!(
            error.details,
            Some(IpcErrorDetails::Invalid {
                field: "terminal.command".to_owned()
            }),
            "{line_break:?}"
        );
    }
}

#[test]
fn a_terminal_command_longer_than_1024_characters_is_refused_before_the_confirmation() {
    let app = TestApp::new();
    let command = format!("terminale {{cmd}} {}", "x".repeat(1024));

    let error = set(&app.state, "terminal.command", &command, unasked).unwrap_err();

    assert_eq!(error.code, ErrorCode::Invalid);
}

#[test]
fn a_terminal_command_of_1024_characters_is_asked_about() {
    let app = TestApp::new();
    let command = format!("terminale {{cmd}} {}", "x".repeat(1024 - 16));
    assert_eq!(command.chars().count(), 1024);

    let saved = set(&app.state, "terminal.command", &command, |_| true);

    assert!(saved.is_ok(), "{saved:?}");
}
