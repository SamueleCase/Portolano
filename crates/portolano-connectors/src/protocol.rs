//! The [`Protocol`] extension point and the inputs of a launch plan
//! (SPEC §7.1).

use portolano_core::{AppConfig, Document, Issue, Node};
use serde_json::Value;
use uuid::Uuid;

use crate::ConnectError;
use crate::env::Environment;
use crate::plan::LaunchPlan;
use crate::rdp::RdpClient;

/// A connection protocol, identified by the `protocol` field of
/// [`Connection`](portolano_core::Connection).
pub trait Protocol: Send + Sync {
    /// Identifier stored in documents, such as `"ssh"`.
    fn id(&self) -> &'static str;

    /// Name shown to the user, such as `"SSH"`.
    fn display_name(&self) -> &'static str;

    /// Port used when the connection does not set one.
    fn default_port(&self) -> u16;

    /// Checks that `settings` deserialize into the protocol's typed settings.
    ///
    /// # Errors
    /// A description of the problem, in Italian.
    fn validate_settings(&self, settings: &serde_json::Map<String, Value>) -> Result<(), String>;

    /// Protocol-specific problems of a connection node.
    fn validate(&self, doc: &Document, node: &Node) -> Vec<Issue>;

    /// One-line summary for `tree` and `list`, such as
    /// `"admin@web01.example.com:22"`.
    fn summary(&self, doc: &Document, node: &Node) -> String;

    /// Builds the launch plan of a connection. Pure: no I/O, and secrets only
    /// as [`SecretRef`](portolano_core::SecretRef).
    ///
    /// # Errors
    /// Invalid node, settings or references, or no usable client.
    fn plan(&self, ctx: &PlanContext) -> Result<LaunchPlan, ConnectError>;
}

/// Everything [`Protocol::plan`] needs.
#[derive(Debug, Clone, Copy)]
pub struct PlanContext<'a> {
    /// The document that contains the connection.
    pub doc: &'a Document,
    /// The connection to plan.
    pub node_id: Uuid,
    /// The application configuration.
    pub config: &'a AppConfig,
    /// The programs available on this machine.
    pub env: &'a Environment,
    /// The options of `portolano connect`.
    pub options: &'a ConnectOptions,
}

/// Options of `portolano connect`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConnectOptions {
    /// Open the connection in a new terminal window.
    pub new_window: bool,
    /// RDP client chosen on the command line; it overrides the configuration.
    pub rdp_client: Option<RdpClient>,
    /// Wait for the client to exit (RDP detaches by default).
    pub wait: bool,
    /// Global options to pass again when Portolano relaunches itself in a new
    /// window (`--file`, `--vault`, `--vault-file`, `--config`), DEC-004.
    /// They are never secrets.
    pub relaunch_args: Vec<String>,
}
