use pretty_assertions::assert_eq;
use tempfile::TempDir;

use super::*;
use crate::model::Node;
use crate::store::Store;

const PNG: &[u8] = b"\x89PNG\r\n\x1a\nnot really pixels";
const GIF: &[u8] = b"GIF89a-not-really-pixels";

fn icon_store(dir: &TempDir) -> IconStore {
    IconStore::for_document(&dir.path().join("portolano.json"))
}

/// The icon store of an empty document saved in `dir`, and its store.
fn saved_document(dir: &TempDir) -> (Store, IconStore) {
    let (store, _) = Store::create(dir.path().join("portolano.json"), "Portolano").unwrap();
    let icons = store.icons();
    (store, icons)
}

fn icons_dir(dir: &TempDir) -> PathBuf {
    dir.path().join("portolano.icons")
}

fn stored_files(dir: &TempDir) -> usize {
    fs::read_dir(icons_dir(dir)).unwrap().count()
}

/// Moves the modification time of `path` an hour back, past the grace
/// period of `gc`.
fn make_old(path: &Path) {
    let an_hour_ago = SystemTime::now() - Duration::from_secs(60 * 60);
    fs::File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_modified(an_hour_ago)
        .unwrap();
}

fn modified(path: &Path) -> SystemTime {
    fs::metadata(path).unwrap().modified().unwrap()
}

/// Adds an image and makes its file old.
fn add_old(store: &IconStore, bytes: &[u8]) -> Icon {
    let icon = store.add(bytes, "application/octet-stream").unwrap();
    make_old(&store.path_for(&icon).unwrap());
    icon
}

fn custom(sha256: &str, mime: &str) -> Icon {
    Icon::Custom {
        sha256: sha256.to_owned(),
        mime: mime.to_owned(),
    }
}

/// A document whose only icon, `icon`, is on a connection inside a folder.
fn document_using(icon: Icon) -> Document {
    let mut connection = Node::connection("web01", "ssh", "web01.example.com");
    connection.icon = Some(icon);
    let mut folder = Node::folder("Clienti");
    folder.as_folder_mut().unwrap().children.push(connection);
    let mut doc = Document::new("Portolano");
    doc.nodes.push(folder);
    doc
}

#[test]
fn add_returns_the_hash_and_type_of_the_image() {
    let dir = TempDir::new().unwrap();

    let icon = icon_store(&dir).add(PNG, "image/png").unwrap();

    let expected = hex::encode(Sha256::digest(PNG));
    assert_eq!(icon, custom(&expected, "image/png"));
}

#[test]
fn add_writes_the_bytes_under_their_hash() {
    let dir = TempDir::new().unwrap();
    let store = icon_store(&dir);

    let icon = store.add(PNG, "image/png").unwrap();

    let path = store.path_for(&icon).unwrap();
    assert_eq!(path.parent(), Some(icons_dir(&dir).as_path()));
    assert_eq!(fs::read(path).unwrap(), PNG);
}

#[test]
fn every_accepted_type_is_recognized_from_its_magic_bytes() {
    let samples: [(&[u8], &str, &str); 8] = [
        (PNG, "image/png", "png"),
        (b"\xff\xd8\xff\xe0jfif", "image/jpeg", "jpg"),
        (b"GIF87a-pixels", "image/gif", "gif"),
        (GIF, "image/gif", "gif"),
        (b"II*\0little-endian", "image/tiff", "tiff"),
        (b"MM\0*big-endian", "image/tiff", "tiff"),
        (b"%PDF-1.7\n", "application/pdf", "pdf"),
        (
            b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>",
            "image/svg+xml",
            "svg",
        ),
    ];
    let dir = TempDir::new().unwrap();
    let store = icon_store(&dir);

    for (bytes, mime, extension) in samples {
        let icon = store.add(bytes, "application/octet-stream").unwrap();

        let Icon::Custom { mime: found, .. } = &icon else {
            panic!("not a custom icon: {icon:?}");
        };
        assert_eq!(found, mime);
        let path = store.path_for(&icon).unwrap();
        assert_eq!(path.extension().unwrap(), extension);
    }
}

#[test]
fn svg_after_a_byte_order_mark_and_xml_declaration_is_recognized() {
    let dir = TempDir::new().unwrap();
    let svg = b"\xef\xbb\xbf\n<?xml version=\"1.0\"?>\n<!-- editor -->\n<svg viewBox=\"0 0 1 1\"/>";

    let icon = icon_store(&dir).add(svg, "image/svg+xml").unwrap();

    assert!(matches!(icon, Icon::Custom { mime, .. } if mime == "image/svg+xml"));
}

#[test]
fn the_type_found_in_the_bytes_wins_over_the_declared_one() {
    let dir = TempDir::new().unwrap();

    let icon = icon_store(&dir).add(PNG, "image/jpeg").unwrap();

    assert!(matches!(icon, Icon::Custom { mime, .. } if mime == "image/png"));
}

#[test]
fn bytes_of_another_type_are_rejected_with_the_declared_type() {
    let dir = TempDir::new().unwrap();

    let error = icon_store(&dir).add(b"BM bitmap", "image/bmp").unwrap_err();

    assert!(matches!(error, Error::UnsupportedIcon { mime } if mime == "image/bmp"));
    assert!(!icons_dir(&dir).exists());
}

#[test]
fn xml_that_is_not_svg_is_rejected() {
    let dir = TempDir::new().unwrap();

    let error = icon_store(&dir)
        .add(b"<?xml version=\"1.0\"?><html/>", "image/svg+xml")
        .unwrap_err();

    assert!(matches!(error, Error::UnsupportedIcon { .. }));
}

#[test]
fn svg_nested_in_another_root_element_is_rejected() {
    let dir = TempDir::new().unwrap();
    let html = b"<html><body><svg viewBox=\"0 0 1 1\"/></body></html>";

    let error = icon_store(&dir).add(html, "image/svg+xml").unwrap_err();

    assert!(matches!(error, Error::UnsupportedIcon { .. }));
}

#[test]
fn a_root_element_that_only_starts_with_svg_is_rejected() {
    let dir = TempDir::new().unwrap();

    let error = icon_store(&dir)
        .add(b"<svgfont/>", "image/svg+xml")
        .unwrap_err();

    assert!(matches!(error, Error::UnsupportedIcon { .. }));
}

#[test]
fn svg_after_a_doctype_with_an_internal_subset_is_recognized() {
    let dir = TempDir::new().unwrap();
    let svg = b"<?xml version=\"1.0\"?>\n<!DOCTYPE svg [ <!ENTITY a \"x\"> ]>\n<svg>\n</svg>";

    let icon = icon_store(&dir).add(svg, "image/svg+xml").unwrap();

    assert!(matches!(icon, Icon::Custom { mime, .. } if mime == "image/svg+xml"));
}

#[test]
fn the_same_image_is_stored_once() {
    let dir = TempDir::new().unwrap();
    let store = icon_store(&dir);

    let first = store.add(PNG, "image/png").unwrap();
    let second = store.add(PNG, "image/png").unwrap();

    assert_eq!(first, second);
    assert_eq!(stored_files(&dir), 1);
}

#[test]
fn path_for_is_none_for_icons_that_are_not_custom() {
    let dir = TempDir::new().unwrap();
    let icon = Icon::Royal {
        name: "Flat/Hardware/Server".to_owned(),
    };

    assert_eq!(icon_store(&dir).path_for(&icon), None);
}

#[test]
fn path_for_refuses_a_hash_that_could_leave_the_directory() {
    let dir = TempDir::new().unwrap();

    let path = icon_store(&dir).path_for(&custom("../../escape", "image/png"));

    assert_eq!(path, None);
}

#[test]
fn path_for_is_none_for_a_type_that_is_not_accepted() {
    let dir = TempDir::new().unwrap();

    let path = icon_store(&dir).path_for(&custom(&"ab".repeat(32), "image/bmp"));

    assert_eq!(path, None);
}

#[test]
fn gc_without_an_icon_directory_removes_nothing() {
    let dir = TempDir::new().unwrap();

    let removed = icon_store(&dir).gc(&Document::new("Portolano")).unwrap();

    assert_eq!(removed, 0);
}

#[test]
fn gc_removes_unused_images_and_keeps_used_ones() {
    let dir = TempDir::new().unwrap();
    let (_document, store) = saved_document(&dir);
    let used = add_old(&store, PNG);
    let unused = add_old(&store, GIF);

    let removed = store.gc(&document_using(used.clone())).unwrap();

    assert_eq!(removed, 1);
    assert!(store.path_for(&used).unwrap().exists());
    assert!(!store.path_for(&unused).unwrap().exists());
}

#[test]
fn gc_leaves_files_it_did_not_store() {
    let dir = TempDir::new().unwrap();
    let (_document, store) = saved_document(&dir);
    add_old(&store, PNG);
    let foreign = icons_dir(&dir).join("README.txt");
    fs::write(&foreign, "not an icon").unwrap();
    make_old(&foreign);

    let removed = store.gc(&Document::new("Portolano")).unwrap();

    assert_eq!(removed, 1);
    assert!(foreign.exists());
}

#[test]
fn gc_keeps_an_unused_image_added_within_the_grace_period() {
    let dir = TempDir::new().unwrap();
    let store = icon_store(&dir);
    let recent = store.add(PNG, "image/png").unwrap();

    let removed = store.gc(&Document::new("Portolano")).unwrap();

    assert_eq!(removed, 0);
    assert!(store.path_for(&recent).unwrap().exists());
}

#[test]
fn gc_keeps_an_old_image_that_was_just_added_again() {
    let dir = TempDir::new().unwrap();
    let store = icon_store(&dir);
    let reused = add_old(&store, PNG);
    store.add(PNG, "image/png").unwrap();

    let removed = store.gc(&Document::new("Portolano")).unwrap();

    assert_eq!(removed, 0);
    assert!(store.path_for(&reused).unwrap().exists());
}

#[test]
fn adding_an_existing_image_refreshes_its_modification_time() {
    let dir = TempDir::new().unwrap();
    let store = icon_store(&dir);
    let icon = add_old(&store, PNG);
    let path = store.path_for(&icon).unwrap();
    let before = modified(&path);

    store.add(PNG, "image/png").unwrap();

    assert!(modified(&path) > before);
    assert_eq!(fs::read(&path).unwrap(), PNG);
}

#[test]
fn gc_with_a_stale_document_keeps_an_image_used_by_the_file_on_disk() {
    let dir = TempDir::new().unwrap();
    let (_document, store) = saved_document(&dir);
    let stale = Document::new("Portolano");
    let icon = add_old(&store, PNG);
    let mut current = document_using(icon.clone());
    // Saved by another process, which loaded the file before the icon existed.
    let other = Store::open(dir.path().join("portolano.json")).unwrap();
    other.save(&mut current).unwrap();

    let removed = store.gc(&stale).unwrap();

    assert_eq!(removed, 0);
    assert!(store.path_for(&icon).unwrap().exists());
}

#[test]
fn gc_removes_nothing_when_the_document_on_disk_is_invalid() {
    let dir = TempDir::new().unwrap();
    let (_document, store) = saved_document(&dir);
    let icon = add_old(&store, PNG);
    fs::write(dir.path().join("portolano.json"), "{ not json").unwrap();

    let error = store.gc(&Document::new("Portolano")).unwrap_err();

    assert!(matches!(error, Error::InvalidDocument { .. }));
    assert!(store.path_for(&icon).unwrap().exists());
}

#[test]
fn gc_removes_nothing_when_the_document_on_disk_is_missing() {
    let dir = TempDir::new().unwrap();
    let store = icon_store(&dir);
    let icon = add_old(&store, PNG);

    let error = store.gc(&Document::new("Portolano")).unwrap_err();

    assert!(matches!(error, Error::Io { .. }));
    assert!(store.path_for(&icon).unwrap().exists());
}

#[test]
fn stored_image_is_named_after_the_hash_of_its_bytes() {
    let dir = TempDir::new().unwrap();

    let icon = icon_store(&dir).add(PNG, "image/png").unwrap();

    let Icon::Custom { sha256, .. } = icon else {
        panic!("custom icon expected");
    };
    assert_eq!(sha256, sha256_hex(PNG));
    assert!(is_sha256_hex(&sha256));
}

#[test]
fn extension_of_a_stored_image_maps_to_its_type() {
    assert_eq!(mime_for_extension("jpg"), Some("image/jpeg"));
    assert_eq!(mime_for_extension("svg"), Some("image/svg+xml"));
    assert_eq!(mime_for_extension("html"), None);
    assert_eq!(mime_for_extension("jpeg"), None);
}
