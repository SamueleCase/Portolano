//! `config set` (SPEC §9, DEC-003, DEC-008, DEC-011): changes one key of
//! `config.toml`, validating the core sections with the core and each
//! protocol section with its protocol.

use portolano_connectors::Registry;
use portolano_core::AppConfig;
use toml::Value;

use crate::{OpsError, Result};

/// The top-level key that is not a section.
const DEFAULT_DOCUMENT: &str = "default_document";

/// Sections typed by the core.
const CORE_SECTIONS: [&str; 2] = ["vault", "terminal"];

/// `config` with `key` set to `value` and checked.
///
/// `key` is `default_document` or `<section>.<field>`, where the section is
/// `vault`, `terminal` or the id of a protocol (`ssh.program`,
/// `rdp.client`). `value` is read as a TOML literal (`true`, `3`, `"x"`)
/// and, if it is not one, taken as a string.
///
/// # Errors
/// [`OpsError::UnknownConfigKey`] for any other key,
/// [`OpsError::InvalidConfigValue`] for a value that the core or the
/// protocol rejects.
pub fn set_config_value(
    config: &AppConfig,
    key: &str,
    value: &str,
    protocols: &Registry,
) -> Result<AppConfig> {
    let unknown = || OpsError::UnknownConfigKey {
        key: key.to_owned(),
    };
    let invalid = |reason: String| OpsError::InvalidConfigValue {
        key: key.to_owned(),
        reason,
    };
    let mut table = Value::try_from(config)
        .map_err(|error| invalid(error.to_string()))?
        .as_table()
        .cloned()
        .ok_or_else(unknown)?;
    let value = parse_value(value);
    match key.split_once('.') {
        None if key == DEFAULT_DOCUMENT => {
            table.insert(key.to_owned(), value);
        }
        Some((section, field))
            if !field.is_empty()
                && !field.contains('.')
                && (CORE_SECTIONS.contains(&section) || protocols.get(section).is_some()) =>
        {
            let entry = table
                .entry(section.to_owned())
                .or_insert_with(|| Value::Table(toml::Table::new()));
            entry
                .as_table_mut()
                .ok_or_else(unknown)?
                .insert(field.to_owned(), value);
            if let Some(protocol) = protocols.get(section) {
                protocol.validate_config(entry).map_err(invalid)?;
            }
        }
        _ => return Err(unknown()),
    }
    Value::Table(table)
        .try_into()
        .map_err(|error: toml::de::Error| invalid(error.message().to_owned()))
}

/// A TOML literal, or the text itself as a string.
fn parse_value(text: &str) -> Value {
    format!("value = {text}")
        .parse::<toml::Table>()
        .ok()
        .and_then(|mut table| table.remove("value"))
        .unwrap_or_else(|| Value::String(text.to_owned()))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use portolano_core::config::{TerminalApp, VaultBackend};
    use pretty_assertions::assert_eq;

    use super::*;

    fn set(config: &AppConfig, key: &str, value: &str) -> Result<AppConfig> {
        set_config_value(config, key, value, &Registry::with_defaults())
    }

    #[test]
    fn default_document_is_set_as_a_path() {
        let config = set(&AppConfig::default(), "default_document", "/infra/doc.json").unwrap();
        assert_eq!(
            config.default_document,
            Some(PathBuf::from("/infra/doc.json"))
        );
    }

    #[test]
    fn core_section_values_are_typed() {
        let config = set(&AppConfig::default(), "vault.backend", "file").unwrap();
        let config = set(&config, "terminal.app", "iterm").unwrap();
        assert_eq!(config.vault.backend, VaultBackend::File);
        assert_eq!(config.terminal.app, TerminalApp::Iterm);
    }

    #[test]
    fn invalid_core_value_is_refused() {
        let result = set(&AppConfig::default(), "vault.backend", "cassaforte");
        assert!(matches!(result, Err(OpsError::InvalidConfigValue { .. })));
    }

    #[test]
    fn unknown_core_field_is_refused() {
        let result = set(&AppConfig::default(), "vault.colore", "blu");
        assert!(matches!(result, Err(OpsError::InvalidConfigValue { .. })));
    }

    #[test]
    fn protocol_section_is_validated_by_the_protocol() {
        let config = set(&AppConfig::default(), "rdp.client", "freerdp").unwrap();
        assert_eq!(
            config
                .protocols
                .get("rdp")
                .and_then(|rdp| rdp.get("client")),
            Some(&Value::String("freerdp".to_owned()))
        );
        let result = set(&config, "rdp.client", "telepatia");
        assert!(matches!(result, Err(OpsError::InvalidConfigValue { .. })));
    }

    #[test]
    fn unknown_sections_and_keys_are_refused() {
        for key in ["vnc.program", "colore", "ssh", "ssh.", "ssh.program.extra"] {
            let result = set(&AppConfig::default(), key, "x");
            assert!(
                matches!(result, Err(OpsError::UnknownConfigKey { .. })),
                "{key}"
            );
        }
    }

    #[test]
    fn literals_are_parsed_and_other_text_is_a_string() {
        assert_eq!(parse_value("true"), Value::Boolean(true));
        assert_eq!(parse_value("\"ssh\""), Value::String("ssh".to_owned()));
        assert_eq!(
            parse_value("/opt/homebrew/bin/sdl-freerdp"),
            Value::String("/opt/homebrew/bin/sdl-freerdp".to_owned())
        );
    }
}
