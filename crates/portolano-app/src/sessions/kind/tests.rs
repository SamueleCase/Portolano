//! The kind of the real plans of the connectors, so that a change in their
//! shape shows up here.

use std::path::PathBuf;

use portolano_connectors::{ConnectOptions, Environment, Os, RdpClient, Registry};
use portolano_core::{AppConfig, CredentialSpec, Document, Node, SecretRef};
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
