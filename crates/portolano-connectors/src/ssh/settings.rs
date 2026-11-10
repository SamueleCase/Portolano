//! Typed SSH settings of a connection and the `[ssh]` configuration section.

// Stub (WP0): the parameters are used once WP4 implements the bodies.
#![expect(unused_variables)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use portolano_core::AppConfig;
use portolano_core::model::is_false;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ConnectError;

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

/// A jump host: another SSH connection of the document, or a literal address.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
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
        todo!()
    }
}
