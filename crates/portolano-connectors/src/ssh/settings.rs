//! Typed SSH settings of a connection and the `[ssh]` configuration section.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use portolano_core::AppConfig;
use portolano_core::model::is_false;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use uuid::Uuid;

use super::options::check_option;
use crate::ConnectError;
use crate::decode::{decode_config_section, decode_settings};
use crate::env::{DEFAULT_SSH_PROGRAM, check_program_path};

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
    /// The `ssh` program: `ssh` (the default, looked up in the `PATH`), or
    /// an absolute path (DEC-024.2).
    pub program: PathBuf,
}

impl Default for SshConfig {
    fn default() -> Self {
        Self {
            program: PathBuf::from(DEFAULT_SSH_PROGRAM),
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

    /// The program set in the configuration; `None` for the default (`ssh`
    /// or an empty value), which is looked up in the `PATH`.
    pub(crate) fn configured_program(&self) -> Option<&Path> {
        let default =
            self.program.as_os_str().is_empty() || self.program.as_os_str() == DEFAULT_SSH_PROGRAM;
        (!default).then_some(self.program.as_path())
    }

    /// Checks the section: a configured program must be an absolute path.
    ///
    /// # Errors
    /// A description of the problem, in Italian.
    pub(crate) fn validate(&self) -> Result<(), String> {
        self.configured_program()
            .map_or(Ok(()), |program| check_program_path("program", program))
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
    fn options_outside_the_allowed_list_are_rejected() {
        let options = json!({ "options": { "RemoteForward": "8080 localhost:80" } });

        let error = SshSettings::parse(&map(options)).unwrap_err();

        assert!(error.contains("'RemoteForward'"), "{error}");
        assert!(error.contains("~/.ssh/config"), "{error}");
    }

    #[test]
    fn options_of_add_ssh_key_are_accepted() {
        let options = json!({ "options": {
            "IdentityFile": "~/.ssh/id_ed25519",
            "IdentitiesOnly": "yes",
            "StrictHostKeyChecking": "accept-new",
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
    fn default_program_is_not_a_configured_one() {
        for program in ["ssh", ""] {
            let config = SshConfig {
                program: PathBuf::from(program),
            };

            assert_eq!(config.configured_program(), None);
            assert_eq!(config.validate(), Ok(()));
        }
    }

    #[test]
    fn configured_program_must_be_absolute() {
        let config = SshConfig {
            program: PathBuf::from("bin/ssh"),
        };

        let error = config.validate().unwrap_err();

        assert!(error.contains("percorso assoluto"), "{error}");
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
