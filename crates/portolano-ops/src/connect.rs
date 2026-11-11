//! Preparation of `connect` (SPEC §7.1, DEC-016.3), in two phases: whether
//! login data must be asked ([`login_request`]), then the launch plan
//! ([`plan_connection`]). Running the plan is
//! [`portolano_connectors::execute`] (or `prepare` and `launch`), with a
//! [`SessionVault`](crate::vault::SessionVault) when login data was typed.

use portolano_connectors::{
    ConnectError, ConnectOptions, Environment, LaunchPlan, PlanContext, Registry,
};
use portolano_core::{AppConfig, CredentialOrigin, Document, Node};
use serde::Serialize;
use uuid::Uuid;

use crate::tree_util::node_path;
use crate::{OpsError, Result};

/// Protocol whose login has a Windows domain.
const RDP: &str = "rdp";

/// Login data to ask before connecting: the credential of the connection is
/// "ask at launch".
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LoginRequest {
    /// The connection.
    pub node: Uuid,
    /// Its path.
    pub path: String,
    /// Its protocol.
    pub protocol: String,
    /// Whether a Windows domain makes sense (RDP).
    pub ask_domain: bool,
}

/// The login data to ask for connection `id`, or `None` if its credential
/// does not resolve to "ask at launch".
///
/// # Errors
/// [`portolano_core::Error::NodeNotFound`], [`OpsError::NotAConnection`],
/// credential resolution errors.
pub fn login_request(doc: &Document, id: Uuid) -> Result<Option<LoginRequest>> {
    let (node, protocol) = connection_of(doc, id)?;
    if doc.resolve_credential(id)?.origin != CredentialOrigin::Prompt {
        return Ok(None);
    }
    Ok(Some(LoginRequest {
        node: node.id,
        path: node_path(doc, id),
        protocol: protocol.to_owned(),
        ask_domain: protocol == RDP,
    }))
}

/// The launch plan of connection `id`. It holds secrets only as references;
/// `options.session_credential` carries the login data typed for a
/// credential "ask at launch".
///
/// # Errors
/// [`portolano_core::Error::NodeNotFound`], [`OpsError::NotAConnection`],
/// [`ConnectError::UnknownProtocol`] and the errors of the protocol's plan.
pub fn plan_connection(
    doc: &Document,
    config: &AppConfig,
    env: &Environment,
    protocols: &Registry,
    id: Uuid,
    options: &ConnectOptions,
) -> Result<LaunchPlan> {
    let (_, protocol_id) = connection_of(doc, id)?;
    let protocol = protocols
        .get(protocol_id)
        .ok_or_else(|| ConnectError::UnknownProtocol {
            id,
            protocol: protocol_id.to_owned(),
        })?;
    let context = PlanContext {
        doc,
        node_id: id,
        config,
        env,
        options,
    };
    Ok(protocol.plan(&context)?)
}

fn connection_of(doc: &Document, id: Uuid) -> Result<(&Node, &str)> {
    let node = doc
        .find(id)
        .ok_or(portolano_core::Error::NodeNotFound { id })?;
    let connection = node
        .as_connection()
        .ok_or_else(|| OpsError::NotAConnection {
            path: node_path(doc, id),
        })?;
    Ok((node, &connection.protocol))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use portolano_connectors::{Os, render};
    use portolano_core::CredentialSpec;
    use portolano_core::vault::MemoryVault;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::testutil::*;
    use crate::vault::{Login, SessionVault};

    fn environment() -> Environment {
        Environment {
            os: Os::MacOs,
            ssh: Some(PathBuf::from("/usr/bin/ssh")),
            ssh_add: Some(PathBuf::from("/usr/bin/ssh-add")),
            freerdp: Some(PathBuf::from("/opt/homebrew/bin/sdl-freerdp")),
            windows_app: None,
            self_exe: PathBuf::from("/usr/local/bin/portolano"),
        }
    }

    fn plan(doc: &Document, path: &str, options: &ConnectOptions) -> Result<LaunchPlan> {
        plan_connection(
            doc,
            &AppConfig::default(),
            &environment(),
            &Registry::with_defaults(),
            id_of(doc, path),
            options,
        )
    }

    #[test]
    fn prompt_credential_needs_a_login() {
        let doc = document(vec![connection_with("dc01", "rdp", CredentialSpec::Prompt)]);
        let request = login_request(&doc, id_of(&doc, "/dc01")).unwrap().unwrap();
        assert_eq!(request.path, "/dc01");
        assert!(request.ask_domain);
    }

    #[test]
    fn other_credentials_need_no_login() {
        let doc = document(vec![connection("web01", "ssh")]);
        assert_eq!(login_request(&doc, id_of(&doc, "/web01")).unwrap(), None);
    }

    #[test]
    fn login_of_a_folder_is_refused() {
        let doc = document(vec![folder("Clienti", vec![])]);
        let result = login_request(&doc, id_of(&doc, "/Clienti"));
        assert!(matches!(result, Err(OpsError::NotAConnection { .. })));
    }

    #[test]
    fn plan_uses_the_protocol_of_the_connection() {
        let doc = document(vec![connection("web01", "ssh")]);
        let rendered = render(&plan(&doc, "/web01", &ConnectOptions::default()).unwrap());
        assert!(rendered.starts_with("/usr/bin/ssh"));
        assert!(rendered.ends_with("web01.example.com"));
    }

    #[test]
    fn unknown_protocol_cannot_be_planned() {
        let doc = document(vec![connection("legacy", "telnet")]);
        let result = plan(&doc, "/legacy", &ConnectOptions::default());
        assert!(matches!(
            result,
            Err(OpsError::Connect(ConnectError::UnknownProtocol { .. }))
        ));
    }

    #[test]
    fn session_login_is_used_and_its_password_never_rendered() {
        let doc = document(vec![connection_with(
            "web01",
            "ssh",
            CredentialSpec::Prompt,
        )]);
        let base = MemoryVault::default();
        let session = SessionVault::new(&base);
        let credential = session
            .session_credential(Login {
                username: Some("operatore".to_owned()),
                domain: None,
                password: Some(secret(SENTINEL)),
            })
            .unwrap();
        let options = ConnectOptions {
            session_credential: Some(credential),
            ..ConnectOptions::default()
        };
        let rendered = render(&plan(&doc, "/web01", &options).unwrap());
        assert!(rendered.contains("-l operatore"));
        assert!(rendered.contains("<vault:"));
        assert!(!rendered.contains(SENTINEL));
    }
}
