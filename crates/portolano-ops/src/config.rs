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
    let value = parse_value(value);
    update(config, key, protocols, |table, field| {
        table.insert(field.to_owned(), value);
    })
}

/// `config` without `key`, which takes its default again (SPEC-UI §5.4,
/// the preferences of the app); the keys are those of
/// [`set_config_value`].
///
/// # Errors
/// As [`set_config_value`].
pub fn unset_config_value(
    config: &AppConfig,
    key: &str,
    protocols: &Registry,
) -> Result<AppConfig> {
    update(config, key, protocols, |table, field| {
        table.remove(field);
    })
}

/// `config` with `change` applied to the table that holds `key` and the
/// field name, then checked by the core and, for a protocol section, by the
/// protocol.
fn update(
    config: &AppConfig,
    key: &str,
    protocols: &Registry,
    change: impl FnOnce(&mut toml::Table, &str),
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
    match key.split_once('.') {
        None if key == DEFAULT_DOCUMENT => change(&mut table, key),
        Some((section, field))
            if !field.is_empty()
                && !field.contains('.')
                && (CORE_SECTIONS.contains(&section) || protocols.get(section).is_some()) =>
        {
            let entry = table
                .entry(section.to_owned())
                .or_insert_with(|| Value::Table(toml::Table::new()));
            change(entry.as_table_mut().ok_or_else(unknown)?, field);
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
    fn unset_key_takes_its_default_again() {
        let config = set(&AppConfig::default(), "terminal.app", "iterm").unwrap();

        let config =
            unset_config_value(&config, "terminal.app", &Registry::with_defaults()).unwrap();

        assert_eq!(config.terminal.app, TerminalApp::default());
    }

    #[test]
    fn unset_protocol_field_leaves_the_rest_of_the_section() {
        let config = set(&AppConfig::default(), "rdp.client", "freerdp").unwrap();
        let config = set(&config, "rdp.cert", "deny").unwrap();

        let config = unset_config_value(&config, "rdp.client", &Registry::with_defaults()).unwrap();

        let rdp = config.protocols.get("rdp").unwrap();
        assert_eq!(rdp.get("client"), None);
        assert_eq!(rdp.get("cert"), Some(&Value::String("deny".to_owned())));
    }

    #[test]
    fn unset_of_an_unknown_key_is_refused() {
        let result =
            unset_config_value(&AppConfig::default(), "colore", &Registry::with_defaults());

        assert!(matches!(result, Err(OpsError::UnknownConfigKey { .. })));
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
