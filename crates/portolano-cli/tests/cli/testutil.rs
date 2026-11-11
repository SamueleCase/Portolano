//! A temporary workspace with a document, a file vault and a configuration.
//! Hosts are under `example.com` and users are fake (SPEC §1.3).

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use assert_cmd::Command;
use assert_cmd::assert::Assert;
use portolano_core::vault::{FileVault, KdfParams};
use portolano_core::{MasterPasswordSource, Secret, VaultError};
use serde_json::Value;
use tempfile::TempDir;

/// The only master password allowed in tests (SPEC §1.3).
pub const TEST_MASTER_PASSWORD: &str = "test-only-master-password";

/// A secret value that must never appear in output or files.
pub const SENTINEL: &str = "sentinel-cli-secret-91c4";

/// The cheapest Argon2id costs: the vault is created with them before the
/// tests, and every run of the binary reads them from the file (DEC-022).
const TEST_KDF_PARAMS: KdfParams = KdfParams {
    m_cost_kib: 8,
    t_cost: 1,
    p_cost: 1,
};

/// No test may wait for a terminal.
const TIMEOUT: Duration = Duration::from_secs(60);

struct TestPassword;

impl MasterPasswordSource for TestPassword {
    fn master_password(&self, _creating: bool) -> Result<Secret, VaultError> {
        Ok(Secret::new(TEST_MASTER_PASSWORD.to_owned()))
    }
}

/// A temporary directory with `doc.json`, `vault.json` and `config.toml`.
pub struct Workspace {
    pub dir: TempDir,
}

impl Workspace {
    /// A workspace with a fast file vault and no document.
    pub fn empty() -> Workspace {
        let dir = tempfile::tempdir().unwrap();
        FileVault::with_kdf_params(
            dir.path().join("vault.json"),
            &TestPassword,
            TEST_KDF_PARAMS,
        )
        .unwrap();
        Workspace { dir }
    }

    /// A workspace with an empty document.
    pub fn new() -> Workspace {
        let workspace = Workspace::empty();
        workspace.ok(&["init"]);
        workspace
    }

    pub fn path(&self, name: &str) -> PathBuf {
        self.dir.path().join(name)
    }

    pub fn doc(&self) -> PathBuf {
        self.path("doc.json")
    }

    pub fn doc_text(&self) -> String {
        fs::read_to_string(self.doc()).unwrap()
    }

    /// The binary with the workspace's document, vault and configuration.
    pub fn cmd(&self) -> Command {
        let mut cmd = self.bare_cmd();
        cmd.args(["--vault", "file", "--vault-file"])
            .arg(self.path("vault.json"));
        cmd
    }

    /// The binary with the workspace's document and configuration, and no
    /// vault option.
    pub fn bare_cmd(&self) -> Command {
        let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("portolano");
        for name in [
            "PORTOLANO_FILE",
            "PORTOLANO_VAULT",
            "PORTOLANO_VAULT_FILE",
            "PORTOLANO_ASKPASS",
        ] {
            cmd.env_remove(name);
        }
        cmd.env("PORTOLANO_VAULT_PASSWORD", TEST_MASTER_PASSWORD)
            .arg("--file")
            .arg(self.doc())
            .arg("--config")
            .arg(self.path("config.toml"))
            .write_stdin("")
            .timeout(TIMEOUT);
        cmd
    }

    /// The binary run from the workspace directory, with the document and
    /// the vault given by environment variables as relative paths.
    pub fn env_cmd(&self) -> Command {
        let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("portolano");
        cmd.current_dir(self.dir.path())
            .env_remove("PORTOLANO_ASKPASS")
            .env("PORTOLANO_FILE", "doc.json")
            .env("PORTOLANO_VAULT", "file")
            .env("PORTOLANO_VAULT_FILE", "vault.json")
            .env("PORTOLANO_VAULT_PASSWORD", TEST_MASTER_PASSWORD)
            .args(["--config", "config.toml"])
            .write_stdin("")
            .timeout(TIMEOUT);
        cmd
    }

    pub fn run(&self, args: &[&str]) -> Assert {
        self.cmd().args(args).assert()
    }

    /// Runs with `stdin` as input.
    pub fn run_with_stdin(&self, args: &[&str], stdin: &str) -> Assert {
        self.cmd().args(args).write_stdin(stdin).assert()
    }

    /// Runs and expects success; returns stdout.
    pub fn ok(&self, args: &[&str]) -> String {
        stdout(self.run(args).success())
    }

    /// Writes an executable script in the workspace and returns its path.
    pub fn script(&self, name: &str, body: &str) -> PathBuf {
        let path = self.path(name);
        fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    pub fn write_config(&self, text: &str) {
        fs::write(self.path("config.toml"), text).unwrap();
    }

    /// A credential `/Admin` with user `admin` and the sentinel password.
    pub fn add_admin(&self) {
        self.run_with_stdin(
            &[
                "add",
                "cred",
                "/Admin",
                "--user",
                "admin",
                "--password-stdin",
            ],
            &format!("{SENTINEL}\n"),
        )
        .success();
    }
}

/// A document with folders, a credential and two connections.
pub fn sample() -> Workspace {
    let workspace = Workspace::new();
    workspace.ok(&["add", "folder", "-p", "/Clienti/ACME"]);
    workspace.add_admin();
    workspace.ok(&[
        "add",
        "ssh",
        "/Clienti/ACME/web01",
        "--host",
        "web01.example.com",
        "--cred",
        "/Admin",
    ]);
    workspace.ok(&[
        "add",
        "rdp",
        "/Clienti/ACME/dc01",
        "--host",
        "192.0.2.10",
        "--user",
        "administrator",
        "--domain",
        "ACME",
    ]);
    workspace
}

pub fn stdout(assert: Assert) -> String {
    String::from_utf8(assert.get_output().stdout.clone()).unwrap()
}

pub fn stderr(assert: &Assert) -> String {
    String::from_utf8(assert.get_output().stderr.clone()).unwrap()
}

/// Everything a run printed.
pub fn all_output(assert: &Assert) -> String {
    let output = assert.get_output();
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// The model of the node at `path`.
pub fn node(workspace: &Workspace, path: &str) -> Value {
    let text = workspace.ok(&["show", "--json", path]);
    serde_json::from_str::<Value>(&text).unwrap()["node"].clone()
}

/// The names of the children of `path`, in order.
pub fn names(workspace: &Workspace, path: &str) -> Vec<String> {
    let text = workspace.ok(&["list", "--json", path]);
    let entries: Value = serde_json::from_str(&text).unwrap();
    entries
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| entry["depth"] == 0)
        .map(|entry| entry["name"].as_str().unwrap().to_owned())
        .collect()
}

pub fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../portolano-import/tests/fixtures")
        .join(name)
}
