//! Temporary files of the plans (the `.rdp` of Windows App, SPEC §7.3), in
//! a private per-user directory. They never contain secrets.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::ConnectError;
use crate::plan::temp_placeholder;

/// Age after which a temporary file is removed: the client has long read it.
const STALE_AFTER: Duration = Duration::from_secs(10 * 60);

/// The temporary files written while running a plan. They outlive the plan,
/// because a detached client reads them after `connect` returns, and are
/// removed by a later run once stale.
pub(super) struct TempFiles {
    dir: PathBuf,
    written: Vec<(String, PathBuf)>,
}

impl TempFiles {
    /// Files in `dir`, created on the first write.
    pub(super) fn new(dir: PathBuf) -> TempFiles {
        TempFiles {
            dir,
            written: Vec::new(),
        }
    }

    /// The per-user directory: on macOS the system temporary directory is
    /// already per user.
    pub(super) fn default_dir() -> PathBuf {
        std::env::temp_dir().join("portolano")
    }

    /// Writes `contents` to the file `name` with permissions `mode`,
    /// replacing an older file of the same name.
    ///
    /// # Errors
    /// [`ConnectError::Io`] for an invalid name or a filesystem failure.
    pub(super) fn write(
        &mut self,
        name: &str,
        contents: &str,
        mode: u32,
    ) -> Result<(), ConnectError> {
        let io_error = |source: io::Error| ConnectError::Io {
            context: format!("scrittura del file temporaneo '{name}'"),
            source,
        };
        if name.is_empty() || name.contains(['/', '\\']) || name == "." || name == ".." {
            return Err(io_error(io::Error::new(
                io::ErrorKind::InvalidInput,
                "nome di file non valido",
            )));
        }
        ensure_private_dir(&self.dir).map_err(io_error)?;
        let path = self.dir.join(name);
        match fs::remove_file(&path) {
            Err(error) if error.kind() != io::ErrorKind::NotFound => return Err(io_error(error)),
            _ => {}
        }
        // `create_new` refuses a file or link planted after the removal.
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(mode)
            .open(&path)
            .map_err(io_error)?;
        file.write_all(contents.as_bytes()).map_err(io_error)?;
        self.written.push((name.to_owned(), path));
        Ok(())
    }

    /// `arg` with every `{temp:<name>}` of a written file replaced by its
    /// path.
    pub(super) fn substitute(&self, arg: &str) -> String {
        self.written
            .iter()
            .fold(arg.to_owned(), |arg, (name, path)| {
                arg.replace(&temp_placeholder(name), &path.to_string_lossy())
            })
    }

    /// Removes the files older than [`STALE_AFTER`]. Best effort: a file
    /// that cannot be removed now will be removed by a later run.
    pub(super) fn remove_stale(&self) {
        remove_older_than(&self.dir, STALE_AFTER, SystemTime::now());
    }
}

fn remove_older_than(dir: &Path, age: Duration, now: SystemTime) -> usize {
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    entries
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.metadata().is_ok_and(|metadata| {
                metadata.is_file()
                    && metadata.modified().is_ok_and(|modified| {
                        now.duration_since(modified)
                            .is_ok_and(|elapsed| elapsed > age)
                    })
            })
        })
        .filter(|entry| fs::remove_file(entry.path()).is_ok())
        .count()
}

/// Creates `dir` if needed and makes it private. Changing the permissions
/// fails on a directory owned by someone else, and a symbolic link is
/// refused, so the files cannot be redirected.
fn ensure_private_dir(dir: &Path) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let metadata = fs::symlink_metadata(dir)?;
    if !metadata.is_dir() {
        return Err(io::Error::other(
            "la directory temporanea non è una directory",
        ));
    }
    fs::set_permissions(dir, fs::Permissions::from_mode(0o700))
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn written_file_is_private_and_substituted_in_arguments() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("portolano");
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
        let mut files = TempFiles::new(root.path().to_path_buf());
        files.write("a.rdp", "old", 0o600).unwrap();

        files.write("a.rdp", "new", 0o600).unwrap();

        assert_eq!(
            fs::read_to_string(root.path().join("a.rdp")).unwrap(),
            "new"
        );
    }

    #[test]
    fn names_that_leave_the_directory_are_rejected() {
        let root = tempfile::tempdir().unwrap();
        let mut files = TempFiles::new(root.path().to_path_buf());

        for name in ["../escape.rdp", "..", "", "a/b.rdp"] {
            assert!(files.write(name, "x", 0o600).is_err(), "{name}");
        }
    }

    #[test]
    fn only_files_older_than_the_limit_are_removed() {
        let root = tempfile::tempdir().unwrap();
        let old = root.path().join("old.rdp");
        let fresh = root.path().join("fresh.rdp");
        File::create(&old)
            .unwrap()
            .set_modified(SystemTime::now() - Duration::from_secs(11 * 60))
            .unwrap();
        File::create(&fresh).unwrap();

        let removed = remove_older_than(root.path(), STALE_AFTER, SystemTime::now());

        assert_eq!(removed, 1);
        assert!(!old.exists());
        assert!(fresh.exists());
    }
}
