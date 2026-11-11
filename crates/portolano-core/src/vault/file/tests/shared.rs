//! Several `FileVault` instances on the same file, as with the app and the
//! CLI open together (DEC-017). The lock is per open file, so instances in
//! one process exclude each other exactly like separate processes.

use std::thread;

use pretty_assertions::assert_eq;

use super::*;

#[test]
fn secrets_set_by_two_instances_are_both_kept() {
    let (_dir, path) = vault_path();
    let first = open_test_vault(&path);
    let second = open_test_vault(&path);
    let (one, two) = (SecretRef::new(), SecretRef::new());

    first.set(&one, &sentinel()).unwrap();
    second.set(&two, &sentinel()).unwrap();

    for vault in [&first, &second] {
        assert!(vault.contains(&one).unwrap());
        assert!(vault.contains(&two).unwrap());
    }
}

#[test]
fn delete_by_one_instance_is_seen_by_the_other() {
    let (_dir, path) = vault_path();
    let first = open_test_vault(&path);
    let second = open_test_vault(&path);
    let reference = SecretRef::new();
    first.set(&reference, &sentinel()).unwrap();

    second.delete(&reference).unwrap();

    assert!(first.get(&reference).unwrap().is_none());
}

#[test]
fn concurrent_writers_lose_no_secret() {
    let (_dir, path) = vault_path();
    open_test_vault(&path);
    let writers: Vec<_> = (0..8)
        .map(|_| {
            let path = path.clone();
            thread::spawn(move || {
                let vault = open_test_vault(&path);
                (0..4)
                    .map(|_| {
                        let reference = SecretRef::new();
                        vault.set(&reference, &sentinel()).unwrap();
                        reference
                    })
                    .collect::<Vec<_>>()
            })
        })
        .collect();

    let written: Vec<SecretRef> = writers
        .into_iter()
        .flat_map(|writer| writer.join().unwrap())
        .collect();

    let vault = open_test_vault(&path);
    let missing = written
        .iter()
        .filter(|reference| !vault.contains(reference).unwrap())
        .count();
    assert_eq!(missing, 0);
}

#[test]
fn vault_removed_after_opening_is_reported() {
    let (_dir, path) = vault_path();
    let vault = open_test_vault(&path);
    fs::remove_file(&path).unwrap();

    let result = vault.get(&SecretRef::new());

    assert!(matches!(
        result,
        Err(VaultError::Backend {
            backend: "file",
            ..
        })
    ));
}

#[test]
fn vault_replaced_after_opening_is_reported() {
    let (_dir, path) = vault_path();
    let vault = open_test_vault(&path);
    fs::remove_file(&path).unwrap();
    // A new vault at the same path has a new salt, hence another key.
    open_test_vault(&path);

    let result = vault.set(&SecretRef::new(), &sentinel());

    assert!(matches!(
        result,
        Err(VaultError::Backend {
            backend: "file",
            ..
        })
    ));
}

/// Plays another process that creates the vault while this one is asking
/// for a new master password: the window between the check and the write.
struct CreatedDuringPrompt<'a> {
    path: &'a Path,
    /// A secret the other process stores in its vault.
    other_reference: &'a SecretRef,
}

impl MasterPasswordSource for CreatedDuringPrompt<'_> {
    fn master_password(&self, creating: bool) -> Result<Secret, VaultError> {
        if creating {
            open_test_vault(self.path)
                .set(self.other_reference, &sentinel())
                .unwrap();
        }
        FixedPassword::test().master_password(creating)
    }
}

#[test]
fn vault_created_by_another_process_during_creation_is_reported_and_kept() {
    let (_dir, path) = vault_path();
    let other_reference = SecretRef::new();
    let source = CreatedDuringPrompt {
        path: &path,
        other_reference: &other_reference,
    };

    let result = FileVault::with_kdf_params(&path, &source, TEST_KDF_PARAMS);

    assert!(matches!(
        result,
        Err(VaultError::Backend {
            backend: "file",
            ..
        })
    ));
    assert!(open_test_vault(&path).contains(&other_reference).unwrap());
}
