//! The configuration and the vault as the preferences show them.

use portolano_core::AppConfig;
use serde::Serialize;

use super::status::VaultKind;

/// One key of `config.toml` with its value, as `config set` names them
/// (`default_document`, `vault.backend`, `ssh.program`, …). Only the keys
/// present in the configuration are listed; the frontend shows the defaults
/// of the others.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
pub struct ConfigEntry {
    /// The key.
    pub key: String,
    /// The value: the text of a string, the TOML literal of anything else.
    pub value: String,
}

/// The keys and values of `config`, ordered by key.
pub fn config_entries(config: &AppConfig) -> Vec<ConfigEntry> {
    let Ok(toml::Value::Table(table)) = toml::Value::try_from(config) else {
        return Vec::new();
    };
    let mut entries = Vec::new();
    for (key, value) in table {
        match value {
            toml::Value::Table(section) => {
                entries.extend(section.into_iter().map(|(field, value)| ConfigEntry {
                    key: format!("{key}.{field}"),
                    value: text_of(value),
                }))
            }
            other => entries.push(ConfigEntry {
                key,
                value: text_of(other),
            }),
        }
    }
    entries.sort_by(|a, b| a.key.cmp(&b.key));
    entries
}

fn text_of(value: toml::Value) -> String {
    match value {
        toml::Value::String(text) => text,
        other => other.to_string(),
    }
}

/// The vault of the configuration and whether it is locked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultStatus {
    /// Its backend.
    pub kind: VaultKind,
    /// The file of the file vault, for display.
    pub path: Option<String>,
    /// Whether the file of the file vault exists; the keychain always does.
    pub exists: bool,
    /// Whether the file vault waits for its master password. The keychain
    /// is never locked by the app (ambiguity 20).
    pub locked: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries_name_the_keys_as_config_set_does() {
        let config: AppConfig = toml::from_str(
            "default_document = \"/Users/utente-di-prova/infra.json\"\n[rdp]\nclient = \"freerdp\"\n",
        )
        .unwrap();

        let entries = config_entries(&config);

        assert!(entries.contains(&ConfigEntry {
            key: "default_document".to_owned(),
            value: "/Users/utente-di-prova/infra.json".to_owned(),
        }));
        assert!(entries.contains(&ConfigEntry {
            key: "rdp.client".to_owned(),
            value: "freerdp".to_owned(),
        }));
        assert!(entries.contains(&ConfigEntry {
            key: "vault.backend".to_owned(),
            value: "keychain".to_owned(),
        }));
    }
}
