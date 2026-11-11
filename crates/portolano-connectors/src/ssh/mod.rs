//! The SSH protocol, through the system OpenSSH (SPEC §7.2).

pub mod askpass;
mod plan;
mod settings;

use portolano_core::{Document, Issue, Node};
use serde_json::Value;

pub use settings::{JumpHost, SshConfig, SshSettings};

use self::plan::{Endpoint, Jump, JumpLink, SSH_PORT, SshLaunch};
use crate::ConnectError;
use crate::decode::decode_section;
use crate::plan::LaunchPlan;
use crate::protocol::{
    PlanContext, Protocol, apply_session_credential, connection_node, error_issue,
};
use crate::terminal::relaunch_step;

/// The `ssh` protocol.
#[derive(Debug, Clone, Copy, Default)]
pub struct SshProtocol;

impl Protocol for SshProtocol {
    fn id(&self) -> &'static str {
        "ssh"
    }

    fn display_name(&self) -> &'static str {
        "SSH"
    }

    fn default_port(&self) -> u16 {
        SSH_PORT
    }

    fn validate_settings(&self, settings: &serde_json::Map<String, Value>) -> Result<(), String> {
        SshSettings::parse(settings).map(drop)
    }

    fn validate_config(&self, section: &toml::Value) -> Result<(), String> {
        decode_section::<SshConfig>(section).map(drop)
    }

    fn validate(&self, doc: &Document, node: &Node) -> Vec<Issue> {
        let Some(connection) = node.as_connection() else {
            return Vec::new();
        };
        let settings = match SshSettings::parse(&connection.settings) {
            Ok(settings) => settings,
            Err(reason) => {
                return vec![error_issue(
                    node.id,
                    "invalid_settings",
                    format!("impostazioni ssh non valide: {reason}"),
                )];
            }
        };
        match plan::jump_chain(doc, node.id, &settings) {
            Ok(_) => Vec::new(),
            Err(error @ ConnectError::JumpCycle { .. }) => {
                vec![error_issue(node.id, "jump_cycle", error.to_string())]
            }
            // A reference to a missing node is already a `dangling_ref` of
            // the core.
            Err(ConnectError::InvalidJumpHost { id, .. }) if doc.find(id).is_none() => Vec::new(),
            Err(error) => vec![error_issue(node.id, "invalid_jump_host", error.to_string())],
        }
    }

    fn summary(&self, doc: &Document, node: &Node) -> String {
        let Some(connection) = node.as_connection() else {
            return String::new();
        };
        let user = doc
            .resolve_credential(node.id)
            .ok()
            .and_then(|credential| credential.username)
            .map(|user| format!("{user}@"))
            .unwrap_or_default();
        let port = connection.port.unwrap_or(SSH_PORT);
        format!("{user}{}:{port}", connection.host)
    }

    fn plan(&self, ctx: &PlanContext) -> Result<LaunchPlan, ConnectError> {
        let (_, connection) = connection_node(ctx.doc, ctx.node_id)?;
        let settings = SshSettings::parse(&connection.settings).map_err(|reason| {
            ConnectError::InvalidSettings {
                id: ctx.node_id,
                reason,
            }
        })?;
        let ssh = ctx
            .env
            .ssh
            .clone()
            .ok_or_else(|| ConnectError::ClientNotFound {
                client: "ssh".to_owned(),
                hint:
                    "installa OpenSSH o imposta `program` nella sezione [ssh] della configurazione"
                        .to_owned(),
            })?;

        let mut warnings = Vec::new();
        let credential = apply_session_credential(
            ctx.doc.resolve_credential(ctx.node_id)?,
            ctx.options,
            &mut warnings,
        );
        let target = Endpoint {
            node: ctx.node_id,
            host: connection.host.clone(),
            port: connection.port.unwrap_or(SSH_PORT),
            credential,
        };
        let jumps = plan::jump_chain(ctx.doc, ctx.node_id, &settings)?
            .into_iter()
            .map(|link| resolve_jump(ctx.doc, link))
            .collect::<Result<Vec<_>, _>>()?;

        let mut plan = plan::build(&SshLaunch {
            ssh,
            ssh_add_available: ctx.env.ssh_add.is_some(),
            askpass_program: ctx.env.self_exe.clone(),
            target,
            jumps,
            settings: &settings,
        })?;
        if ctx.options.new_window {
            // The plan above only validates the connection early; the new
            // window plans again and shows its own warnings.
            return Ok(LaunchPlan {
                steps: vec![relaunch_step(ctx)],
                warnings: Vec::new(),
            });
        }
        warnings.append(&mut plan.warnings);
        plan.warnings = warnings;
        Ok(plan)
    }
}

fn resolve_jump(doc: &Document, link: JumpLink) -> Result<Jump, ConnectError> {
    match link {
        JumpLink::Address(address) => Ok(Jump::Address(address)),
        JumpLink::Node { id, connection } => Ok(Jump::Endpoint(Endpoint {
            node: id,
            host: connection.host.clone(),
            port: connection.port.unwrap_or(SSH_PORT),
            credential: doc.resolve_credential(id)?,
        })),
    }
}

#[cfg(test)]
mod tests;
