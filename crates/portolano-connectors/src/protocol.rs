//! The [`Protocol`] extension point and the inputs of a launch plan
//! (SPEC §7.1).

use portolano_core::{
    AppConfig, Connection, CredentialOrigin, Document, Issue, Node, ResolvedCredential, SecretRef,
    Severity,
};
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

    /// Checks the protocol's section of `config.toml`: `section` is the
    /// top-level table whose name equals [`Protocol::id`] (DEC-003, DEC-008).
    /// The CLI calls it on `config set`, so the core never knows the fields.
    ///
    /// # Errors
    /// A description of the problem, in Italian.
    fn validate_config(&self, section: &toml::Value) -> Result<(), String>;

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
    /// RDP client chosen on the command line; it overrides the connection's
    /// setting and the configuration (DEC-016.7).
    pub rdp_client: Option<RdpClient>,
    /// Wait for the client to exit (RDP detaches by default).
    pub wait: bool,
    /// Global options to pass again when Portolano relaunches itself in a new
    /// window (`--file`, `--vault`, `--vault-file`, `--config`), DEC-004.
    /// They are never secrets.
    pub relaunch_args: Vec<String>,
    /// Login data entered at connection time, used when the connection's
    /// credential resolves to "ask at launch" (`Prompt`), DEC-016.3.
    pub session_credential: Option<SessionCredential>,
}

/// Login data asked at connection time for a credential in `Prompt` mode
/// (DEC-016.3). It is used like an inline credential; the password stays in
/// a vault, usually an in-memory one that lives for the session only.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SessionCredential {
    /// User name.
    pub username: Option<String>,
    /// Windows domain.
    pub domain: Option<String>,
    /// Password, in the vault passed to [`execute`](crate::execute).
    pub password: Option<SecretRef>,
}

/// The node `id` of `doc` and its connection data.
///
/// # Errors
/// [`ConnectError::Core`] if the node does not exist,
/// [`ConnectError::NotAConnection`] if it is not a connection.
pub(crate) fn connection_node(
    doc: &Document,
    id: Uuid,
) -> Result<(&Node, &Connection), ConnectError> {
    let node = doc
        .find(id)
        .ok_or(portolano_core::Error::NodeNotFound { id })?;
    let connection = node
        .as_connection()
        .ok_or(ConnectError::NotAConnection { id })?;
    Ok((node, connection))
}

/// `host:port`, with an IPv6 address in brackets as `ssh -J` and FreeRDP's
/// `/v:` expect.
pub(crate) fn host_port(host: &str, port: u16) -> String {
    if host.contains(':') {
        format!("[{host}]:{port}")
    } else {
        format!("{host}:{port}")
    }
}

/// Applies the session credential to a credential in `Prompt` mode
/// (DEC-016.3): with one, it acts as inline data; without, the client asks
/// for the login itself and a warning says so.
pub(crate) fn apply_session_credential(
    credential: ResolvedCredential,
    options: &ConnectOptions,
    warnings: &mut Vec<String>,
) -> ResolvedCredential {
    if credential.origin != CredentialOrigin::Prompt {
        return credential;
    }
    match &options.session_credential {
        Some(session) => ResolvedCredential {
            username: session.username.clone(),
            domain: session.domain.clone(),
            password: session.password.clone(),
            ..credential
        },
        None => {
            warnings.push(
                "credenziale da chiedere all'avvio: utente e password li chiederà il client"
                    .to_owned(),
            );
            ResolvedCredential {
                username: None,
                domain: None,
                password: None,
                ..credential
            }
        }
    }
}

/// An error [`Issue`] reported by a protocol about `node`.
pub(crate) fn error_issue(node: Uuid, code: &'static str, message: String) -> Issue {
    Issue {
        severity: Severity::Error,
        node: Some(node),
        code,
        message,
    }
}
