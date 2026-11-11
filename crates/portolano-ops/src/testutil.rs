//! Fixtures shared by the unit tests of the crate (SPEC §11.3). Hosts are
//! under `example.com` and users are fake (SPEC §1.3).

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use portolano_core::vault::{FileVault, KdfParams, MemoryVault};
use portolano_core::{
    CredentialSpec, Document, MasterPasswordSource, Node, Secret, SecretRef, Store, Vault,
    VaultError,
};
use tempfile::TempDir;
use uuid::Uuid;

/// The only master password allowed in tests (SPEC §1.3).
pub(crate) const TEST_MASTER_PASSWORD: &str = "test-only-master-password";

/// The cheapest Argon2id costs, so that file vaults open quickly.
const TEST_KDF_PARAMS: KdfParams = KdfParams {
    m_cost_kib: 8,
    t_cost: 1,
    p_cost: 1,
};

/// Always answers [`TEST_MASTER_PASSWORD`].
pub(crate) struct TestPassword;

impl MasterPasswordSource for TestPassword {
    fn master_password(&self, _creating: bool) -> Result<Secret, VaultError> {
        Ok(Secret::new(TEST_MASTER_PASSWORD.to_owned()))
    }
}

/// A sentinel secret value: tests check that it never leaks.
pub(crate) const SENTINEL: &str = "sentinel-ops-secret-7f3a";

/// A folder with the given children.
pub(crate) fn folder(name: &str, children: Vec<Node>) -> Node {
    let mut node = Node::folder(name);
    if let Some(folder) = node.as_folder_mut() {
        folder.children = children;
    }
    node
}

/// A connection of `protocol` to `<name>.example.com`.
pub(crate) fn connection(name: &str, protocol: &str) -> Node {
    Node::connection(name, protocol, format!("{name}.example.com"))
}

/// A connection with the given credential spec.
pub(crate) fn connection_with(name: &str, protocol: &str, spec: CredentialSpec) -> Node {
    let mut node = connection(name, protocol);
    if let Some(connection) = node.as_connection_mut() {
        connection.credential = spec;
    }
    node
}

/// A credential with a user name and, if given, a password reference.
pub(crate) fn credential(name: &str, username: &str, password: Option<SecretRef>) -> Node {
    let mut node = Node::credential(name);
    if let Some(credential) = node.as_credential_mut() {
        credential.username = Some(username.to_owned());
        credential.password = password;
    }
    node
}

/// A document with the given top-level nodes.
pub(crate) fn document(nodes: Vec<Node>) -> Document {
    let mut doc = Document::new("Test");
    doc.nodes = nodes;
    doc
}

/// A document saved in a temporary directory, with its store.
pub(crate) struct Stored {
    pub(crate) dir: TempDir,
    pub(crate) store: Store,
    pub(crate) doc: Document,
}

impl Stored {
    /// The document file.
    pub(crate) fn path(&self) -> PathBuf {
        self.dir.path().join("doc.json")
    }

    /// The bytes of the document file.
    pub(crate) fn file_text(&self) -> String {
        std::fs::read_to_string(self.path()).unwrap()
    }

    /// The document as saved on disk.
    pub(crate) fn reloaded(&self) -> Document {
        Store::open(self.path()).unwrap().load().unwrap()
    }

    /// Makes the next save of [`Stored::store`] fail with
    /// `ModifiedExternally`, by saving through another store.
    pub(crate) fn modify_externally(&self) {
        let other = Store::open(self.path()).unwrap();
        let mut doc = other.load().unwrap();
        doc.name = "cambiato altrove".to_owned();
        other.save(&mut doc).unwrap();
    }
}

/// Saves `nodes` as a new document in a temporary directory.
pub(crate) fn stored(nodes: Vec<Node>) -> Stored {
    let dir = tempfile::tempdir().unwrap();
    let (store, mut doc) = Store::create(dir.path().join("doc.json"), "Test").unwrap();
    doc.nodes = nodes;
    store.save(&mut doc).unwrap();
    Stored { dir, store, doc }
}

/// A memory vault holding `value` under a new reference.
pub(crate) fn vault_with(value: &str) -> (MemoryVault, SecretRef) {
    let vault = MemoryVault::default();
    let reference = SecretRef::new();
    vault
        .set(&reference, &Secret::new(value.to_owned()))
        .unwrap();
    (vault, reference)
}

/// A secret with the given value.
pub(crate) fn secret(value: &str) -> Secret {
    Secret::new(value.to_owned())
}

/// The id of the node at `path`.
pub(crate) fn id_of(doc: &Document, path: &str) -> Uuid {
    doc.resolve_path(path).unwrap()
}

/// Creates an empty file vault with cheap KDF costs in `dir` and returns its
/// path; reopening it uses the costs stored in the file.
pub(crate) fn fast_file_vault(dir: &Path) -> PathBuf {
    let path = dir.join("vault.json");
    FileVault::with_kdf_params(&path, &TestPassword, TEST_KDF_PARAMS).unwrap();
    path
}

/// A memory vault that records the references written and deleted, in
/// order, to check rollbacks.
#[derive(Default)]
pub(crate) struct RecordingVault {
    inner: MemoryVault,
    history: Mutex<(Vec<SecretRef>, Vec<SecretRef>)>,
}

impl RecordingVault {
    /// The references written and those deleted.
    pub(crate) fn history(&self) -> (Vec<SecretRef>, Vec<SecretRef>) {
        self.history.lock().unwrap().clone()
    }
}

impl Vault for RecordingVault {
    fn backend(&self) -> &'static str {
        "memory"
    }

    fn get(&self, reference: &SecretRef) -> Result<Option<Secret>, VaultError> {
        self.inner.get(reference)
    }

    fn set(&self, reference: &SecretRef, secret: &Secret) -> Result<(), VaultError> {
        self.history.lock().unwrap().0.push(reference.clone());
        self.inner.set(reference, secret)
    }

    fn delete(&self, reference: &SecretRef) -> Result<(), VaultError> {
        self.history.lock().unwrap().1.push(reference.clone());
        self.inner.delete(reference)
    }
}
