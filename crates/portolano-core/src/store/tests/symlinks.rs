use pretty_assertions::assert_eq;

use super::*;

/// A document created in `repo/` and a symlink to it, `link.json`, outside.
fn symlinked_document(dir: &TempDir) -> (PathBuf, PathBuf) {
    let target = dir.path().join("repo").join("portolano.json");
    Store::create(&target, "Portolano").unwrap();
    let link = dir.path().join("link.json");
    std::os::unix::fs::symlink(&target, &link).unwrap();
    (target, link)
}

#[test]
fn save_through_a_symlink_keeps_the_link_and_updates_its_target() {
    let dir = TempDir::new().unwrap();
    let (target, link) = symlinked_document(&dir);
    let store = Store::open(&link).unwrap();
    let mut doc: Document = serde_json::from_slice(&fs::read(&target).unwrap()).unwrap();
    doc.name = "Rinominato".to_owned();

    store.save(&mut doc).unwrap();

    assert!(
        fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    let on_disk: Document = serde_json::from_slice(&fs::read(&target).unwrap()).unwrap();
    assert_eq!(on_disk.name, "Rinominato");
}

#[test]
fn save_through_a_symlink_writes_backup_and_lock_next_to_the_target() {
    let dir = TempDir::new().unwrap();
    let (target, link) = symlinked_document(&dir);
    let store = Store::open(&link).unwrap();
    let mut doc: Document = serde_json::from_slice(&fs::read(&target).unwrap()).unwrap();

    store.save(&mut doc).unwrap();

    assert!(dir.path().join("repo").join("portolano.json.bak").is_file());
    assert!(
        dir.path()
            .join("repo")
            .join("portolano.json.lock")
            .is_file()
    );
    assert!(!dir.path().join("link.json.bak").exists());
    assert!(!dir.path().join("link.json.lock").exists());
}
