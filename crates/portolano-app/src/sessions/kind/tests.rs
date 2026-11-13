//! The kind of the real plans of the connectors, so that a change in their
//! shape shows up here.

use std::path::PathBuf;

use portolano_connectors::{ConnectOptions, Environment, Os, RdpClient, Registry};
use portolano_core::{AppConfig, CredentialSpec, Document, Node, PrivateKey, SecretRef};
use portolano_ops::connect::plan_connection;
use pretty_assertions::assert_eq;

use super::*;

fn environment() -> Environment {
    Environment {
        os: Os::MacOs,
        ssh: Some(PathBuf::from("/usr/bin/ssh")),
        ssh_add: Some(PathBuf::from("/usr/bin/ssh-add")),
        freerdp: Some(PathBuf::from("/opt/homebrew/bin/sdl-freerdp")),
        windows_app: Some(PathBuf::from("/Applications/Windows App.app")),
        self_exe: PathBuf::from("/Applications/Portolano.app/Contents/MacOS/portolano"),
        rejected: Vec::new(),
    }
}

/// A connection of `protocol` to an example host with an inline login,
/// with a password when `password` is set.
fn connection(protocol: &str, password: bool) -> Node {
    let mut node = Node::connection("srv01", protocol, "srv01.example.com");
    if let Some(connection) = node.as_connection_mut() {
        connection.credential = CredentialSpec::Inline {
            username: Some("operatore".to_owned()),
            domain: None,
            password: password.then(SecretRef::new),
        };
    }
    node
}

fn kind_of(node: Node, options: &ConnectOptions) -> Result<SessionKind, SessionError> {
    let id = node.id;
    let mut doc = Document::new("Test");
    doc.nodes = vec![node];
    let plan = plan_connection(
        &doc,
        &AppConfig::default(),
        &environment(),
        &Registry::with_defaults(),
        id,
        options,
    )
    .unwrap();
    session_kind(&plan)
}

#[test]
fn ssh_runs_in_a_terminal() {
    let kind = kind_of(connection("ssh", true), &ConnectOptions::default());

    assert_eq!(kind.unwrap(), SessionKind::Terminal);
}

#[test]
fn freerdp_with_its_password_runs_in_its_own_window() {
    let kind = kind_of(connection("rdp", true), &ConnectOptions::default());

    assert_eq!(kind.unwrap(), SessionKind::RdpExternal);
}

#[test]
fn freerdp_that_has_to_ask_for_the_password_runs_in_a_terminal() {
    let kind = kind_of(connection("rdp", false), &ConnectOptions::default());

    assert_eq!(kind.unwrap(), SessionKind::Terminal);
}

#[test]
fn windows_app_is_an_external_session() {
    let options = ConnectOptions {
        rdp_client: Some(RdpClient::WindowsApp),
        ..ConnectOptions::default()
    };

    let kind = kind_of(connection("rdp", true), &options);

    assert_eq!(kind.unwrap(), SessionKind::External);
}

#[test]
fn ssh_through_jump_hosts_runs_in_a_terminal() {
    let mut node = connection("ssh", true);
    if let Some(connection) = node.as_connection_mut() {
        connection.settings = serde_json::json!({
            "jump_hosts": [{ "address": "admin@bastion.example.com" }],
        })
        .as_object()
        .unwrap()
        .clone();
    }

    let kind = kind_of(node, &ConnectOptions::default());

    assert_eq!(kind.unwrap(), SessionKind::Terminal);
}

#[test]
fn ssh_with_a_key_in_the_vault_runs_in_a_terminal_after_adding_it() {
    let mut key_owner = Node::credential("Chiave");
    if let Some(credential) = key_owner.as_credential_mut() {
        credential.username = Some("operatore".to_owned());
        credential.private_key = Some(PrivateKey::Vault {
            secret: SecretRef::new(),
        });
    }
    let mut node = Node::connection("srv01", "ssh", "srv01.example.com");
    if let Some(connection) = node.as_connection_mut() {
        connection.credential = CredentialSpec::ById { id: key_owner.id };
    }
    let id = node.id;
    let mut doc = Document::new("Test");
    doc.nodes = vec![key_owner, node];
    let plan = plan_connection(
        &doc,
        &AppConfig::default(),
        &environment(),
        &Registry::with_defaults(),
        id,
        &ConnectOptions::default(),
    )
    .unwrap();

    assert!(matches!(
        plan.steps.first(),
        Some(LaunchStep::SshAddKey { .. })
    ));
    assert_eq!(session_kind(&plan).unwrap(), SessionKind::Terminal);
}

#[test]
fn a_terminal_that_wants_a_secret_on_stdin_is_refused_from_the_plan() {
    let plan = LaunchPlan {
        steps: vec![LaunchStep::Run {
            program: PathBuf::from("/opt/homebrew/bin/sdl-freerdp"),
            args: Vec::new(),
            env: Vec::new(),
            stdin_secret: Some(SecretRef::new()),
            askpass: None,
            wait: true,
        }],
        warnings: Vec::new(),
    };

    let kind = session_kind(&plan);

    assert!(matches!(kind, Err(SessionError::StdinSecretInTerminal)));
}

#[test]
fn a_new_terminal_window_is_not_a_session() {
    let options = ConnectOptions {
        new_window: true,
        ..ConnectOptions::default()
    };

    let kind = kind_of(connection("ssh", true), &options);

    assert!(matches!(kind, Err(SessionError::NotASession { .. })));
}

#[test]
fn an_empty_plan_is_not_a_session() {
    let kind = session_kind(&LaunchPlan::default());

    assert!(matches!(kind, Err(SessionError::NotASession { .. })));
}
