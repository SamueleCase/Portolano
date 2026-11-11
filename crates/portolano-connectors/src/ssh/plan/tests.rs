//! Tests of the SSH plan builder on resolved data: they need no document.

use std::collections::BTreeMap;

use portolano_core::SecretRef;
use pretty_assertions::assert_eq;

use super::*;
use crate::testutil::{login, run_args};

fn endpoint(host: &str, port: u16, credential: ResolvedCredential) -> Endpoint {
    Endpoint {
        node: uuid::Uuid::new_v4(),
        host: host.into(),
        port,
        credential,
    }
}

fn launch<'a>(target: Endpoint, jumps: Vec<Jump>, settings: &'a SshSettings) -> SshLaunch<'a> {
    SshLaunch {
        ssh: PathBuf::from("/usr/bin/ssh"),
        ssh_add_available: true,
        askpass_program: PathBuf::from("/usr/local/bin/portolano"),
        target,
        jumps,
        settings,
    }
}

fn strings(words: &[&str]) -> Vec<String> {
    words.iter().map(|word| (*word).to_owned()).collect()
}

fn askpass_entries(plan: &LaunchPlan) -> Vec<AskpassEntry> {
    plan.steps
        .iter()
        .find_map(|step| match step {
            LaunchStep::Run { askpass, .. } => askpass.clone(),
            _ => None,
        })
        .map(|spec| spec.entries)
        .unwrap_or_default()
}

#[test]
fn password_is_served_by_askpass_with_a_single_prompt() {
    let password = SecretRef::new();
    let settings = SshSettings::default();
    let target = endpoint(
        "web01.example.com",
        22,
        login(Some("admin"), Some(&password)),
    );

    let plan = build(&launch(target, vec![], &settings)).unwrap();

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
        vec![AskpassEntry {
            matcher: PromptMatcher::Password {
                user: "admin".into(),
                host: "web01.example.com".into(),
            },
            secret: password,
        }]
    );
}

#[test]
fn without_secrets_there_is_no_askpass() {
    let settings = SshSettings::default();
    let target = endpoint("web01.example.com", 2222, login(Some("admin"), None));

    let plan = build(&launch(target, vec![], &settings)).unwrap();

    assert_eq!(
        run_args(&plan),
        strings(&["-p", "2222", "-l", "admin", "--", "web01.example.com"])
    );
    assert!(askpass_entries(&plan).is_empty());
}

#[test]
fn key_file_is_passed_with_identities_only_and_its_passphrase_via_askpass() {
    let passphrase = SecretRef::new();
    let settings = SshSettings::default();
    let mut credential = login(Some("deploy"), None);
    credential.private_key = Some(PrivateKey::File {
        path: "/keys/id_ed25519".into(),
    });
    credential.passphrase = Some(passphrase.clone());

    let plan = build(&launch(
        endpoint("192.0.2.10", 22, credential),
        vec![],
        &settings,
    ))
    .unwrap();

    assert_eq!(
        run_args(&plan),
        strings(&[
            "-p",
            "22",
            "-l",
            "deploy",
            "-i",
            "/keys/id_ed25519",
            "-o",
            "IdentitiesOnly=yes",
            "-o",
            "NumberOfPasswordPrompts=1",
            "--",
            "192.0.2.10",
        ])
    );
    assert_eq!(
        askpass_entries(&plan),
        vec![AskpassEntry {
            matcher: PromptMatcher::Passphrase {
                key_path: "/keys/id_ed25519".into(),
            },
            secret: passphrase,
        }]
    );
}

#[test]
fn key_file_path_with_tilde_is_expanded() {
    let settings = SshSettings::default();
    let mut credential = login(Some("deploy"), None);
    credential.private_key = Some(PrivateKey::File {
        path: "~/.ssh/id_test".into(),
    });
    let expected = std::env::home_dir().unwrap().join(".ssh/id_test");

    let plan = build(&launch(
        endpoint("192.0.2.10", 22, credential),
        vec![],
        &settings,
    ))
    .unwrap();

    assert!(run_args(&plan).contains(&expected.to_string_lossy().into_owned()));
}

#[test]
fn vault_key_adds_an_ssh_add_step_before_ssh() {
    let key = SecretRef::new();
    let passphrase = SecretRef::new();
    let settings = SshSettings::default();
    let mut credential = login(Some("deploy"), None);
    credential.private_key = Some(PrivateKey::Vault {
        secret: key.clone(),
    });
    credential.passphrase = Some(passphrase.clone());

    let plan = build(&launch(
        endpoint("192.0.2.10", 22, credential),
        vec![],
        &settings,
    ))
    .unwrap();

    assert_eq!(
        plan.steps[0],
        LaunchStep::SshAddKey {
            key,
            passphrase: Some(passphrase),
            lifetime_s: 300,
            askpass_program: PathBuf::from("/usr/local/bin/portolano"),
        }
    );
    assert_eq!(
        run_args(&plan),
        strings(&["-p", "22", "-l", "deploy", "--", "192.0.2.10"])
    );
    assert_eq!(plan.steps.len(), 2);
}

#[test]
fn vault_key_without_ssh_add_is_an_error() {
    let settings = SshSettings::default();
    let mut credential = login(Some("deploy"), None);
    credential.private_key = Some(PrivateKey::Vault {
        secret: SecretRef::new(),
    });
    let mut input = launch(endpoint("192.0.2.10", 22, credential), vec![], &settings);
    input.ssh_add_available = false;

    let result = build(&input);

    assert!(
        matches!(result, Err(ConnectError::ClientNotFound { ref client, .. }) if client == "ssh-add"),
        "{result:?}"
    );
}

#[test]
fn jumps_become_one_j_option_and_their_passwords_are_served() {
    let jump_password = SecretRef::new();
    let settings = SshSettings::default();
    let jumps = vec![
        Jump::Address("ops@bastion.example.com:2222".into()),
        Jump::Endpoint(endpoint(
            "gw.example.com",
            22,
            login(Some("jumper"), Some(&jump_password)),
        )),
        Jump::Endpoint(endpoint("2001:db8::1", 2200, login(None, None))),
    ];
    let target = endpoint("web01.example.com", 22, login(Some("admin"), None));

    let plan = build(&launch(target, jumps, &settings)).unwrap();

    assert_eq!(
        run_args(&plan),
        strings(&[
            "-p",
            "22",
            "-l",
            "admin",
            "-J",
            "ops@bastion.example.com:2222,jumper@gw.example.com:22,[2001:db8::1]:2200",
            "-o",
            "NumberOfPasswordPrompts=1",
            "--",
            "web01.example.com",
        ])
    );
    assert_eq!(askpass_entries(&plan)[0].secret, jump_password);
}

#[test]
fn jump_key_file_is_reported_as_not_forwarded() {
    let settings = SshSettings::default();
    let mut credential = login(Some("jumper"), None);
    credential.private_key = Some(PrivateKey::File {
        path: "/keys/jump".into(),
    });
    let jumps = vec![Jump::Endpoint(endpoint("gw.example.com", 22, credential))];
    let target = endpoint("web01.example.com", 22, login(Some("admin"), None));

    let plan = build(&launch(target, jumps, &settings)).unwrap();

    assert!(!run_args(&plan).contains(&"/keys/jump".to_owned()));
    assert_eq!(plan.warnings.len(), 1);
}

#[test]
fn settings_flags_come_in_spec_order_before_the_host() {
    let settings = SshSettings {
        agent_forwarding: true,
        x11: true,
        x11_trusted: true,
        compression: true,
        keepalive_s: Some(30),
        options: BTreeMap::from([("StrictHostKeyChecking".into(), "yes".into())]),
        ..SshSettings::default()
    };
    let target = endpoint("web01.example.com", 22, login(None, None));

    let plan = build(&launch(target, vec![], &settings)).unwrap();

    assert_eq!(
        run_args(&plan),
        strings(&[
            "-p",
            "22",
            "-A",
            "-Y",
            "-C",
            "-o",
            "ServerAliveInterval=30",
            "-o",
            "StrictHostKeyChecking=yes",
            "--",
            "web01.example.com",
        ])
    );
}

#[test]
fn password_without_user_is_not_served_and_warns() {
    let settings = SshSettings::default();
    let target = endpoint(
        "web01.example.com",
        22,
        login(None, Some(&SecretRef::new())),
    );

    let plan = build(&launch(target, vec![], &settings)).unwrap();

    assert!(askpass_entries(&plan).is_empty());
    assert_eq!(plan.warnings.len(), 1);
}

#[test]
fn ssh_runs_in_the_foreground_without_stdin_secret() {
    let settings = SshSettings::default();
    let target = endpoint(
        "web01.example.com",
        22,
        login(Some("admin"), Some(&SecretRef::new())),
    );

    let plan = build(&launch(target, vec![], &settings)).unwrap();

    assert!(matches!(
        plan.steps.last(),
        Some(LaunchStep::Run {
            wait: true,
            stdin_secret: None,
            ..
        })
    ));
}

#[test]
fn jump_host_with_a_comma_is_rejected() {
    let settings = SshSettings::default();
    let jumps = vec![Jump::Endpoint(endpoint(
        "gw.example.com,evil.example.com",
        22,
        login(None, None),
    ))];
    let target = endpoint("web01.example.com", 22, login(None, None));

    let result = build(&launch(target, jumps, &settings));

    assert!(
        matches!(result, Err(ConnectError::InvalidValue { ref field, .. }) if field == "host"),
        "{result:?}"
    );
}

#[test]
fn jump_user_with_a_space_or_control_character_is_rejected() {
    let settings = SshSettings::default();
    for user in ["ops admin", "ops\nadmin"] {
        let jumps = vec![Jump::Endpoint(endpoint(
            "gw.example.com",
            22,
            login(Some(user), None),
        ))];
        let target = endpoint("web01.example.com", 22, login(None, None));

        let result = build(&launch(target, jumps, &settings));

        assert!(
            matches!(result, Err(ConnectError::InvalidValue { ref field, .. }) if field == "username"),
            "{result:?}"
        );
    }
}

#[test]
fn askpass_program_comes_from_the_launch() {
    let settings = SshSettings::default();
    let target = endpoint(
        "web01.example.com",
        22,
        login(Some("admin"), Some(&SecretRef::new())),
    );

    let plan = build(&launch(target, vec![], &settings)).unwrap();

    let Some(LaunchStep::Run {
        askpass: Some(spec),
        ..
    }) = plan.steps.last()
    else {
        panic!("expected askpass");
    };
    assert_eq!(spec.program, PathBuf::from("/usr/local/bin/portolano"));
}
