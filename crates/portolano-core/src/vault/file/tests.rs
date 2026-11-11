use std::fs;
use std::path::{Path, PathBuf};

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use pretty_assertions::assert_eq;
use serde_json::Value;
use tempfile::TempDir;

use super::*;
use crate::vault::testutil::{FixedPassword, NoPassword, TEST_KDF_PARAMS};

mod shared;

const SENTINEL: &str = "sentinel-file-vault-secret";

/// A temporary directory and the path of a vault file inside it; the
/// directory must outlive the test.
fn vault_path() -> (TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("vault.json");
    (dir, path)
}

fn open_test_vault(path: &Path) -> FileVault {
    FileVault::with_kdf_params(path, &FixedPassword::test(), TEST_KDF_PARAMS).unwrap()
}

fn sentinel() -> Secret {
    Secret::new(SENTINEL.to_owned())
}

fn exposed(secret: Option<Secret>) -> Option<String> {
    secret.map(|secret| secret.expose().to_owned())
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn write_json(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_vec(value).unwrap()).unwrap();
}

#[test]
fn creating_a_vault_writes_the_file_and_asks_for_a_new_password() {
    let (_dir, path) = vault_path();
    let password = FixedPassword::test();

    let vault = FileVault::with_kdf_params(&path, &password, TEST_KDF_PARAMS).unwrap();

    assert_eq!(vault.backend(), "file");
    assert!(path.is_file());
    assert_eq!(password.last_creating.get(), Some(true));
}

#[test]
fn file_follows_the_documented_layout() {
    let (_dir, path) = vault_path();

    open_test_vault(&path);

    let json = read_json(&path);
    assert_eq!(json["format"], "portolano-vault");
    assert_eq!(json["version"], 1);
    assert_eq!(json["kdf"]["alg"], "argon2id");
    assert_eq!(json["kdf"]["m_cost_kib"], TEST_KDF_PARAMS.m_cost_kib);
    assert_eq!(json["kdf"]["t_cost"], TEST_KDF_PARAMS.t_cost);
    assert_eq!(json["kdf"]["p_cost"], TEST_KDF_PARAMS.p_cost);
    assert!(BASE64.decode(json["kdf"]["salt"].as_str().unwrap()).is_ok());
    assert_eq!(json["cipher"], "xchacha20poly1305");
    let nonce = BASE64.decode(json["nonce"].as_str().unwrap()).unwrap();
    assert_eq!(nonce.len(), 24);
    assert!(BASE64.decode(json["ciphertext"].as_str().unwrap()).is_ok());
}

#[test]
fn get_returns_the_secret_that_was_set() {
    let (_dir, path) = vault_path();
    let vault = open_test_vault(&path);
    let reference = SecretRef::new();

    vault.set(&reference, &sentinel()).unwrap();

    assert_eq!(
        exposed(vault.get(&reference).unwrap()),
        Some(SENTINEL.to_owned())
    );
}

#[test]
fn get_of_a_missing_entry_is_none() {
    let (_dir, path) = vault_path();
    let vault = open_test_vault(&path);

    assert!(vault.get(&SecretRef::new()).unwrap().is_none());
}

#[test]
fn reopening_with_the_same_password_restores_the_secrets() {
    let (_dir, path) = vault_path();
    let reference = SecretRef::new();
    open_test_vault(&path).set(&reference, &sentinel()).unwrap();
    let password = FixedPassword::test();

    let reopened = FileVault::with_kdf_params(&path, &password, TEST_KDF_PARAMS).unwrap();

    assert_eq!(password.last_creating.get(), Some(false));
    assert_eq!(
        exposed(reopened.get(&reference).unwrap()),
        Some(SENTINEL.to_owned())
    );
}

#[test]
fn reopening_uses_the_kdf_params_stored_in_the_file() {
    let (_dir, path) = vault_path();
    let reference = SecretRef::new();
    open_test_vault(&path).set(&reference, &sentinel()).unwrap();

    // `open` would create a new vault with the recommended costs; an
    // existing one must be derived with the costs recorded in its file.
    let reopened = FileVault::open(&path, &FixedPassword::test()).unwrap();
    reopened.set(&SecretRef::new(), &sentinel()).unwrap();

    assert!(reopened.contains(&reference).unwrap());
    assert_eq!(
        read_json(&path)["kdf"]["m_cost_kib"],
        TEST_KDF_PARAMS.m_cost_kib
    );
}

#[test]
fn delete_removes_the_entry_from_the_file() {
    let (_dir, path) = vault_path();
    let vault = open_test_vault(&path);
    let reference = SecretRef::new();
    vault.set(&reference, &sentinel()).unwrap();

    vault.delete(&reference).unwrap();

    assert!(!vault.contains(&reference).unwrap());
    assert!(!open_test_vault(&path).contains(&reference).unwrap());
}

#[test]
fn delete_of_a_missing_entry_succeeds() {
    let (_dir, path) = vault_path();
    let vault = open_test_vault(&path);

    assert!(vault.delete(&SecretRef::new()).is_ok());
}

#[test]
fn wrong_master_password_is_rejected() {
    let (_dir, path) = vault_path();
    open_test_vault(&path)
        .set(&SecretRef::new(), &sentinel())
        .unwrap();

    let result = FileVault::with_kdf_params(
        &path,
        &FixedPassword::new("not-the-test-master-password"),
        TEST_KDF_PARAMS,
    );

    assert!(matches!(result, Err(VaultError::BadMasterPassword)));
}

#[test]
fn tampered_ciphertext_is_rejected() {
    let (_dir, path) = vault_path();
    open_test_vault(&path)
        .set(&SecretRef::new(), &sentinel())
        .unwrap();
    let mut json = read_json(&path);
    let mut ciphertext = BASE64.decode(json["ciphertext"].as_str().unwrap()).unwrap();
    ciphertext[0] ^= 0x01;
    json["ciphertext"] = Value::String(BASE64.encode(&ciphertext));
    write_json(&path, &json);

    let result = FileVault::with_kdf_params(&path, &FixedPassword::test(), TEST_KDF_PARAMS);

    assert!(matches!(result, Err(VaultError::BadMasterPassword)));
}

#[test]
fn file_that_is_not_json_is_corrupt_and_no_password_is_asked() {
    let (_dir, path) = vault_path();
    fs::write(&path, b"not a vault").unwrap();

    let result = FileVault::with_kdf_params(&path, &NoPassword, TEST_KDF_PARAMS);

    assert!(matches!(result, Err(VaultError::Corrupt { .. })));
}

#[test]
fn nonce_of_the_wrong_length_is_corrupt() {
    let (_dir, path) = vault_path();
    open_test_vault(&path);
    let mut json = read_json(&path);
    json["nonce"] = Value::String(BASE64.encode([0u8; 12]));
    write_json(&path, &json);

    let result = FileVault::with_kdf_params(&path, &FixedPassword::test(), TEST_KDF_PARAMS);

    assert!(matches!(result, Err(VaultError::Corrupt { .. })));
}

/// Creates a vault and overwrites one KDF cost in its file.
fn vault_with_kdf_cost(field: &str, value: u32) -> (TempDir, PathBuf) {
    let (dir, path) = vault_path();
    open_test_vault(&path);
    let mut json = read_json(&path);
    json["kdf"][field] = Value::from(value);
    write_json(&path, &json);
    (dir, path)
}

#[test]
fn memory_cost_above_the_limit_is_corrupt() {
    let (_dir, path) = vault_with_kdf_cost("m_cost_kib", 1024 * 1024 + 1);

    let result = FileVault::with_kdf_params(&path, &NoPassword, TEST_KDF_PARAMS);

    assert!(matches!(result, Err(VaultError::Corrupt { .. })));
}

#[test]
fn time_cost_above_the_limit_is_corrupt() {
    let (_dir, path) = vault_with_kdf_cost("t_cost", 17);

    let result = FileVault::with_kdf_params(&path, &NoPassword, TEST_KDF_PARAMS);

    assert!(matches!(result, Err(VaultError::Corrupt { .. })));
}

#[test]
fn parallelism_above_the_limit_is_corrupt() {
    let (_dir, path) = vault_with_kdf_cost("p_cost", 17);

    let result = FileVault::with_kdf_params(&path, &NoPassword, TEST_KDF_PARAMS);

    assert!(matches!(result, Err(VaultError::Corrupt { .. })));
}

#[test]
fn creating_with_costs_above_the_limit_fails_before_asking_the_password() {
    let (_dir, path) = vault_path();
    let params = KdfParams {
        t_cost: 17,
        ..TEST_KDF_PARAMS
    };

    let result = FileVault::with_kdf_params(&path, &NoPassword, params);

    assert!(matches!(result, Err(VaultError::Crypto { .. })));
    assert!(!path.exists());
}

#[test]
fn time_cost_at_the_limit_is_accepted() {
    let (_dir, path) = vault_with_kdf_cost("t_cost", 16);

    let result = FileVault::with_kdf_params(&path, &FixedPassword::test(), TEST_KDF_PARAMS);

    // The file passes validation; the changed cost derives another key, so
    // only decryption fails.
    assert!(matches!(result, Err(VaultError::BadMasterPassword)));
}

#[test]
fn newer_version_is_unsupported() {
    let (_dir, path) = vault_path();
    write_json(
        &path,
        &serde_json::json!({ "format": "portolano-vault", "version": 2 }),
    );

    let result = FileVault::with_kdf_params(&path, &NoPassword, TEST_KDF_PARAMS);

    assert!(matches!(
        result,
        Err(VaultError::UnsupportedVersion {
            found: 2,
            supported: 1,
            ..
        })
    ));
}

#[test]
fn each_write_uses_a_fresh_nonce() {
    let (_dir, path) = vault_path();
    let vault = open_test_vault(&path);
    let reference = SecretRef::new();
    vault.set(&reference, &sentinel()).unwrap();
    let first = read_json(&path)["nonce"].clone();

    vault.set(&reference, &sentinel()).unwrap();

    assert_ne!(read_json(&path)["nonce"], first);
}

#[test]
fn salt_stays_the_same_across_writes() {
    let (_dir, path) = vault_path();
    let vault = open_test_vault(&path);
    let salt = read_json(&path)["kdf"]["salt"].clone();

    vault.set(&SecretRef::new(), &sentinel()).unwrap();

    assert_eq!(read_json(&path)["kdf"]["salt"], salt);
}

#[test]
fn file_does_not_contain_the_secret_in_clear() {
    let (_dir, path) = vault_path();
    let vault = open_test_vault(&path);

    vault.set(&SecretRef::new(), &sentinel()).unwrap();

    let bytes = fs::read(&path).unwrap();
    let contains = |needle: &[u8]| bytes.windows(needle.len()).any(|window| window == needle);
    assert!(!contains(SENTINEL.as_bytes()));
    assert!(!contains(BASE64.encode(SENTINEL).as_bytes()));
}

#[test]
fn only_the_vault_and_its_lock_file_are_left_after_a_write() {
    let (dir, path) = vault_path();
    let vault = open_test_vault(&path);

    vault.set(&SecretRef::new(), &sentinel()).unwrap();

    let mut names: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    assert_eq!(names, vec!["vault.json", "vault.json.lock"]);
}

#[cfg(unix)]
mod unix {
    use std::os::unix::fs::PermissionsExt;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn file_is_readable_only_by_the_owner() {
        let (_dir, path) = vault_path();
        let vault = open_test_vault(&path);

        vault.set(&SecretRef::new(), &sentinel()).unwrap();

        let mode = fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn failed_write_leaves_the_vault_unchanged() {
        let (dir, path) = vault_path();
        let vault = open_test_vault(&path);
        let reference = SecretRef::new();
        let read_only = fs::Permissions::from_mode(0o500);
        fs::set_permissions(dir.path(), read_only).unwrap();

        let result = vault.set(&reference, &sentinel());

        fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o700)).unwrap();
        assert!(matches!(result, Err(VaultError::Io { .. })));
        assert!(vault.get(&reference).unwrap().is_none());
    }
}
