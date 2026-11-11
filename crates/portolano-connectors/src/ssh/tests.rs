//! Tests of the SSH protocol on whole documents. They resolve nodes and
//! credentials through the core (`Document::find`, `resolve_credential`).

use portolano_core::{CredentialSpec, SecretRef};
use pretty_assertions::assert_eq;
use serde_json::json;

use super::*;
use crate::plan::{AskpassEntry, LaunchStep, PromptMatcher};
use crate::protocol::{ConnectOptions, SessionCredential};
use crate::testutil::{connection_node, credential_node, document, folder, plan_with, run_args};

fn strings(words: &[&str]) -> Vec<String> {
    words.iter().map(|word| (*word).to_owned()).collect()
}

fn askpass_entries(plan: &LaunchPlan) -> Vec<AskpassEntry> {
    match plan.steps.last() {
        Some(LaunchStep::Run {
            askpass: Some(spec),
            ..
        }) => spec.entries.clone(),
        _ => Vec::new(),
    }
}

fn password_entry(user: &str, host: &str, secret: &SecretRef) -> AskpassEntry {
    AskpassEntry {
        matcher: PromptMatcher::Password {
            user: user.into(),
            host: host.into(),
        },
        secret: secret.clone(),
    }
}

#[test]
fn password_of_a_referenced_credential_goes_to_askpass() {
    let password = SecretRef::new();
    let cred = credential_node("Admin", "admin", Some(&password), None);
    let conn = connection_node(
        "ssh",
        "web01.example.com",
        CredentialSpec::ById { id: cred.id },
        json!({}),
    );
    let conn_id = conn.id;
    let doc = document(vec![cred, conn]);

    let plan = plan_with(&SshProtocol, &doc, conn_id, &ConnectOptions::default()).unwrap();

    assert_eq!(
        run_args(&plan),
        strings(&[
            "-p",
            "22",
            "-l",
            "admin",
            "-o",
            "NumberOfPasswordPrompts=1",
            "--",
            "web01.example.com"
        ])
    );
    assert_eq!(
        askpass_entries(&plan),
        vec![password_entry("admin", "web01.example.com", &password)]
    );
}

#[test]
fn credential_is_inherited_from_the_folder() {
    let password = SecretRef::new();
    let cred = credential_node("Ops", "ops", Some(&password), None);
    let conn = connection_node(
        "ssh",
        "db01.example.com",
        CredentialSpec::Inherit,
        json!({}),
    );
    let conn_id = conn.id;
    let doc = document(vec![
        cred.clone(),
        folder(
            "Clienti",
            CredentialSpec::ById { id: cred.id },
            vec![folder("ACME", CredentialSpec::Inherit, vec![conn])],
        ),
    ]);

    let plan = plan_with(&SshProtocol, &doc, conn_id, &ConnectOptions::default()).unwrap();

    assert_eq!(
        askpass_entries(&plan),
        vec![password_entry("ops", "db01.example.com", &password)]
    );
}

#[test]
fn jump_chain_is_flattened_with_the_jumps_of_a_jump_first() {
    let jump_password = SecretRef::new();
    let jump_cred = credential_node("Jump", "jumper", Some(&jump_password), None);
    let outer = connection_node(
        "ssh",
        "outer.example.com",
        CredentialSpec::None,
        json!({ "jump_hosts": [{ "address": "edge.example.com:2222" }] }),
    );
    let mut inner = connection_node(
        "ssh",
        "inner.example.com",
        CredentialSpec::ById { id: jump_cred.id },
        json!({ "jump_hosts": [{ "$ref": outer.id }] }),
    );
    inner.as_connection_mut().unwrap().port = Some(2200);
    let target = connection_node(
        "ssh",
        "web01.example.com",
        CredentialSpec::None,
        json!({ "jump_hosts": [{ "$ref": inner.id }, { "address": "last@198.51.100.7" }] }),
    );
    let target_id = target.id;
    let doc = document(vec![jump_cred, outer, inner, target]);

    let plan = plan_with(&SshProtocol, &doc, target_id, &ConnectOptions::default()).unwrap();

    let args = run_args(&plan);
    let j = args.iter().position(|arg| arg == "-J").unwrap();
    assert_eq!(
        args[j + 1],
        "edge.example.com:2222,outer.example.com:22,jumper@inner.example.com:2200,last@198.51.100.7"
    );
    assert_eq!(
        askpass_entries(&plan),
        vec![password_entry(
            "jumper",
            "inner.example.com",
            &jump_password
        )]
    );
}

#[test]
fn jump_cycle_is_an_error() {
    let mut first = connection_node("ssh", "a.example.com", CredentialSpec::None, json!({}));
    let second = connection_node(
        "ssh",
        "b.example.com",
        CredentialSpec::None,
        json!({ "jump_hosts": [{ "$ref": first.id }] }),
    );
    first.as_connection_mut().unwrap().settings = json!({ "jump_hosts": [{ "$ref": second.id }] })
        .as_object()
        .unwrap()
        .clone();
    let first_id = first.id;
    let doc = document(vec![first, second]);

    let result = plan_with(&SshProtocol, &doc, first_id, &ConnectOptions::default());

    assert!(
        matches!(result, Err(ConnectError::JumpCycle { id }) if id == first_id),
        "{result:?}"
    );
}

#[test]
fn jump_to_an_rdp_connection_is_invalid() {
    let rdp = connection_node("rdp", "dc01.example.com", CredentialSpec::None, json!({}));
    let target = connection_node(
        "ssh",
        "web01.example.com",
        CredentialSpec::None,
        json!({ "jump_hosts": [{ "$ref": rdp.id }] }),
    );
    let target_id = target.id;
    let doc = document(vec![rdp, target]);

    let result = plan_with(&SshProtocol, &doc, target_id, &ConnectOptions::default());

    assert!(
        matches!(result, Err(ConnectError::InvalidJumpHost { .. })),
        "{result:?}"
    );
}

#[test]
fn validate_reports_a_jump_cycle() {
    let mut node = connection_node("ssh", "loop.example.com", CredentialSpec::None, json!({}));
    node.as_connection_mut().unwrap().settings = json!({ "jump_hosts": [{ "$ref": node.id }] })
        .as_object()
        .unwrap()
        .clone();
    let doc = document(vec![node.clone()]);

    let issues = SshProtocol.validate(&doc, &node);

    assert_eq!(
        issues.iter().map(|issue| issue.code).collect::<Vec<_>>(),
        ["jump_cycle"]
    );
}

#[test]
fn new_window_relaunches_portolano_on_the_node() {
    let conn = connection_node("ssh", "web01.example.com", CredentialSpec::None, json!({}));
    let conn_id = conn.id;
    let doc = document(vec![conn]);
    let options = ConnectOptions {
        new_window: true,
        relaunch_args: strings(&["--vault", "file"]),
        ..ConnectOptions::default()
    };

    let plan = plan_with(&SshProtocol, &doc, conn_id, &options).unwrap();

    assert_eq!(
        plan.steps,
        vec![LaunchStep::OpenInTerminal {
            argv: strings(&[
                "/usr/local/bin/portolano",
                "--vault",
                "file",
                "connect",
                &format!("@{conn_id}")
            ]),
        }]
    );
}

#[test]
fn prompt_credential_with_session_login_acts_as_inline() {
    let password = SecretRef::new();
    let conn = connection_node(
        "ssh",
        "web01.example.com",
        CredentialSpec::Prompt,
        json!({}),
    );
    let conn_id = conn.id;
    let doc = document(vec![conn]);
    let options = ConnectOptions {
        session_credential: Some(SessionCredential {
            username: Some("mario".into()),
            domain: None,
            password: Some(password.clone()),
        }),
        ..ConnectOptions::default()
    };

    let plan = plan_with(&SshProtocol, &doc, conn_id, &options).unwrap();

    assert!(
        run_args(&plan)
            .windows(2)
            .any(|pair| pair == ["-l", "mario"])
    );
    assert_eq!(
        askpass_entries(&plan),
        vec![password_entry("mario", "web01.example.com", &password)]
    );
    assert!(plan.warnings.is_empty());
}

#[test]
fn prompt_credential_without_session_login_leaves_it_to_ssh() {
    let conn = connection_node(
        "ssh",
        "web01.example.com",
        CredentialSpec::Prompt,
        json!({}),
    );
    let conn_id = conn.id;
    let doc = document(vec![conn]);

    let plan = plan_with(&SshProtocol, &doc, conn_id, &ConnectOptions::default()).unwrap();

    assert_eq!(
        run_args(&plan),
        strings(&["-p", "22", "--", "web01.example.com"])
    );
    assert_eq!(plan.warnings.len(), 1);
}

#[test]
fn summary_shows_user_host_and_port() {
    let cred = credential_node("Admin", "admin", None, None);
    let conn = connection_node(
        "ssh",
        "web01.example.com",
        CredentialSpec::ById { id: cred.id },
        json!({}),
    );
    let doc = document(vec![cred, conn.clone()]);

    let summary = SshProtocol.summary(&doc, &conn);

    assert_eq!(summary, "admin@web01.example.com:22");
}

#[test]
fn plan_of_a_folder_is_not_a_connection() {
    let node = folder("Clienti", CredentialSpec::Inherit, vec![]);
    let id = node.id;
    let doc = document(vec![node]);

    let result = plan_with(&SshProtocol, &doc, id, &ConnectOptions::default());

    assert!(
        matches!(result, Err(ConnectError::NotAConnection { .. })),
        "{result:?}"
    );
}
