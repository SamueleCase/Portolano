//! Typed RDP settings of a connection and the `[rdp]` configuration section.

use std::path::{Path, PathBuf};

use portolano_core::model::is_false;
use portolano_core::{AppConfig, CredentialSpec};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::ConnectError;
use crate::decode::{decode_config_section, decode_settings};
use crate::env::check_program_path;

/// Scale factors accepted by FreeRDP (`/scale:`).
const SCALES: [u32; 3] = [100, 140, 180];

/// Color depths accepted by RDP.
const COLOR_DEPTHS: [u8; 4] = [15, 16, 24, 32];

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
    /// Client of this connection; `--client` overrides it and it overrides
    /// `[rdp] client` (DEC-016.7).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client: Option<RdpClient>,
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
            client: None,
        }
    }
}

impl RdpSettings {
    /// Decodes the `settings` of an `rdp` connection and checks the values
    /// that RDP clients accept.
    ///
    /// # Errors
    /// A description of the first problem, in Italian.
    pub(crate) fn parse(settings: &Map<String, Value>) -> Result<RdpSettings, String> {
        let parsed: RdpSettings = decode_settings(settings)?;
        if let Some(scale) = parsed.scale.filter(|scale| !SCALES.contains(scale)) {
            return Err(format!(
                "campo 'scale': {scale} non ammesso (ammessi: 100, 140, 180)"
            ));
        }
        if let Some(depth) = parsed
            .color_depth
            .filter(|depth| !COLOR_DEPTHS.contains(depth))
        {
            return Err(format!(
                "campo 'color_depth': {depth} non ammesso (ammessi: 15, 16, 24, 32)"
            ));
        }
        if let Resolution::Fixed { width, height } = parsed.resolution
            && (width == 0 || height == 0)
        {
            return Err(
                "campo 'resolution': larghezza e altezza devono essere positive".to_owned(),
            );
        }
        if parsed
            .gateway
            .as_ref()
            .is_some_and(|gateway| gateway.host.trim().is_empty())
        {
            return Err("campo 'gateway': host vuoto".to_owned());
        }
        Ok(parsed)
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

/// An RDP client program. Serialized as in `[rdp] client`: `"freerdp"`,
/// `"windows-app"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RdpClient {
    /// FreeRDP 3: supports the password on stdin.
    #[serde(rename = "freerdp")]
    FreeRdp,
    /// Microsoft Windows App: asks for the password itself.
    #[serde(rename = "windows-app")]
    WindowsApp,
}

/// The `[rdp]` section of `config.toml` (DEC-003).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RdpConfig {
    /// Which client to use.
    pub client: RdpClientChoice,
    /// Absolute path of the FreeRDP program, overriding the search
    /// (DEC-024.2); an empty path counts as unset.
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
        decode_config_section(config, "rdp")
    }

    /// The FreeRDP program set in the configuration, if any.
    pub(crate) fn configured_freerdp_program(&self) -> Option<&Path> {
        self.freerdp_program
            .as_deref()
            .filter(|program| !program.as_os_str().is_empty())
    }

    /// Checks the section: a configured program must be an absolute path.
    ///
    /// # Errors
    /// A description of the problem, in Italian.
    pub(crate) fn validate(&self) -> Result<(), String> {
        self.configured_freerdp_program().map_or(Ok(()), |program| {
            check_program_path("freerdp_program", program)
        })
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

impl CertPolicy {
    /// The value of FreeRDP's `/cert:` option.
    pub(crate) fn as_freerdp_value(self) -> &'static str {
        match self {
            CertPolicy::Tofu => "tofu",
            CertPolicy::Ignore => "ignore",
            CertPolicy::Deny => "deny",
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use uuid::Uuid;

    use super::*;

    fn map(value: Value) -> Map<String, Value> {
        value.as_object().cloned().unwrap()
    }

    #[test]
    fn default_settings_serialize_to_an_empty_object() {
        let json = serde_json::to_value(RdpSettings::default()).unwrap();

        assert_eq!(json, json!({}));
    }

    #[test]
    fn omitted_clipboard_defaults_to_true() {
        let settings = RdpSettings::parse(&map(json!({ "multimon": true }))).unwrap();

        assert!(settings.clipboard);
    }

    #[test]
    fn disabled_clipboard_is_serialized() {
        let settings = RdpSettings {
            clipboard: false,
            ..RdpSettings::default()
        };

        let json = serde_json::to_value(&settings).unwrap();

        assert_eq!(json, json!({ "clipboard": false }));
    }

    #[test]
    fn full_settings_round_trip() {
        let credential = Uuid::new_v4();
        let settings = RdpSettings {
            resolution: Resolution::Fixed {
                width: 1920,
                height: 1080,
            },
            scale: Some(140),
            color_depth: Some(24),
            gateway: Some(RdpGateway {
                host: "gw.example.com".into(),
                port: Some(443),
                credential: CredentialSpec::ById { id: credential },
            }),
            multimon: true,
            admin: true,
            clipboard: false,
            wol_mac: Some("00:00:5E:00:53:01".into()),
            client: Some(RdpClient::WindowsApp),
        };

        let json = serde_json::to_value(&settings).unwrap();
        let back = RdpSettings::parse(&map(json.clone())).unwrap();

        assert_eq!(
            json["resolution"],
            json!({ "mode": "fixed", "width": 1920, "height": 1080 })
        );
        assert_eq!(json["client"], json!("windows-app"));
        assert_eq!(
            json["gateway"]["credential"],
            json!({ "mode": "by_id", "$ref": credential.to_string() })
        );
        assert_eq!(back, settings);
    }

    #[test]
    fn unknown_gateway_field_names_the_gateway() {
        let error = RdpSettings::parse(&map(
            json!({ "gateway": { "host": "gw.example.com", "prot": 1 } }),
        ))
        .unwrap_err();

        assert!(
            error.starts_with("campo 'gateway': campo sconosciuto 'prot'"),
            "{error}"
        );
    }

    #[test]
    fn unknown_resolution_mode_is_rejected() {
        let error =
            RdpSettings::parse(&map(json!({ "resolution": { "mode": "huge" } }))).unwrap_err();

        assert!(error.contains("'huge'"), "{error}");
    }

    #[test]
    fn unsupported_scale_is_rejected() {
        let error = RdpSettings::parse(&map(json!({ "scale": 150 }))).unwrap_err();

        assert!(error.contains("150"), "{error}");
    }

    #[test]
    fn unsupported_color_depth_is_rejected() {
        let error = RdpSettings::parse(&map(json!({ "color_depth": 8 }))).unwrap_err();

        assert!(error.contains("color_depth"), "{error}");
    }

    #[test]
    fn config_section_is_read_with_defaults() {
        let mut app = AppConfig::default();
        app.protocols.insert(
            "rdp".into(),
            toml::from_str("client = \"windows-app\"").unwrap(),
        );

        let config = RdpConfig::from_app_config(&app).unwrap();

        assert_eq!(config.client, RdpClientChoice::WindowsApp);
        assert_eq!(config.cert, CertPolicy::Tofu);
    }

    #[test]
    fn invalid_config_value_is_an_invalid_config_error() {
        let mut app = AppConfig::default();
        app.protocols
            .insert("rdp".into(), toml::from_str("cert = \"maybe\"").unwrap());

        let error = RdpConfig::from_app_config(&app).unwrap_err();

        assert!(error.to_string().contains("'maybe'"), "{error}");
    }
}
