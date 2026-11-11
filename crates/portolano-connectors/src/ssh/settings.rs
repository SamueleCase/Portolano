//! Typed SSH settings of a connection and the `[ssh]` configuration section.

use std::collections::BTreeMap;
use std::path::PathBuf;

use portolano_core::AppConfig;
use portolano_core::model::is_false;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::ConnectError;
use crate::decode::{decode_config_section, decode_settings};

/// The `settings` of an `ssh` connection. Fields at their default are
/// omitted when serialized.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SshSettings {
    /// Jump hosts, in order from the first hop to the last.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub jump_hosts: Vec<JumpHost>,
    /// Agent forwarding (`-A`).
    #[serde(skip_serializing_if = "is_false")]
    pub agent_forwarding: bool,
    /// X11 forwarding (`-X`).
    #[serde(skip_serializing_if = "is_false")]
    pub x11: bool,
    /// Trusted X11 forwarding (`-Y`).
    #[serde(skip_serializing_if = "is_false")]
    pub x11_trusted: bool,
    /// Compression (`-C`).
    #[serde(skip_serializing_if = "is_false")]
    pub compression: bool,
    /// `ServerAliveInterval`, in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keepalive_s: Option<u32>,
    /// Extra options, passed as `-o Key=Value`.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub options: BTreeMap<String, String>,
}

impl SshSettings {
    /// Decodes the `settings` of an `ssh` connection.
    ///
    /// # Errors
    /// A description of the first problem, in Italian.
    pub(crate) fn parse(settings: &Map<String, Value>) -> Result<SshSettings, String> {
        // `JumpHost` is untagged, so serde can only say that "no variant
        // matched": point at the faulty entry and show the two accepted forms.
        if let Some(Value::Array(jumps)) = settings.get("jump_hosts") {
            for (index, jump) in jumps.iter().enumerate() {
                if serde_json::from_value::<JumpHost>(jump.clone()).is_err() {
                    return Err(format!(
                        "campo 'jump_hosts', elemento {index}: atteso {{\"$ref\": \"<uuid>\"}} \
                         oppure {{\"address\": \"[utente@]host[:porta]\"}}"
                    ));
                }
            }
        }
        let parsed: SshSettings = decode_settings(settings)?;
        for (key, value) in &parsed.options {
            check_option(key, value)?;
        }
        Ok(parsed)
    }
}

/// Options that run commands, change the environment, the agent or the
/// multiplexing socket, read other configuration, or replace a dedicated
/// setting (DEC-024.1). A document may come from someone else, so it is not
/// trusted like `~/.ssh/config`, where whoever needs them can put them.
const FORBIDDEN_OPTIONS: [&str; 17] = [
    "ProxyCommand",
    "ProxyJump",
    "LocalCommand",
    "PermitLocalCommand",
    "KnownHostsCommand",
    "PKCS11Provider",
    "SecurityKeyProvider",
    "ProxyUseFdpass",
    "SendEnv",
    "SetEnv",
    "Include",
    "Match",
    "IdentityAgent",
    "ControlPath",
    "ControlMaster",
    "RemoteCommand",
    "ForwardAgent",
];

/// Checks one `-o Key=Value` option.
///
/// The name must be a plain keyword: ssh reads `-o` like a line of its
/// configuration, so a space or a `=` in the name would let the rest of it
/// become the value of another keyword, and a control character in the value
/// could end the line.
fn check_option(key: &str, value: &str) -> Result<(), String> {
    if key.is_empty() || !key.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(format!(
            "campo 'options': nome di opzione non valido '{key}'"
        ));
    }
    if value.chars().any(char::is_control) {
        return Err(format!(
            "campo 'options': il valore di '{key}' contiene un carattere di controllo"
        ));
    }
    let Some(forbidden) = FORBIDDEN_OPTIONS
        .iter()
        .find(|name| name.eq_ignore_ascii_case(key))
    else {
        return Ok(());
    };
    let alternative = match *forbidden {
        "ProxyJump" => " (per i jump host usa 'jump_hosts')",
        "ForwardAgent" => " (per l'inoltro dell'agent usa 'agent_forwarding')",
        _ => "",
    };
    Err(format!(
        "campo 'options': l'opzione '{key}' non è ammessa nel documento perché può eseguire \
         comandi o cambiare ambiente, agent o configurazione{alternative}; se ti serve, \
         mettila nel tuo ~/.ssh/config"
    ))
}

/// A jump host: another SSH connection of the document, or a literal address.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum JumpHost {
    /// An SSH connection node, serialized as `{"$ref": "<uuid>"}`.
    Ref {
        /// The connection node.
        #[serde(rename = "$ref")]
        id: Uuid,
    },
    /// An address in the form `[user@]host[:port]`.
    Address {
        /// The address.
        address: String,
    },
}

/// The `[ssh]` section of `config.toml` (DEC-003).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SshConfig {
    /// The `ssh` program: a name looked up in the `PATH`, or a path.
    pub program: PathBuf,
}

impl Default for SshConfig {
    fn default() -> Self {
        Self {
            program: PathBuf::from("ssh"),
        }
    }
}

impl SshConfig {
    /// Reads the `[ssh]` section of `config`; defaults if it is missing.
    ///
    /// # Errors
    /// [`ConnectError::InvalidConfig`] if the section does not match.
    pub fn from_app_config(config: &AppConfig) -> Result<SshConfig, ConnectError> {
        decode_config_section(config, "ssh")
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    fn map(value: Value) -> Map<String, Value> {
        match value {
            Value::Object(map) => map,
            other => panic!("not an object: {other}"),
        }
    }

    #[test]
    fn default_settings_serialize_to_an_empty_object() {
        let json = serde_json::to_value(SshSettings::default()).unwrap();

        assert_eq!(json, json!({}));
    }

    #[test]
    fn settings_round_trip_through_json() {
        let id = Uuid::new_v4();
        let settings = SshSettings {
            jump_hosts: vec![
                JumpHost::Ref { id },
                JumpHost::Address {
                    address: "ops@bastion.example.com:2222".into(),
                },
            ],
            agent_forwarding: true,
            keepalive_s: Some(30),
            options: BTreeMap::from([("StrictHostKeyChecking".into(), "yes".into())]),
            ..SshSettings::default()
        };

        let json = serde_json::to_value(&settings).unwrap();
        let back = SshSettings::parse(&map(json.clone())).unwrap();

        assert_eq!(json["jump_hosts"][0], json!({ "$ref": id.to_string() }));
        assert_eq!(back, settings);
    }

    #[test]
    fn unknown_field_is_rejected_in_italian() {
        let error = SshSettings::parse(&map(json!({ "agent_forwading": true }))).unwrap_err();

        assert!(
            error.starts_with("campo sconosciuto 'agent_forwading'"),
            "{error}"
        );
    }

    #[test]
    fn wrong_type_names_the_field() {
        let error = SshSettings::parse(&map(json!({ "compression": "yes" }))).unwrap_err();

        assert!(
            error.starts_with("campo 'compression': tipo non valido"),
            "{error}"
        );
    }

    #[test]
    fn invalid_jump_host_points_at_the_entry() {
        let settings =
            map(json!({ "jump_hosts": [{ "address": "a.example.com" }, { "host": "b" }] }));

        let error = SshSettings::parse(&settings).unwrap_err();

        assert!(error.contains("elemento 1"), "{error}");
        assert!(error.contains("$ref"), "{error}");
    }

    #[test]
    fn jump_host_with_both_forms_is_rejected() {
        let jump = json!({ "$ref": Uuid::new_v4().to_string(), "address": "a.example.com" });

        let result = serde_json::from_value::<JumpHost>(jump);

        assert!(result.is_err());
    }

    #[test]
    fn jump_host_ref_must_be_a_uuid() {
        let result = serde_json::from_value::<JumpHost>(json!({ "$ref": "not-a-uuid" }));

        assert!(result.is_err());
    }

    #[test]
    fn option_names_with_equals_are_rejected() {
        let error = SshSettings::parse(&map(json!({ "options": { "A=B": "C" } }))).unwrap_err();

        assert!(error.contains("'A=B'"), "{error}");
    }

    #[test]
    fn option_names_with_spaces_are_rejected() {
        let options = json!({ "options": { "User ProxyCommand": "x" } });

        let error = SshSettings::parse(&map(options)).unwrap_err();

        assert!(error.contains("nome di opzione non valido"), "{error}");
    }

    #[test]
    fn option_values_with_control_characters_are_rejected() {
        let options = json!({ "options": { "User": "admin\nProxyCommand x" } });

        let error = SshSettings::parse(&map(options)).unwrap_err();

        assert!(error.contains("carattere di controllo"), "{error}");
    }

    #[test]
    fn every_dangerous_option_is_rejected_whatever_its_case() {
        for name in FORBIDDEN_OPTIONS {
            for key in [name.to_owned(), name.to_ascii_lowercase()] {
                let options = json!({ "options": { key.clone(): "x" } });

                let error = SshSettings::parse(&map(options)).unwrap_err();

                assert!(error.contains(&format!("'{key}'")), "{error}");
                assert!(error.contains("~/.ssh/config"), "{error}");
            }
        }
    }

    #[test]
    fn proxy_jump_points_at_jump_hosts() {
        let options = json!({ "options": { "ProxyJump": "bastion.example.com" } });

        let error = SshSettings::parse(&map(options)).unwrap_err();

        assert!(error.contains("jump_hosts"), "{error}");
    }

    #[test]
    fn harmless_options_are_accepted() {
        let options = json!({ "options": {
            "IdentityFile": "~/.ssh/id_ed25519",
            "IdentitiesOnly": "yes",
            "StrictHostKeyChecking": "yes",
        } });

        let parsed = SshSettings::parse(&map(options)).unwrap();

        assert_eq!(parsed.options.len(), 3);
    }

    #[test]
    fn missing_config_section_gives_the_default_program() {
        let config = SshConfig::from_app_config(&AppConfig::default()).unwrap();

        assert_eq!(config.program, PathBuf::from("ssh"));
    }

    #[test]
    fn unknown_config_key_is_an_invalid_config_error() {
        let mut app = AppConfig::default();
        app.protocols
            .insert("ssh".into(), toml::from_str("programm = \"ssh\"").unwrap());

        let error = SshConfig::from_app_config(&app).unwrap_err();

        assert!(
            matches!(error, ConnectError::InvalidConfig { ref section, .. } if section == "ssh"),
            "{error}"
        );
    }
}
