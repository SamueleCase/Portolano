//! The size limit of [`IconStore::add`] and [`IconStore::list`].

use pretty_assertions::assert_eq;

use super::*;

/// A PNG signature padded to `len` bytes.
fn png_of_len(len: usize) -> Vec<u8> {
    let mut bytes = PNG.to_vec();
    bytes.resize(len, 0);
    bytes
}

#[test]
fn image_at_the_size_limit_is_stored() {
    let dir = TempDir::new().unwrap();
    let limit = usize::try_from(MAX_ICON_BYTES).unwrap();

    let icon = icon_store(&dir).add(&png_of_len(limit), "image/png");

    assert!(icon.is_ok(), "{icon:?}");
}

#[test]
fn image_over_the_size_limit_is_refused_and_not_stored() {
    let dir = TempDir::new().unwrap();
    let limit = usize::try_from(MAX_ICON_BYTES).unwrap();

    let result = icon_store(&dir).add(&png_of_len(limit + 1), "image/png");

    assert!(matches!(
        result,
        Err(Error::IconTooLarge { size, max }) if size == MAX_ICON_BYTES + 1 && max == MAX_ICON_BYTES
    ));
    assert!(!icons_dir(&dir).exists());
}

#[test]
fn list_of_a_store_without_a_directory_is_empty() {
    let dir = TempDir::new().unwrap();

    assert_eq!(icon_store(&dir).list().unwrap(), Vec::new());
}

#[test]
fn list_returns_the_stored_images_ordered_by_hash() {
    let dir = TempDir::new().unwrap();
    let store = icon_store(&dir);
    let png = store.add(PNG, "image/png").unwrap();
    let gif = store.add(GIF, "image/gif").unwrap();
    let mut expected = vec![png, gif];
    expected.sort_by_key(|icon| sha256_of(icon).to_owned());

    assert_eq!(store.list().unwrap(), expected);
}

#[test]
fn list_leaves_out_files_the_store_did_not_write() {
    let dir = TempDir::new().unwrap();
    let store = icon_store(&dir);
    let png = store.add(PNG, "image/png").unwrap();
    fs::write(icons_dir(&dir).join("notes.txt"), "not an icon").unwrap();
    fs::write(
        icons_dir(&dir).join(format!("{}.bmp", "ab".repeat(32))),
        "x",
    )
    .unwrap();

    assert_eq!(store.list().unwrap(), vec![png]);
}

#[cfg(unix)]
#[test]
fn list_leaves_out_symlinks_named_as_stored_images() {
    let dir = TempDir::new().unwrap();
    let store = icon_store(&dir);
    let png = store.add(PNG, "image/png").unwrap();
    let target = dir.path().join("elsewhere.gif");
    fs::write(&target, GIF).unwrap();
    std::os::unix::fs::symlink(
        &target,
        icons_dir(&dir).join(format!("{}.gif", "cd".repeat(32))),
    )
    .unwrap();

    assert_eq!(store.list().unwrap(), vec![png]);
}
