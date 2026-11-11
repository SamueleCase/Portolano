//! Typed RDP settings of a connection and the `[rdp]` configuration section.

// Stub (WP0): the parameters are used once WP4 implements the bodies.
#![expect(unused_variables)]

use std::path::PathBuf;

use portolano_core::model::is_false;
use portolano_core::{AppConfig, CredentialSpec};
use serde::{Deserialize, Serialize};

use crate::ConnectError;

/// The `settings` of an `rdp` connection. Fields at their default are
/// omitted when serialized.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RdpSettings {
    /// Size of the remote desktop.
    #[serde(skip_serializing_if = "is_fit")]
    pub resolution: Resolution,
    /// Scale factor in percent: 100, 140 or 180.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<u32>,
    /// Color depth in bits: 15, 16, 24 or 32.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_depth: Option<u8>,
    /// Remote Desktop Gateway.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway: Option<RdpGateway>,
    /// Use all monitors.
    #[serde(skip_serializing_if = "is_false")]
    pub multimon: bool,
    /// Connect to the administrative session.
    #[serde(skip_serializing_if = "is_false")]
    pub admin: bool,
    /// Share the clipboard; enabled by default.
    #[serde(skip_serializing_if = "is_true")]
    pub clipboard: bool,
    /// MAC address for Wake-on-LAN; only stored in this phase.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wol_mac: Option<String>,
}

impl Default for RdpSettings {
    fn default() -> Self {
        Self {
            resolution: Resolution::Fit,
            scale: None,
            color_depth: None,
            gateway: None,
            multimon: false,
            admin: false,
            clipboard: true,
            wol_mac: None,
        }
    }
}

fn is_fit(resolution: &Resolution) -> bool {
    *resolution == Resolution::Fit
}

fn is_true(value: &bool) -> bool {
    *value
}

/// Size of the remote desktop, serialized with the tag `"mode"`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum Resolution {
    /// Follow the window size.
    #[default]
    Fit,
    /// Full screen.
    Fullscreen,
    /// A fixed size.
    Fixed {
        /// Width in pixels.
        width: u32,
        /// Height in pixels.
        height: u32,
    },
}

/// A Remote Desktop Gateway.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RdpGateway {
    /// Gateway host.
    pub host: String,
    /// Gateway port; `None` means the client's default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    /// Login data for the gateway, resolved like a connection's.
    #[serde(default, skip_serializing_if = "CredentialSpec::is_inherit")]
    pub credential: CredentialSpec,
}

/// An RDP client program.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RdpClient {
    /// FreeRDP 3: supports the password on stdin.
    FreeRdp,
    /// Microsoft Windows App: asks for the password itself.
    WindowsApp,
}

/// The `[rdp]` section of `config.toml` (DEC-003).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RdpConfig {
    /// Which client to use.
    pub client: RdpClientChoice,
    /// Path of the FreeRDP program, overriding the search; an empty path
    /// counts as unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freerdp_program: Option<PathBuf>,
    /// Server certificate policy passed to FreeRDP as `/cert:`.
    pub cert: CertPolicy,
}

impl RdpConfig {
    /// Reads the `[rdp]` section of `config`; defaults if it is missing.
    ///
    /// # Errors
    /// [`ConnectError::InvalidConfig`] if the section does not match.
    pub fn from_app_config(config: &AppConfig) -> Result<RdpConfig, ConnectError> {
        todo!()
    }
}

/// The `client` value of `[rdp]`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RdpClientChoice {
    /// FreeRDP if installed, otherwise Windows App.
    #[default]
    #[serde(rename = "auto")]
    Auto,
    /// Always FreeRDP.
    #[serde(rename = "freerdp")]
    FreeRdp,
    /// Always Windows App.
    #[serde(rename = "windows-app")]
    WindowsApp,
}

/// Server certificate policy of FreeRDP (`/cert:`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CertPolicy {
    /// Trust on first use.
    #[default]
    Tofu,
    /// Accept any certificate.
    Ignore,
    /// Reject unknown certificates.
    Deny,
}
