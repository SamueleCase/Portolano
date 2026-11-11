//! SPEC §7.4: no argv, environment or temporary file of any plan, nor its
//! rendering, contains the value of a secret. The secrets hold a sentinel in
//! an in-memory vault; every kind of plan is built and inspected.

use portolano_core::{CredentialSpec, PrivateKey, SecretRef, Vault};
use serde_json::json;

use crate::protocol::{ConnectOptions, Protocol};
use crate::rdp::{RdpClient, RdpProtocol};
use crate::ssh::SshProtocol;
use crate::testutil::{
    SENTINEL, TestVault, assert_no_secret, connection_node, credential_node, document, plan_with,
};

#[test]
fn no_plan_exposes_a_secret_of_the_vault() {
    let password = SecretRef::new();
    let key = SecretRef::new();
    let passphrase = SecretRef::new();
    let vault = TestVault::with(&[
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
