//! The RDP protocol, through FreeRDP or Windows App (SPEC §7.3).

mod freerdp;
mod settings;
mod windows_app;

use std::path::PathBuf;

use portolano_core::{Document, Issue, Node, ResolvedCredential};
use serde_json::Value;
use uuid::Uuid;

pub use settings::{
    CertPolicy, RdpClient, RdpClientChoice, RdpConfig, RdpGateway, RdpSettings, Resolution,
};

use crate::ConnectError;
use crate::decode::decode_section;
use crate::env::Environment;
use crate::plan::LaunchPlan;
use crate::protocol::{
    PlanContext, Protocol, apply_session_credential, check_value, connection_node, error_issue,
};
use crate::terminal::relaunch_step;

/// Port of RDP when the connection does not set one.
const RDP_PORT: u16 = 3389;

/// The `rdp` protocol.
#[derive(Debug, Clone, Copy, Default)]
pub struct RdpProtocol;

impl Protocol for RdpProtocol {
    fn id(&self) -> &'static str {
        "rdp"
    }

    fn display_name(&self) -> &'static str {
        "RDP"
    }

    fn default_port(&self) -> u16 {
        RDP_PORT
    }

    fn validate_settings(&self, settings: &serde_json::Map<String, Value>) -> Result<(), String> {
        RdpSettings::parse(settings).map(drop)
    }

    fn validate_config(&self, section: &toml::Value) -> Result<(), String> {
        decode_section::<RdpConfig>(section)?.validate()
    }

    fn validate(&self, doc: &Document, node: &Node) -> Vec<Issue> {
        let Some(connection) = node.as_connection() else {
            return Vec::new();
        };
        let settings = match RdpSettings::parse(&connection.settings) {
            Ok(settings) => settings,
            Err(reason) => {
                return vec![error_issue(
                    node.id,
                    "invalid_settings",
                    format!("impostazioni rdp non valide: {reason}"),
                )];
            }
        };
        let Some(gateway) = &settings.gateway else {
            return Vec::new();
        };
        match doc.resolve_spec(node.id, &gateway.credential) {
            Ok(_) => Vec::new(),
            // Already reported by the core as `dangling_ref`.
            Err(portolano_core::Error::DanglingReference { .. }) => Vec::new(),
            Err(error) => vec![error_issue(
                node.id,
                "invalid_gateway_credential",
                format!("credenziale del gateway non valida: {error}"),
            )],
        }
    }

    fn summary(&self, doc: &Document, node: &Node) -> String {
        let Some(connection) = node.as_connection() else {
            return String::new();
        };
        let login = doc
            .resolve_credential(node.id)
            .ok()
            .and_then(|credential| account(&credential))
            .map(|account| format!("{account}@"))
            .unwrap_or_default();
        let port = connection
            .port
            .map(|port| format!(":{port}"))
            .unwrap_or_default();
        format!("{login}{}{port}", connection.host)
    }

    fn plan(&self, ctx: &PlanContext) -> Result<LaunchPlan, ConnectError> {
        let (node, connection) = connection_node(ctx.doc, ctx.node_id)?;
        let settings = RdpSettings::parse(&connection.settings).map_err(|reason| {
            ConnectError::InvalidSettings {
                id: ctx.node_id,
                reason,
            }
        })?;
        let config = RdpConfig::from_app_config(ctx.config)?;
        let requested = ctx.options.rdp_client.or(settings.client);
        let client = choose_client(requested, config.client, ctx.env)?;

        let mut warnings = Vec::new();
        if ctx.options.new_window && matches!(client, ChosenClient::WindowsApp { .. }) {
            warnings.push(
                "--new-window non ha effetto con Windows App: apre una propria finestra".to_owned(),
            );
        }
        let credential = apply_session_credential(
            ctx.doc.resolve_credential(ctx.node_id)?,
            ctx.options,
            &mut warnings,
        );
        let target = RdpTarget {
            node_id: ctx.node_id,
            name: &node.name,
            host: &connection.host,
            port: connection.port.unwrap_or(RDP_PORT),
            credential,
            settings: &settings,
            gateway: gateway_login(ctx, &settings)?,
        };

        let mut plan = match client {
            ChosenClient::FreeRdp(program) => {
                let plan = freerdp::plan(program, &target, config.cert, ctx.options.wait)?;
                if ctx.options.new_window {
                    // In a new terminal window FreeRDP can ask what it needs
                    // (DEC-033.1). The plan above only validates the
                    // connection early; the relaunched process plans again,
                    // with the same client and waiting if asked, whatever
                    // the connection or the configuration choose, and shows
                    // its own warnings.
                    let mut options = vec!["--client", "freerdp"];
                    if ctx.options.wait {
                        options.push("--wait");
                    }
                    return Ok(LaunchPlan {
                        steps: vec![relaunch_step(ctx, &options)],
                        warnings: Vec::new(),
                    });
                }
                plan
            }
            ChosenClient::WindowsApp { app, fallback } => {
                windows_app::plan(&app, &target, fallback)?
            }
        };
        warnings.append(&mut plan.warnings);
        plan.warnings = warnings;
        Ok(plan)
    }
}

/// The gateway of the connection, if any, with its credential resolved in
/// the context of the connection.
fn gateway_login<'a>(
    ctx: &PlanContext,
    settings: &'a RdpSettings,
) -> Result<Option<GatewayLogin<'a>>, ConnectError> {
    let Some(gateway) = &settings.gateway else {
        return Ok(None);
    };
    Ok(Some(GatewayLogin {
        host: &gateway.host,
        port: gateway.port,
        credential: ctx.doc.resolve_spec(ctx.node_id, &gateway.credential)?,
    }))
}

/// An RDP connection with its login data resolved.
struct RdpTarget<'a> {
    /// The connection node, named by errors.
    node_id: Uuid,
    /// Name of the node, used for the `.rdp` file of Windows App.
    name: &'a str,
    host: &'a str,
    port: u16,
    credential: ResolvedCredential,
    settings: &'a RdpSettings,
    gateway: Option<GatewayLogin<'a>>,
}

impl RdpTarget<'_> {
    /// Rejects control characters in the host, user and domain of the
    /// destination, which every client receives as text: they would end a
    /// line of the `.rdp` file or reach FreeRDP's arguments and its log.
    ///
    /// # Errors
    /// [`ConnectError::InvalidValue`] naming the field.
    fn check_login_values(&self) -> Result<(), ConnectError> {
        let id = self.node_id;
        check_value(id, "host", self.host, &[])?;
        if let Some(user) = &self.credential.username {
            check_value(id, "username", user, &[])?;
        }
        if let Some(domain) = &self.credential.domain {
            check_value(id, "domain", domain, &[])?;
        }
        Ok(())
    }
}

/// The gateway of a connection with its login data resolved.
struct GatewayLogin<'a> {
    host: &'a str,
    port: Option<u16>,
    credential: ResolvedCredential,
}

impl GatewayLogin<'_> {
    /// `host[:port]`.
    fn address(&self) -> String {
        match self.port {
            Some(port) => format!("{}:{port}", self.host),
            None => self.host.to_owned(),
        }
    }
}

/// `DOMAIN\user` or `user`, if there is a user.
fn account(credential: &ResolvedCredential) -> Option<String> {
    let user = credential.username.as_ref()?;
    Some(match &credential.domain {
        Some(domain) => format!("{domain}\\{user}"),
        None => user.clone(),
    })
}

/// The client that runs the connection.
#[derive(Debug, PartialEq, Eq)]
enum ChosenClient {
    /// FreeRDP, with its program.
    FreeRdp(PathBuf),
    /// Windows App, with its bundle; `fallback` when chosen by `auto`
    /// because FreeRDP is missing.
    WindowsApp { app: PathBuf, fallback: bool },
}

/// Picks the client (SPEC §7.3): `requested` (from `--client` or the
/// connection) wins over the configuration; `auto` prefers FreeRDP.
fn choose_client(
    requested: Option<RdpClient>,
    configured: RdpClientChoice,
    env: &Environment,
) -> Result<ChosenClient, ConnectError> {
    let choice = match requested {
        Some(RdpClient::FreeRdp) => RdpClientChoice::FreeRdp,
        Some(RdpClient::WindowsApp) => RdpClientChoice::WindowsApp,
        None => configured,
    };
    // A FreeRDP set in the configuration but refused is reported as such,
    // and `auto` does not silently fall back to Windows App instead.
    let freerdp_missing = || {
        env.rejection("rdp")
            .unwrap_or_else(|| ConnectError::ClientNotFound {
                client: "FreeRDP".to_owned(),
                hint: "installalo con `brew install freerdp`".to_owned(),
            })
    };
    match choice {
        RdpClientChoice::FreeRdp => env
            .freerdp
            .clone()
            .map(ChosenClient::FreeRdp)
            .ok_or_else(freerdp_missing),
        RdpClientChoice::WindowsApp => env
            .windows_app
            .clone()
            .map(|app| ChosenClient::WindowsApp {
                app,
                fallback: false,
            })
            .ok_or_else(|| ConnectError::ClientNotFound {
                client: "Windows App".to_owned(),
                hint: "installala dal Mac App Store".to_owned(),
            }),
        RdpClientChoice::Auto => match (&env.freerdp, &env.windows_app) {
            (Some(freerdp), _) => Ok(ChosenClient::FreeRdp(freerdp.clone())),
            (None, _) if env.rejection("rdp").is_some() => Err(freerdp_missing()),
            (None, Some(app)) => Ok(ChosenClient::WindowsApp {
                app: app.clone(),
                fallback: true,
            }),
            (None, None) => Err(freerdp_missing()),
        },
    }
}

#[cfg(test)]
mod tests;
