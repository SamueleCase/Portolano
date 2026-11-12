//! Fixtures of the tests of the commands: an app state over a temporary
//! directory, with a file vault protected by the test master password
//! (SPEC §1.3) and cheap key derivation. Hosts are under `example.com`.

use std::fs;
use std::path::PathBuf;

use portolano_core::vault::{FileVault, KdfParams};
use portolano_core::{MasterPasswordSource, Secret, VaultError};
use tempfile::TempDir;

use crate::dto::{DocumentInfo, NewNodeInput};
use crate::state::{AppPaths, AppState};

/// The only master password allowed in tests (SPEC §1.3).
pub const TEST_MASTER_PASSWORD: &str = "test-only-master-password";

/// A sentinel secret: the tests check that it never leaves the vault.
pub const SENTINEL: &str = "sentinel-app-secret-4b1d";

/// The cheapest Argon2id costs, stored in the vault file.
const TEST_KDF_PARAMS: KdfParams = KdfParams {
    m_cost_kib: 8,
    t_cost: 1,
    p_cost: 1,
};

struct TestPassword;

impl MasterPasswordSource for TestPassword {
    fn master_password(&self, _creating: bool) -> Result<Secret, VaultError> {
        Ok(Secret::new(TEST_MASTER_PASSWORD.to_owned()))
    }
}

/// An app state over a temporary directory.
pub struct TestApp {
    /// The directory of every file of the app.
    pub dir: TempDir,
    /// The state.
    pub state: AppState,
}

impl TestApp {
    /// An app whose configuration names a document and a file vault in the
    /// directory; the vault exists and is locked, no document is open.
    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let vault = dir.path().join("vault.json");
        FileVault::with_kdf_params(&vault, &TestPassword, TEST_KDF_PARAMS).unwrap();
        let config = dir.path().join("config.toml");
        fs::write(
            &config,
            format!(
                "default_document = {:?}\n[vault]\nbackend = \"file\"\nfile = {:?}\n",
                dir.path().join("portolano.json").display().to_string(),
                vault.display().to_string(),
            ),
        )
        .unwrap();
        let state = AppState::new(AppPaths {
            config,
            ui_preferences: dir.path().join("ui.toml"),
        });
        Self { dir, state }
    }

    /// An app with an empty default document open and the vault unlocked.
    pub fn with_document() -> Self {
        let app = Self::new();
        app.create_default();
        app.unlock();
        app
    }

    /// Creates the default document and opens it.
    pub fn create_default(&self) -> DocumentInfo {
        super::document::create(&self.state, &self.document_path(), "Test").unwrap()
    }

    /// The default document.
    pub fn document_path(&self) -> PathBuf {
        self.dir.path().join("portolano.json")
    }

    /// Unlocks the vault with the test master password.
    pub fn unlock(&self) {
        super::vault::unlock(&self.state, Secret::new(TEST_MASTER_PASSWORD.to_owned())).unwrap();
    }

    /// Adds the node described by `json` (the JSON the frontend sends) and
    /// returns its id.
    pub fn add(&self, json: serde_json::Value) -> uuid::Uuid {
        let input: NewNodeInput = serde_json::from_value(json).unwrap();
        super::nodes::create(&self.state, input).unwrap().id
    }

    /// The bytes of every file the app wrote in its directory, as text.
    pub fn files_text(&self) -> String {
        let mut text = String::new();
        let mut dirs = vec![self.dir.path().to_owned()];
        while let Some(dir) = dirs.pop() {
            for entry in fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    dirs.push(path);
                } else {
                    text.push_str(&String::from_utf8_lossy(&fs::read(path).unwrap()));
                }
            }
        }
        text
    }
}

/// A credential `/Admin` with the sentinel as password, as the frontend
/// sends it.
pub fn admin_with_sentinel() -> serde_json::Value {
    serde_json::json!({
        "parent": null,
        "name": "Admin",
        "kind": { "type": "credential", "username": "admin", "domain": null, "keyFile": null },
        "secrets": { "password": SENTINEL },
    })
}

/// An SSH connection `name` to `<name>.example.com` that inherits its
/// credential.
pub fn ssh(name: &str, parent: Option<uuid::Uuid>) -> serde_json::Value {
    serde_json::json!({
        "parent": parent,
        "name": name,
        "kind": {
            "type": "connection",
            "protocol": "ssh",
            "host": format!("{name}.example.com"),
            "port": null,
            "credential": { "mode": "inherit" },
            "settings": {},
        },
    })
}
