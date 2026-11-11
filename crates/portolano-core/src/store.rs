//! Document persistence: exclusive lock, versioned load and atomic save
//! (SPEC §6.2).

use std::cmp::Ordering;
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde_json::Value;

use crate::error::{Error, Result};
use crate::icons::IconStore;
use crate::migrate::{MIGRATIONS, migrate};
use crate::model::{Document, FORMAT_ID, SCHEMA_VERSION};

/// Code of the validation issue that makes a document unloadable (SPEC §4.4).
const DUPLICATE_ID: &str = "duplicate_id";

/// An open document file, locked exclusively (`<file>.lock`) for as long as
/// the value lives.
///
/// The lock file is left on disk when the store is dropped: removing it
/// would let another process lock a file that is about to disappear.
#[derive(Debug)]
pub struct Store {
    path: PathBuf,
    /// Never read: owning the locked file is what keeps the lock, and closing
    /// it when the store is dropped is what releases it.
    _lock: File,
}

impl Store {
    /// Opens an existing document file and acquires its lock.
    ///
    /// # Errors
    /// [`Error::Locked`](crate::Error::Locked) if another process holds the
    /// lock, [`Error::Io`](crate::Error::Io) on filesystem errors, including
    /// a missing document.
    pub fn open(path: impl Into<PathBuf>) -> Result<Store> {
        let path = path.into();
        // Checked before locking so that a mistyped path leaves no lock file.
        fs::metadata(&path).map_err(io_error(&path))?;
        let lock = lock_document(&path)?;
        Ok(Store { path, _lock: lock })
    }

    /// Creates a new document file containing an empty document named
    /// `name`, acquires its lock and returns both. The directory of the file
    /// is created if needed, so that the first run can use the default
    /// location (SPEC §6.1).
    ///
    /// # Errors
    /// [`Error::AlreadyExists`](crate::Error::AlreadyExists) if the file
    /// exists, plus the errors of [`Store::open`] and [`Store::save`].
    pub fn create(path: impl Into<PathBuf>, name: &str) -> Result<(Store, Document)> {
        let path = path.into();
        let dir = parent_dir(&path);
        fs::create_dir_all(dir).map_err(io_error(dir))?;
        let lock = lock_document(&path)?;
        // Checked under the lock, so two concurrent creations cannot both
        // succeed.
        if path.try_exists().map_err(io_error(&path))? {
            return Err(Error::AlreadyExists { path });
        }
        let store = Store { path, _lock: lock };
        let mut doc = Document::new(name);
        store.save(&mut doc)?;
        Ok((store, doc))
    }

    /// Reads the document, migrating it from an older schema version if
    /// needed.
    ///
    /// # Errors
    /// [`Error::NotADocument`](crate::Error::NotADocument),
    /// [`Error::UnsupportedSchema`](crate::Error::UnsupportedSchema),
    /// migration errors, [`Error::InvalidDocument`](crate::Error::InvalidDocument),
    /// [`Error::DuplicateId`](crate::Error::DuplicateId),
    /// [`Error::Io`](crate::Error::Io).
    pub fn load(&self) -> Result<Document> {
        let bytes = fs::read(&self.path).map_err(io_error(&self.path))?;
        let value: Value = serde_json::from_slice(&bytes).map_err(|e| self.invalid(e))?;
        let found = self.schema_version_of(&value)?;
        let parsed = match found.cmp(&SCHEMA_VERSION) {
            Ordering::Greater => {
                return Err(Error::UnsupportedSchema {
                    found,
                    supported: SCHEMA_VERSION,
                });
            }
            // Parsed again from the bytes so that errors carry line and column.
            Ordering::Equal => serde_json::from_slice(&bytes),
            Ordering::Less => {
                serde_json::from_value(migrate(value, found, SCHEMA_VERSION, MIGRATIONS)?)
            }
        };
        let doc: Document = parsed.map_err(|e| self.invalid(e))?;
        reject_duplicate_ids(&doc)?;
        Ok(doc)
    }

    /// Writes the document atomically (temporary file, `fsync`, rename) after
    /// copying the previous file to `<file>.bak`, and updates
    /// `doc.modified`. The output is deterministic: 2-space pretty JSON with
    /// a final newline.
    ///
    /// # Errors
    /// [`Error::Serialize`](crate::Error::Serialize),
    /// [`Error::Io`](crate::Error::Io).
    pub fn save(&self, doc: &mut Document) -> Result<()> {
        doc.modified = Utc::now();
        let bytes = to_json(doc)?;
        self.back_up()?;
        write_atomically(&self.path, &bytes)
    }

    /// The icon store next to this document (`<dir>/<stem>.icons/`).
    pub fn icons(&self) -> IconStore {
        IconStore::for_document(&self.path)
    }

    /// Copies the current file to `<file>.bak`; nothing to do on the first
    /// save.
    fn back_up(&self) -> Result<()> {
        if !self.path.try_exists().map_err(io_error(&self.path))? {
            return Ok(());
        }
        let backup = with_suffix(&self.path, ".bak");
        fs::copy(&self.path, &backup).map_err(io_error(&backup))?;
        Ok(())
    }

    /// The `schema_version` of a Portolano document; anything else is not a
    /// document.
    fn schema_version_of(&self, value: &Value) -> Result<u32> {
        let not_a_document = || Error::NotADocument {
            path: self.path.clone(),
        };
        if value.get("format").and_then(Value::as_str) != Some(FORMAT_ID) {
            return Err(not_a_document());
        }
        value
            .get("schema_version")
            .and_then(Value::as_u64)
            .and_then(|version| u32::try_from(version).ok())
            .ok_or_else(not_a_document)
    }

    fn invalid(&self, source: serde_json::Error) -> Error {
        Error::InvalidDocument {
            path: self.path.clone(),
            source,
        }
    }
}

/// Wraps an I/O error with the path it concerns; for `map_err`.
pub(crate) fn io_error(path: &Path) -> impl FnOnce(io::Error) -> Error + '_ {
    move |source| Error::Io {
        path: path.to_owned(),
        source,
    }
}

/// Replaces the file at `path` with `bytes` so that readers see either the
/// old or the new content, never a partial one: the bytes go to
/// `<file>.tmp-<random>` in the same directory (a rename is atomic only
/// within a filesystem), are flushed to disk, and the temporary file is
/// renamed over `path`. An existing file keeps its permissions; a new one is
/// readable by the owner only.
pub(crate) fn write_atomically(path: &Path, bytes: &[u8]) -> Result<()> {
    let dir = parent_dir(path);
    let mut prefix = path.file_name().unwrap_or_default().to_os_string();
    prefix.push(".tmp-");
    // Dropping the temporary file on an early return deletes it, so a failed
    // write leaves nothing behind.
    let mut temporary = tempfile::Builder::new()
        .prefix(&prefix)
        .tempfile_in(dir)
        .map_err(io_error(dir))?;
    if let Ok(metadata) = fs::metadata(path) {
        temporary
            .as_file()
            .set_permissions(metadata.permissions())
            .map_err(io_error(temporary.path()))?;
    }
    temporary
        .write_all(bytes)
        .map_err(io_error(temporary.path()))?;
    temporary
        .as_file()
        .sync_all()
        .map_err(io_error(temporary.path()))?;
    temporary.persist(path).map_err(|e| Error::Io {
        path: path.to_owned(),
        source: e.error,
    })?;
    sync_dir(dir)
}

/// Flushes the directory entry of a rename, so that it survives a crash.
#[cfg(unix)]
fn sync_dir(dir: &Path) -> Result<()> {
    File::open(dir)
        .and_then(|handle| handle.sync_all())
        .map_err(io_error(dir))
}

/// Directories cannot be opened as files on Windows, where the rename is
/// already durable once it returns.
#[cfg(not(unix))]
fn sync_dir(_dir: &Path) -> Result<()> {
    Ok(())
}

/// Takes the exclusive lock of a document on `<file>.lock`, without waiting.
fn lock_document(path: &Path) -> Result<File> {
    let lock_path = with_suffix(path, ".lock");
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&lock_path)
        .map_err(io_error(&lock_path))?;
    let mut lock = fd_lock::RwLock::new(file);
    match lock.try_write() {
        // The guard borrows `lock` and would release the lock when dropped,
        // so it cannot live in the `Store` next to the file it borrows.
        // Forgetting it is sound and keeps the lock: the OS lock belongs to
        // the open file, which the `Store` owns and closes on drop.
        Ok(guard) => std::mem::forget(guard),
        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
            return Err(Error::Locked {
                path: path.to_owned(),
            });
        }
        Err(source) => {
            return Err(Error::Io {
                path: lock_path,
                source,
            });
        }
    }
    Ok(lock.into_inner())
}

/// Serializes a document in its on-disk form. The bytes depend only on the
/// document: every map in the model is ordered by key.
fn to_json(doc: &Document) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(doc).map_err(|source| Error::Serialize { source })?;
    bytes.push(b'\n');
    Ok(bytes)
}

/// A document with duplicated ids cannot be addressed by id, so it is
/// refused instead of being loaded (SPEC §6.2).
fn reject_duplicate_ids(doc: &Document) -> Result<()> {
    let duplicate = doc
        .validate()
        .into_iter()
        .find_map(|issue| issue.node.filter(|_| issue.code == DUPLICATE_ID));
    match duplicate {
        Some(id) => Err(Error::DuplicateId { id }),
        None => Ok(()),
    }
}

/// The directory of `path`, `.` for a bare file name.
pub(crate) fn parent_dir(path: &Path) -> &Path {
    match path.parent() {
        Some(dir) if !dir.as_os_str().is_empty() => dir,
        _ => Path::new("."),
    }
}

/// `path` with `suffix` appended to its file name (`a.json` → `a.json.bak`).
fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = OsString::from(path.as_os_str());
    name.push(suffix);
    PathBuf::from(name)
}

#[cfg(test)]
mod tests;
