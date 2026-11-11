//! Plans of whole documents, resolved through the core.

use pretty_assertions::assert_eq;
use serde_json::json;

use super::*;
use crate::protocol::{ConnectOptions, SessionCredential};
use crate::testutil::{connection_node, credential_node, document, plan_with};

fn program(plan: &LaunchPlan) -> PathBuf {
    match plan.steps.last() {
        Some(LaunchStep::Run { program, .. }) => program.clone(),
        other => panic!("expected a run step, got {other:?}"),
    }
}

#[test]
fn connection_client_overrides_the_configuration_and_option_overrides_both() {
    let conn = connection_node(
        "rdp",
        "dc01.example.com",
        CredentialSpec::None,
        json!({ "client": "windows-app" }),
    );
    let conn_id = conn.id;
    let doc = document(vec![conn]);
    let forced = ConnectOptions {
        rdp_client: Some(RdpClient::FreeRdp),
        ..ConnectOptions::default()
    };

    let by_setting = plan_with(&RdpProtocol, &doc, conn_id, &ConnectOptions::default()).unwrap();
    let by_option = plan_with(&RdpProtocol, &doc, conn_id, &forced).unwrap();

    assert_eq!(program(&by_setting), PathBuf::from("/usr/bin/open"));
    assert_eq!(
        program(&by_option),
        PathBuf::from("/opt/homebrew/bin/sdl-freerdp")
    );
}

#[test]
fn gateway_credential_is_resolved_by_reference() {
    let gw_cred = credential_node("Gateway", "gwuser", None, None);
    let conn = connection_node(
        "rdp",
        "dc01.example.com",
        CredentialSpec::None,
        json!({ "gateway": { "host": "gw.example.com", "credential": { "mode": "by_id", "$ref": gw_cred.id } } }),
    );
    let conn_id = conn.id;
    let doc = document(vec![gw_cred, conn]);

    let plan = plan_with(&RdpProtocol, &doc, conn_id, &ConnectOptions::default()).unwrap();

    assert!(run_args(&plan).contains(&"/gateway:g:gw.example.com,u:gwuser".to_owned()));
}

#[test]
fn prompt_credential_with_session_login_acts_as_inline() {
    let password = SecretRef::new();
    let conn = connection_node("rdp", "dc01.example.com", CredentialSpec::Prompt, json!({}));
    let conn_id = conn.id;
    let doc = document(vec![conn]);
    let options = ConnectOptions {
        session_credential: Some(SessionCredential {
            username: Some("mario".into()),
            domain: Some("ACME".into()),
            password: Some(password.clone()),
        }),
        wait: true,
        ..ConnectOptions::default()
    };

    let plan = plan_with(&RdpProtocol, &doc, conn_id, &options).unwrap();

    let args = run_args(&plan);
    assert!(args.contains(&"/u:mario".to_owned()) && args.contains(&"/d:ACME".to_owned()));
    assert!(
        matches!(&plan.steps[0], LaunchStep::Run { stdin_secret: Some(secret), .. } if *secret == password)
    );
    assert!(plan.warnings.is_empty());
}

#[test]
fn prompt_credential_without_session_login_leaves_it_to_freerdp() {
    let conn = connection_node("rdp", "dc01.example.com", CredentialSpec::Prompt, json!({}));
    let conn_id = conn.id;
    let doc = document(vec![conn]);
    let options = ConnectOptions {
        wait: true,
        ..ConnectOptions::default()
    };

    let plan = plan_with(&RdpProtocol, &doc, conn_id, &options).unwrap();

    let args = run_args(&plan);
    assert!(
        args.iter()
            .all(|arg| !arg.starts_with("/u:") && arg != "/from-stdin:force")
    );
    assert_eq!(plan.warnings.len(), 1);
}

#[test]
fn summary_shows_domain_user_and_host() {
    let mut cred = credential_node("Admin", "administrator", None, None);
    cred.as_credential_mut().unwrap().domain = Some("ACME".into());
    let conn = connection_node(
        "rdp",
        "192.0.2.10",
        CredentialSpec::ById { id: cred.id },
        json!({}),
    );
    let doc = document(vec![cred, conn.clone()]);

    let summary = RdpProtocol.summary(&doc, &conn);

    assert_eq!(summary, "ACME\\administrator@192.0.2.10");
}

#[test]
fn unknown_node_is_a_core_error() {
    let doc = document(vec![]);

    let result = plan_with(
        &RdpProtocol,
        &doc,
        Uuid::new_v4(),
        &ConnectOptions::default(),
    );

    assert!(matches!(result, Err(ConnectError::Core(_))), "{result:?}");
}
