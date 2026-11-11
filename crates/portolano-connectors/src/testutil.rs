//! Builders and fixtures shared by the tests of the crate. Only example hosts
//! (RFC 2606, RFC 5737) and fake users (SPEC §1.3).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use portolano_core::{
    AppConfig, CredentialOrigin, CredentialSpec, Document, Node, NodeKind, PrivateKey,
    ResolvedCredential, Secret, SecretRef, Vault, VaultError,
};
use serde_json::Value;
use uuid::Uuid;

use crate::env::{Environment, Os};
use crate::plan::{LaunchPlan, LaunchStep};
use crate::protocol::{ConnectOptions, PlanContext};

/// A secret value that must never appear in argv, env or output.
pub(crate) const SENTINEL: &str = "sentinel-secret-7f3a9c";

/// An environment with every client installed.
pub(crate) fn environment() -> Environment {
    Environment {
        os: Os::MacOs,
        ssh: Some(PathBuf::from("/usr/bin/ssh")),
        ssh_add: Some(PathBuf::from("/usr/bin/ssh-add")),
        freerdp: Some(PathBuf::from("/opt/homebrew/bin/sdl-freerdp")),
        windows_app: Some(PathBuf::from("/Applications/Windows App.app")),
        self_exe: PathBuf::from("/usr/local/bin/portolano"),
    }
}

/// Inline login data with optional user and password.
pub(crate) fn login(username: Option<&str>, password: Option<&SecretRef>) -> ResolvedCredential {
    ResolvedCredential {
        origin: CredentialOrigin::Inline { node: Uuid::nil() },
        username: username.map(str::to_owned),
        domain: None,
        password: password.cloned(),
        private_key: None,
        passphrase: None,
    }
}

/// A credential node.
pub(crate) fn credential_node(
    name: &str,
    username: &str,
    password: Option<&SecretRef>,
    private_key: Option<PrivateKey>,
) -> Node {
    let mut node = Node::credential(name);
    if let NodeKind::Credential(credential) = &mut node.kind {
        credential.username = Some(username.to_owned());
        credential.password = password.cloned();
        credential.private_key = private_key;
    }
    node
}

/// A connection node with the given credential spec and settings.
pub(crate) fn connection_node(
    protocol: &str,
    host: &str,
    credential: CredentialSpec,
    settings: Value,
) -> Node {
    let mut node = Node::connection(host, protocol, host);
    if let NodeKind::Connection(connection) = &mut node.kind {
        connection.credential = credential;
        if let Value::Object(settings) = settings {
            connection.settings = settings;
        }
    }
    node
}

/// A folder with the given credential spec and children.
pub(crate) fn folder(name: &str, credential: CredentialSpec, children: Vec<Node>) -> Node {
    let mut node = Node::folder(name);
    if let NodeKind::Folder(folder) = &mut node.kind {
        folder.credential = credential;
        folder.children = children;
    }
    node
}

/// A document whose root holds `nodes`.
pub(crate) fn document(nodes: Vec<Node>) -> Document {
    let mut doc = Document::new("Test");
    doc.nodes = nodes;
    doc
}

/// Plans `node_id` with `protocol`, the default configuration and the full
/// environment.
pub(crate) fn plan_with(
    protocol: &dyn crate::Protocol,
    doc: &Document,
    node_id: Uuid,
    options: &ConnectOptions,
) -> Result<LaunchPlan, crate::ConnectError> {
    let config = AppConfig::default();
    let env = environment();
    protocol.plan(&PlanContext {
        doc,
        node_id,
        config: &config,
        env: &env,
        options,
    })
}

/// The arguments of the last `Run` step of `plan`.
pub(crate) fn run_args(plan: &LaunchPlan) -> Vec<String> {
    plan.steps
        .iter()
        .rev()
        .find_map(|step| match step {
            LaunchStep::Run { args, .. } => Some(args.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

/// Asserts that no argv, env or temporary file of `plan`, nor its rendering,
/// contains `secret`.
pub(crate) fn assert_no_secret(plan: &LaunchPlan, secret: &str) {
    let rendered = crate::render(plan);
    assert!(!rendered.contains(secret), "secret in render: {rendered}");
    for step in &plan.steps {
        let texts: Vec<&str> = match step {
            LaunchStep::Run { args, env, .. } => args
                .iter()
                .map(String::as_str)
                .chain(env.iter().flat_map(|(k, v)| [k.as_str(), v.as_str()]))
                .collect(),
            LaunchStep::WriteTempFile { contents, .. } => vec![contents.as_str()],
            LaunchStep::OpenInTerminal { argv } => argv.iter().map(String::as_str).collect(),
            LaunchStep::SshAddKey { .. } => vec![],
        };
        assert!(
            texts.iter().all(|text| !text.contains(secret)),
            "secret in {step:?}"
        );
    }
}

/// A vault in memory for the tests of this crate.
#[derive(Default)]
pub(crate) struct TestVault {
    entries: Mutex<HashMap<SecretRef, String>>,
}

impl TestVault {
    /// A vault with one entry per `(reference, value)`.
    pub(crate) fn with(entries: &[(&SecretRef, &str)]) -> TestVault {
        let vault = TestVault::default();
        for (reference, value) in entries {
            vault
                .set(reference, &Secret::new((*value).to_owned()))
                .unwrap();
        }
        vault
    }
}

impl Vault for TestVault {
    fn backend(&self) -> &'static str {
        "memory"
    }

    fn get(&self, reference: &SecretRef) -> Result<Option<Secret>, VaultError> {
        let entries = self.entries.lock().unwrap();
        Ok(entries
            .get(reference)
            .map(|value| Secret::new(value.clone())))
    }

    fn set(&self, reference: &SecretRef, secret: &Secret) -> Result<(), VaultError> {
        let mut entries = self.entries.lock().unwrap();
        entries.insert(reference.clone(), secret.expose().to_owned());
        Ok(())
    }

    fn delete(&self, reference: &SecretRef) -> Result<(), VaultError> {
        self.entries.lock().unwrap().remove(reference);
        Ok(())
    }
}
