//! SPEC §7.4: no argv, environment or temporary file of any plan, nor its
//! rendering, contains the value of a secret. The secrets hold a sentinel in
//! an in-memory vault; every kind of plan is built and inspected.

use portolano_core::vault::MemoryVault;
use portolano_core::{CredentialSpec, PrivateKey, SecretRef, Vault};
use serde_json::json;

use crate::protocol::{ConnectOptions, Protocol};
use crate::rdp::{RdpClient, RdpProtocol};
use crate::ssh::SshProtocol;
use crate::testutil::{
    SENTINEL, assert_no_secret, connection_node, credential_node, document, plan_with, vault_with,
};

#[test]
fn no_plan_exposes_a_secret_of_the_vault() {
    let password = SecretRef::new();
    let key = SecretRef::new();
    let passphrase = SecretRef::new();
    let vault = vault_with(&[
        (&password, SENTINEL),
        (&key, SENTINEL),
        (&passphrase, SENTINEL),
    ]);

    let with_password = credential_node("Password", "admin", Some(&password), None);
    let mut with_vault_key = credential_node(
        "Vault key",
        "deploy",
        None,
        Some(PrivateKey::Vault {
            secret: key.clone(),
        }),
    );
    with_vault_key.as_credential_mut().unwrap().passphrase = Some(passphrase.clone());
    let mut with_key_file = credential_node(
        "Key file",
        "deploy",
        None,
        Some(PrivateKey::File {
            path: "/keys/id".into(),
        }),
    );
    with_key_file.as_credential_mut().unwrap().passphrase = Some(passphrase.clone());
    let by_id = |node: &portolano_core::Node| CredentialSpec::ById { id: node.id };

    let jump = connection_node("ssh", "gw.example.com", by_id(&with_password), json!({}));
    let ssh_password =
        connection_node("ssh", "web01.example.com", by_id(&with_password), json!({}));
    let ssh_vault_key = connection_node(
        "ssh",
        "web02.example.com",
        by_id(&with_vault_key),
        json!({}),
    );
    let ssh_key_file =
        connection_node("ssh", "web03.example.com", by_id(&with_key_file), json!({}));
    let ssh_jump = connection_node(
        "ssh",
        "web04.example.com",
        by_id(&with_password),
        json!({ "jump_hosts": [{ "$ref": jump.id }] }),
    );
    let rdp_gateway = connection_node(
        "rdp",
        "dc01.example.com",
        by_id(&with_password),
        json!({ "gateway": { "host": "rdgw.example.com", "credential": { "mode": "by_id", "$ref": with_password.id } } }),
    );
    let ssh_ids = [
        ssh_password.id,
        ssh_vault_key.id,
        ssh_key_file.id,
        ssh_jump.id,
    ];
    let rdp_id = rdp_gateway.id;
    let doc = document(vec![
        with_password,
        with_vault_key,
        with_key_file,
        jump,
        ssh_password,
        ssh_vault_key,
        ssh_key_file,
        ssh_jump,
        rdp_gateway,
    ]);
    let option_sets = [
        ConnectOptions::default(),
        ConnectOptions {
            new_window: true,
            ..ConnectOptions::default()
        },
        ConnectOptions {
            rdp_client: Some(RdpClient::WindowsApp),
            ..ConnectOptions::default()
        },
    ];

    let mut checked = 0;
    for options in &option_sets {
        let targets = ssh_ids
            .iter()
            .map(|id| (&SshProtocol as &dyn Protocol, *id))
            .chain([(&RdpProtocol as &dyn Protocol, rdp_id)]);
        for (protocol, id) in targets {
            let plan = plan_with(protocol, &doc, id, options).unwrap();
            assert_no_secret(&plan, SENTINEL);
            checked += 1;
        }
    }

    assert_eq!(checked, 15);
    assert!(vault.contains(&password).unwrap());
}

/// The plans are executed for real, with `ssh` and FreeRDP replaced by a
/// probe script that records its argv, its environment and its stdin.
#[cfg(unix)]
mod runtime {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;

    use portolano_core::{AppConfig, Document, Node, NodeKind};
    use pretty_assertions::assert_eq;
    use tempfile::TempDir;
    use uuid::Uuid;

    use super::*;
    use crate::env::Environment;
    use crate::protocol::PlanContext;
    use crate::testutil::environment;

    /// Appends argv and environment to `seen.txt`, next to itself. It reads
    /// stdin into `stdin.txt` only when asked to (`/from-stdin:force`, where
    /// stdin is a pipe written by `execute`): otherwise stdin is the one of
    /// `cargo test`, possibly a terminal that never ends.
    const PROBE: &str = "#!/bin/sh\n\
        dir=$(dirname \"$0\")\n\
        { printf '%s\\n' \"$@\"; env; } >> \"$dir/seen.txt\"\n\
        case \" $* \" in *' /from-stdin:force '*) cat >> \"$dir/stdin.txt\" ;; esac\n";

    fn probe_environment(dir: &TempDir) -> Environment {
        let probe = dir.path().join("probe");
        fs::write(&probe, PROBE).unwrap();
        fs::set_permissions(&probe, fs::Permissions::from_mode(0o755)).unwrap();
        Environment {
            ssh: Some(probe.clone()),
            freerdp: Some(probe.clone()),
            self_exe: probe,
            ..environment()
        }
    }

    fn run(
        doc: &Document,
        node_id: Uuid,
        protocol: &dyn Protocol,
        env: &Environment,
        vault: &MemoryVault,
    ) {
        let config = AppConfig::default();
        let options = ConnectOptions {
            wait: true,
            ..ConnectOptions::default()
        };
        let context = PlanContext {
            doc,
            node_id,
            config: &config,
            env,
            options: &options,
        };
        let plan = protocol.plan(&context).unwrap();

        let code = crate::execute(&plan, vault, &config).unwrap();

        assert_eq!(code, 0);
    }

    fn read(dir: &Path, name: &str) -> String {
        fs::read_to_string(dir.join(name)).unwrap_or_default()
    }

    fn with_passphrase(mut node: Node, passphrase: &SecretRef) -> Node {
        if let NodeKind::Credential(credential) = &mut node.kind {
            credential.passphrase = Some(passphrase.clone());
        }
        node
    }

    #[test]
    fn ssh_with_password_jump_and_key_passphrase_never_sees_the_secret() {
        let dir = tempfile::tempdir().unwrap();
        let env = probe_environment(&dir);
        let password = SecretRef::new();
        let passphrase = SecretRef::new();
        let vault = vault_with(&[(&password, SENTINEL), (&passphrase, SENTINEL)]);
        let jump_cred = credential_node("Jump", "jumper", Some(&password), None);
        let key = PrivateKey::File {
            path: "/keys/id_test".into(),
        };
        let target_cred = with_passphrase(
            credential_node("Target", "admin", Some(&password), Some(key)),
            &passphrase,
        );
        let jump = connection_node(
            "ssh",
            "gw.example.com",
            CredentialSpec::ById { id: jump_cred.id },
            json!({}),
        );
        let target = connection_node(
            "ssh",
            "web01.example.com",
            CredentialSpec::ById { id: target_cred.id },
            json!({ "jump_hosts": [{ "$ref": jump.id }] }),
        );
        let target_id = target.id;
        let doc = document(vec![jump_cred, target_cred, jump, target]);

        run(&doc, target_id, &SshProtocol, &env, &vault);

        let seen = read(dir.path(), "seen.txt");
        assert!(seen.contains("web01.example.com"));
        assert!(seen.contains("SSH_ASKPASS_REQUIRE=force"));
        assert!(!seen.contains(SENTINEL));
        assert!(!dir.path().join("stdin.txt").exists());
    }

    #[test]
    fn freerdp_gets_the_password_only_on_stdin() {
        let dir = tempfile::tempdir().unwrap();
        let env = probe_environment(&dir);
        let password = SecretRef::new();
        let vault = vault_with(&[(&password, SENTINEL)]);
        let cred = credential_node("Admin", "administrator", Some(&password), None);
        let conn = connection_node(
            "rdp",
            "dc01.example.com",
            CredentialSpec::ById { id: cred.id },
            json!({ "gateway": { "host": "rdgw.example.com", "credential": { "mode": "by_id", "$ref": cred.id } } }),
        );
        let conn_id = conn.id;
        let doc = document(vec![cred, conn]);

        run(&doc, conn_id, &RdpProtocol, &env, &vault);

        let seen = read(dir.path(), "seen.txt");
        assert!(seen.contains("/v:dc01.example.com:3389"));
        assert!(!seen.contains(SENTINEL));
        assert_eq!(read(dir.path(), "stdin.txt"), format!("{SENTINEL}\n"));
    }
}
