//! Tests of the private temporary files of the plans.

use std::fs::File;
use std::os::unix::fs::PermissionsExt;

use pretty_assertions::assert_eq;

use super::*;

fn old_file(path: &Path) {
    File::create(path)
        .unwrap()
        .set_modified(SystemTime::now() - Duration::from_secs(11 * 60))
        .unwrap();
}

#[test]
fn written_file_is_private_and_substituted_in_arguments() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("cache").join("tmp");
    let mut files = TempFiles::new(dir.clone());

    files
        .write("dc01.rdp", "full address:s:192.0.2.10:3389\n", 0o600)
        .unwrap();

    let path = dir.join("dc01.rdp");
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "full address:s:192.0.2.10:3389\n"
    );
    assert_eq!(path.metadata().unwrap().permissions().mode() & 0o777, 0o600);
    assert_eq!(dir.metadata().unwrap().permissions().mode() & 0o777, 0o700);
    assert_eq!(files.substitute("{temp:dc01.rdp}"), path.to_string_lossy());
}

#[test]
fn writing_again_replaces_the_file() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("tmp");
    let mut files = TempFiles::new(dir.clone());
    files.write("a.rdp", "old", 0o600).unwrap();

    files.write("a.rdp", "new", 0o600).unwrap();

    assert_eq!(fs::read_to_string(dir.join("a.rdp")).unwrap(), "new");
}

#[test]
fn names_that_leave_the_directory_are_rejected() {
    let root = tempfile::tempdir().unwrap();
    let mut files = TempFiles::new(root.path().join("tmp"));

    for name in ["../escape.rdp", "..", "", "a/b.rdp"] {
        assert!(files.write(name, "x", 0o600).is_err(), "{name}");
    }
}

#[test]
fn symlink_in_place_of_the_directory_is_refused_and_nothing_outside_is_removed() {
    let root = tempfile::tempdir().unwrap();
    let outside = root.path().join("outside");
    fs::create_dir(&outside).unwrap();
    fs::set_permissions(&outside, fs::Permissions::from_mode(0o700)).unwrap();
    let victim = outside.join("victim.rdp");
    old_file(&victim);
    let link = root.path().join("tmp");
    std::os::unix::fs::symlink(&outside, &link).unwrap();
    let mut files = TempFiles::new(link);

    let result = files.write("dc01.rdp", "x", 0o600);

    assert!(result.is_err());
    assert!(victim.exists());
    assert!(!outside.join("dc01.rdp").exists());
}

#[test]
fn existing_directory_with_open_permissions_is_refused() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("tmp");
    fs::create_dir(&dir).unwrap();
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o777)).unwrap();
    let mut files = TempFiles::new(dir.clone());

    let result = files.write("dc01.rdp", "x", 0o600);

    assert!(result.is_err());
    assert!(!dir.join("dc01.rdp").exists());
}

#[test]
fn first_write_removes_only_old_regular_rdp_files() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("tmp");
    fs::create_dir(&dir).unwrap();
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
    let old_rdp = dir.join("old.rdp");
    let old_other = dir.join("old.txt");
    let fresh_rdp = dir.join("fresh.rdp");
    old_file(&old_rdp);
    old_file(&old_other);
    File::create(&fresh_rdp).unwrap();
    let mut files = TempFiles::new(dir);

    files.write("new.rdp", "x", 0o600).unwrap();

    assert!(!old_rdp.exists());
    assert!(old_other.exists());
    assert!(fresh_rdp.exists());
}

#[test]
fn two_logs_created_at_once_get_different_names() {
    let root = tempfile::tempdir().unwrap();
    let mut files = TempFiles::new(root.path().join("tmp"));
    let program = Path::new("/opt/homebrew/bin/sdl-freerdp");

    let (_, first) = files.create_log(program).unwrap();
    let (_, second) = files.create_log(program).unwrap();

    assert_ne!(first, second);
}

#[test]
fn log_is_private_and_named_after_the_program() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("tmp");
    let mut files = TempFiles::new(dir.clone());

    let (_, path) = files
        .create_log(Path::new("/opt/homebrew/bin/sdl-freerdp"))
        .unwrap();

    assert_eq!(path.parent(), Some(dir.as_path()));
    let name = path.file_name().unwrap().to_string_lossy().into_owned();
    assert!(
        name.starts_with("sdl-freerdp-") && name.ends_with(".log"),
        "{name}"
    );
    assert_eq!(path.metadata().unwrap().permissions().mode() & 0o777, 0o600);
}

#[test]
fn old_logs_are_removed_like_rdp_files() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("tmp");
    fs::create_dir(&dir).unwrap();
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
    let old_log = dir.join("sdl-freerdp-1.log");
    old_file(&old_log);
    let mut files = TempFiles::new(dir);

    files.write("new.rdp", "x", 0o600).unwrap();

    assert!(!old_log.exists());
}

#[test]
fn stale_cleanup_does_not_follow_links() {
    let root = tempfile::tempdir().unwrap();
    let target = root.path().join("target.rdp");
    old_file(&target);
    let dir = root.path().join("tmp");
    fs::create_dir(&dir).unwrap();
    std::os::unix::fs::symlink(&target, dir.join("link.rdp")).unwrap();

    let removed = remove_stale(&dir, STALE_AFTER, SystemTime::now());

    assert_eq!(removed, 0);
    assert!(target.exists());
}
