//! Temporary files of the plans (the `.rdp` of Windows App, SPEC §7.3), in a
//! private per-user directory. They never contain secrets.
//!
//! The directory lives in the user's cache directory, not in a shared
//! temporary directory, so another user can neither predict nor plant it; it
//! is used only after checking, without following links, that it is a
//! directory of the current user with permissions `0700`.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::ConnectError;
use crate::plan::temp_placeholder;

/// Age after which a temporary file is removed: the client has long read it.
const STALE_AFTER: Duration = Duration::from_secs(10 * 60);

/// Extensions of the files that the cleanup removes: the `.rdp` files of
/// SPEC §7.3 and the logs of detached clients.
const STALE_EXTENSIONS: [&str; 2] = ["rdp", LOG_EXTENSION];

/// Extension of the logs of detached clients.
const LOG_EXTENSION: &str = "log";

/// Permissions of the logs: client messages may name users and hosts.
const LOG_MODE: u32 = 0o600;

/// The per-user directory of the temporary files and logs: `tmp` inside the
/// cache directory of Portolano, such as
/// `~/Library/Caches/dev.portolano.Portolano/tmp` on macOS. `None` without a
/// home directory.
fn per_user_dir() -> Option<PathBuf> {
    directories::ProjectDirs::from("dev", "portolano", "Portolano")
        .map(|dirs| dirs.cache_dir().join("tmp"))
}

/// The temporary files written while running a plan. They outlive the plan,
/// because a detached client reads them after `connect` returns, and are
/// removed by a later run once stale.
pub(super) struct TempFiles {
    dir: Option<PathBuf>,
    checked: bool,
    written: Vec<(String, PathBuf)>,
}

impl TempFiles {
    /// Files in `dir`, created and checked on the first write.
    #[cfg(test)]
    pub(super) fn new(dir: PathBuf) -> TempFiles {
        TempFiles {
            dir: Some(dir),
            checked: false,
            written: Vec::new(),
        }
    }

    /// Files in [`per_user_dir`].
    pub(super) fn per_user() -> TempFiles {
        TempFiles {
            dir: per_user_dir(),
            checked: false,
            written: Vec::new(),
        }
    }

    /// The checked directory; the first call checks it and removes the
    /// stale files in it.
    fn ready_dir(&mut self) -> io::Result<PathBuf> {
        let dir = self.dir.clone().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "directory dei file temporanei non determinabile (manca la home)",
            )
        })?;
        if !self.checked {
            ensure_private_dir(&dir)?;
            remove_stale(&dir, STALE_AFTER, SystemTime::now());
            self.checked = true;
        }
        Ok(dir)
    }

    /// Creates a new log for the detached program `program`, named
    /// `<program>-<milliseconds since the epoch>-<random hex>.log`, with
    /// permissions `0600`.
    ///
    /// # Errors
    /// [`ConnectError::Io`] for a directory that is not private or a
    /// filesystem failure.
    pub(super) fn create_log(
        &mut self,
        program: &Path,
    ) -> Result<(fs::File, PathBuf), ConnectError> {
        let io_error = |source: io::Error| ConnectError::Io {
            context: format!("creazione del log di '{}'", program.display()),
            source,
        };
        let dir = self.ready_dir().map_err(io_error)?;
        let stem: String = program
            .file_stem()
            .map(|stem| stem.to_string_lossy())
            .unwrap_or_default()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let millis = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_millis())
            .unwrap_or_default();
        // A random suffix keeps two launches in the same millisecond apart.
        let mut random = [0u8; 4];
        getrandom::fill(&mut random).map_err(|error| io_error(io::Error::other(error)))?;
        let suffix = hex::encode(random);
        let path = dir.join(format!("{stem}-{millis}-{suffix}.{LOG_EXTENSION}"));
        let file = create_new(&path, LOG_MODE).map_err(io_error)?;
        Ok((file, path))
    }

    /// Writes `contents` to the file `name` with permissions `mode`,
    /// replacing an older file of the same name. The first write checks the
    /// directory and removes the stale `.rdp` and `.log` files in it.
    ///
    /// # Errors
    /// [`ConnectError::Io`] for an invalid name, a directory that is not
    /// private or a filesystem failure.
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
        let dir = self.ready_dir().map_err(io_error)?;
        let path = dir.join(name);
        match fs::remove_file(&path) {
            Err(error) if error.kind() != io::ErrorKind::NotFound => return Err(io_error(error)),
            _ => {}
        }
        let mut file = create_new(&path, mode).map_err(io_error)?;
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
}

/// Creates `path` with permissions `mode`; `create_new` refuses a file or
/// link planted after the removal of the previous one.
fn create_new(path: &Path, mode: u32) -> io::Result<fs::File> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(mode);
    }
    #[cfg(not(unix))]
    let _ = mode;
    options.open(path)
}

/// Removes the regular `.rdp` and `.log` files of `dir` older than `age`. Links and
/// other files are left alone. Best effort: a file that cannot be removed
/// now will be removed by a later run.
fn remove_stale(dir: &Path, age: Duration, now: SystemTime) -> usize {
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    entries
        .filter_map(Result::ok)
        .filter(|entry| {
            let is_ours = entry
                .path()
                .extension()
                .is_some_and(|ext| STALE_EXTENSIONS.iter().any(|ours| ext == *ours));
            // `DirEntry::file_type` and `metadata` do not follow links.
            let is_file = entry.file_type().is_ok_and(|kind| kind.is_file());
            let is_old = entry.metadata().is_ok_and(|metadata| {
                metadata.modified().is_ok_and(|modified| {
                    now.duration_since(modified)
                        .is_ok_and(|elapsed| elapsed > age)
                })
            });
            is_ours && is_file && is_old
        })
        .filter(|entry| fs::remove_file(entry.path()).is_ok())
        .count()
}

/// Creates `dir` with permissions `0700` if it does not exist, then checks
/// without following links that it is a private directory of the current
/// user. An existing directory is never fixed: it is refused.
fn ensure_private_dir(dir: &Path) -> io::Result<()> {
    match fs::symlink_metadata(dir) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            if let Some(parent) = dir.parent() {
                fs::create_dir_all(parent)?;
            }
            // Another `connect` may create it at the same moment: what counts
            // is the check below.
            match create_private_dir(dir) {
                Err(error) if error.kind() != io::ErrorKind::AlreadyExists => return Err(error),
                _ => {}
            }
        }
        Err(error) => return Err(error),
        Ok(_) => {}
    }
    check_private(dir, &fs::symlink_metadata(dir)?)
}

#[cfg(unix)]
fn create_private_dir(dir: &Path) -> io::Result<()> {
    use std::os::unix::fs::DirBuilderExt;
    fs::DirBuilder::new().mode(0o700).create(dir)
}

#[cfg(not(unix))]
fn create_private_dir(dir: &Path) -> io::Result<()> {
    fs::create_dir(dir)
}

#[cfg(unix)]
fn check_private(dir: &Path, metadata: &fs::Metadata) -> io::Result<()> {
    use std::os::unix::fs::MetadataExt;
    let refuse = |reason: &str| {
        Err(io::Error::other(format!(
            "la directory temporanea '{}' {reason}",
            dir.display()
        )))
    };
    if !metadata.file_type().is_dir() {
        return refuse("non è una directory (o è un collegamento simbolico)");
    }
    if metadata.uid() != crate::safe_path::current_uid()? {
        return refuse("appartiene a un altro utente");
    }
    if metadata.mode() & 0o777 != 0o700 {
        return refuse("non ha i permessi 0700");
    }
    Ok(())
}

#[cfg(not(unix))]
fn check_private(dir: &Path, metadata: &fs::Metadata) -> io::Result<()> {
    if metadata.file_type().is_dir() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "la directory temporanea '{}' non è una directory",
            dir.display()
        )))
    }
}

#[cfg(test)]
#[cfg(unix)]
mod tests;
