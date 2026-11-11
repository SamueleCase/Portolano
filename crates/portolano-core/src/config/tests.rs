use pretty_assertions::assert_eq;
use tempfile::TempDir;

use super::*;

/// The example of SPEC §9.
const SPEC_EXAMPLE: &str = r#"
default_document = "/Users/me/infra/portolano.json"
[vault]
backend = "keychain"          # keychain | file
file = "/Users/me/vault.json"
[terminal]
app = "terminal"              # terminal | iterm | ghostty | custom
command = ""                  # per custom, con {cmd}
[ssh]
program = "ssh"
[rdp]
client = "auto"               # auto | freerdp | windows-app
freerdp_program = ""          # override del percorso
cert = "tofu"                 # tofu | ignore | deny
"#;

fn config_path(dir: &TempDir) -> PathBuf {
    dir.path().join(CONFIG_FILE_NAME)
}

fn load_text(dir: &TempDir, text: &str) -> Result<AppConfig> {
    fs::write(config_path(dir), text).unwrap();
    AppConfig::load(&config_path(dir))
}

/// The message of the TOML error inside an `InvalidConfig`.
fn invalid_config_reason(error: Error) -> String {
    match error {
        Error::InvalidConfig { source, .. } => source.to_string(),
        other => panic!("expected InvalidConfig, got {other:?}"),
    }
}

#[test]
fn a_missing_file_gives_the_defaults() {
    let dir = TempDir::new().unwrap();

    let config = AppConfig::load(&config_path(&dir)).unwrap();

    assert_eq!(config, AppConfig::default());
    assert_eq!(config.vault.backend, VaultBackend::Keychain);
    assert_eq!(config.terminal.app, TerminalApp::Terminal);
}

#[test]
fn an_empty_file_gives_the_defaults() {
    let dir = TempDir::new().unwrap();

    let config = load_text(&dir, "").unwrap();

    assert_eq!(config, AppConfig::default());
}

#[test]
fn values_in_the_file_override_the_defaults_and_missing_keys_keep_them() {
    let dir = TempDir::new().unwrap();

    let config = load_text(&dir, "[vault]\nfile = \"/srv/vault.json\"\n").unwrap();

    assert_eq!(config.vault.file, Some(PathBuf::from("/srv/vault.json")));
    assert_eq!(config.vault.backend, VaultBackend::Keychain);
    assert_eq!(config.terminal, TerminalConfig::default());
    assert_eq!(config.default_document, None);
}

#[test]
fn the_spec_example_fills_typed_and_protocol_sections() {
    let dir = TempDir::new().unwrap();

    let config = load_text(&dir, SPEC_EXAMPLE).unwrap();

    assert_eq!(
        config.default_document,
        Some(PathBuf::from("/Users/me/infra/portolano.json"))
    );
    assert_eq!(config.vault.backend, VaultBackend::Keychain);
    assert_eq!(config.terminal.command.as_deref(), Some(""));
    assert_eq!(config.protocols.keys().collect::<Vec<_>>(), ["rdp", "ssh"]);
    assert_eq!(config.protocols["rdp"]["cert"].as_str(), Some("tofu"));
}

#[test]
fn the_spec_example_round_trips_through_save() {
    let dir = TempDir::new().unwrap();
    let original = load_text(&dir, SPEC_EXAMPLE).unwrap();
    let copy = dir.path().join("copy.toml");

    original.save(&copy).unwrap();

    assert_eq!(AppConfig::load(&copy).unwrap(), original);
}

#[test]
fn save_creates_the_missing_directory() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("nested").join(CONFIG_FILE_NAME);

    AppConfig::default().save(&path).unwrap();

    assert!(path.is_file());
}

#[test]
fn an_unknown_top_level_scalar_is_rejected_with_its_name() {
    let dir = TempDir::new().unwrap();

    let error = load_text(&dir, "default_documnt = \"/srv/portolano.json\"\n").unwrap_err();

    assert!(invalid_config_reason(error).contains("'default_documnt'"));
}

#[test]
fn an_unknown_key_inside_a_typed_section_is_rejected() {
    let dir = TempDir::new().unwrap();

    let error = load_text(&dir, "[vault]\nbakend = \"file\"\n").unwrap_err();

    assert!(invalid_config_reason(error).contains("bakend"));
}

#[test]
fn an_unknown_value_of_a_typed_key_is_rejected() {
    let dir = TempDir::new().unwrap();

    let error = load_text(&dir, "[terminal]\napp = \"konsole\"\n").unwrap_err();

    assert!(matches!(error, Error::InvalidConfig { .. }));
}

#[test]
fn malformed_toml_is_rejected() {
    let dir = TempDir::new().unwrap();

    let error = load_text(&dir, "[vault\n").unwrap_err();

    assert!(matches!(error, Error::InvalidConfig { .. }));
}

#[cfg(target_os = "macos")]
#[test]
fn data_dir_is_in_application_support_on_macos() {
    let dir = data_dir().unwrap();

    assert!(dir.ends_with("Library/Application Support/dev.portolano.Portolano"));
}
