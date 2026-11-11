//! Document persistence: versioned load, atomic save and detection of
//! concurrent changes (SPEC §6.2, DEC-016.1).
//!
//! Several processes may keep the same document open (the app and the CLI).
//! The document is locked only during each operation, shared to read and
//! exclusive to write, and a save refuses to overwrite a file that changed
//! since this store last read or wrote it.

mod lock;

use std::cmp::Ordering;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};

use chrono::Utc;
use serde_json::Value;
use sha2::{Digest, Sha256};

pub(crate) use lock::{Access, with_lock};

use crate::error::{Error, Result};
use crate::icons::IconStore;
use crate::migrate::{MIGRATIONS, migrate};
use crate::model::{Document, FORMAT_ID, SCHEMA_VERSION};

/// Code of the validation issue that makes a document unloadable (SPEC §4.4).
const DUPLICATE_ID: &str = "duplicate_id";

/// SHA-256 of the content of a document file.
type Fingerprint = [u8; 32];

/// An open document file. Opening it does not keep it locked: other stores,
/// in this or other processes, may open the same file at the same time.
///
/// The store remembers the fingerprint of the bytes it last read or wrote,
/// so that [`Store::save`] can tell whether someone else changed the file in
/// the meantime. It is `Send + Sync`: one store can serve several threads.
#[derive(Debug)]
pub struct Store {
    path: PathBuf,
    fingerprint: Mutex<Fingerprint>,
}

impl Store {
    /// Opens an existing document file and records the fingerprint of its
    /// current content, without parsing it. Symlinks are resolved: the store
    /// works on the file they point to.
    ///
    /// # Errors
    /// [`Error::Io`](crate::Error::Io) on filesystem errors, including a
    /// missing document.
    pub fn open(path: impl Into<PathBuf>) -> Result<Store> {
        let given = path.into();
        // Resolved before locking: a missing document fails here and leaves
        // no lock file, and a symlink (for example into a repository of the
        // user) stays intact because the lock, the backup, the temporary file
        // and the rename all work next to the real file.
        let path = fs::canonicalize(&given).map_err(io_error(&given))?;
        let bytes = with_lock(&path, Access::Shared, || read(&path))?;
        Ok(Store::new(path, &bytes))
    }

    /// Creates a new document file containing an empty document named
    /// `name` and returns its store and the document. The directory of the
    /// file is created if needed, so that the first run can use the default
    /// location (SPEC §6.1).
    ///
    /// # Errors
    /// [`Error::AlreadyExists`](crate::Error::AlreadyExists) if the file
    /// exists, [`Error::Serialize`](crate::Error::Serialize),
    /// [`Error::Io`](crate::Error::Io).
    pub fn create(path: impl Into<PathBuf>, name: &str) -> Result<(Store, Document)> {
        let given = path.into();
        let dir = parent_dir(&given);
        fs::create_dir_all(dir).map_err(io_error(dir))?;
        let file_name = given
            .file_name()
            .ok_or_else(|| io_error(&given)(io::ErrorKind::InvalidInput.into()))?;
        // The file does not exist yet, so only its directory can be resolved;
        // see `open` for why the path is resolved at all.
        let path = fs::canonicalize(dir)
            .map_err(io_error(dir))?
            .join(file_name);
        let doc = Document::new(name);
        let bytes = to_json(&doc)?;
        with_lock(&path, Access::Exclusive, || {
            // Checked under the lock, so two concurrent creations cannot both
            // succeed.
            if path.try_exists().map_err(io_error(&path))? {
                return Err(Error::AlreadyExists { path: path.clone() });
            }
            write_atomically(&path, &bytes)
        })?;
        Ok((Store::new(path, &bytes), doc))
    }

    /// Reads the document, migrating it from an older schema version if
    /// needed, and remembers its fingerprint for the next [`Store::save`].
    ///
    /// # Errors
    /// [`Error::NotADocument`](crate::Error::NotADocument),
    /// [`Error::UnsupportedSchema`](crate::Error::UnsupportedSchema),
    /// migration errors, [`Error::InvalidDocument`](crate::Error::InvalidDocument),
    /// [`Error::DuplicateId`](crate::Error::DuplicateId),
    /// [`Error::Io`](crate::Error::Io).
    pub fn load(&self) -> Result<Document> {
        let bytes = with_lock(&self.path, Access::Shared, || read(&self.path))?;
        let doc = parse_document(&self.path, &bytes)?;
        // Only a successful load counts as having seen these bytes: after a
        // failure the caller holds no document that reflects them.
        *self.fingerprint() = fingerprint_of(&bytes);
        Ok(doc)
    }

    /// Writes the document atomically (temporary file, `fsync`, rename) after
    /// saving the previous content to `<file>.bak`, and updates
    /// `doc.modified`. The output is deterministic: 2-space pretty JSON with
    /// a final newline.
    ///
    /// # Errors
    /// [`Error::ModifiedExternally`](crate::Error::ModifiedExternally) if the
    /// file changed after this store last read or wrote it; the document must
    /// be loaded again.
    /// [`Error::DocumentMissing`](crate::Error::DocumentMissing) if the file
    /// was moved or deleted.
    /// [`Error::Serialize`](crate::Error::Serialize),
    /// [`Error::Io`](crate::Error::Io).
    pub fn save(&self, doc: &mut Document) -> Result<()> {
        with_lock(&self.path, Access::Exclusive, || {
            // Held for the whole save, so that saves through this store from
            // several threads do not interleave.
            let mut known = self.fingerprint();
            let current = match fs::read(&self.path) {
                Ok(current) => current,
                Err(e) if e.kind() == io::ErrorKind::NotFound => {
                    return Err(Error::DocumentMissing {
                        path: self.path.clone(),
                    });
                }
                Err(source) => return Err(io_error(&self.path)(source)),
            };
            if fingerprint_of(&current) != *known {
                return Err(self.modified_externally());
            }
            doc.modified = Utc::now();
            let bytes = to_json(doc)?;
            write_atomically(&with_suffix(&self.path, ".bak"), &current)?;
            write_atomically(&self.path, &bytes)?;
            *known = fingerprint_of(&bytes);
            Ok(())
        })
    }

    /// The icon store next to this document (`<dir>/<stem>.icons/`).
    pub fn icons(&self) -> IconStore {
        IconStore::for_document(&self.path)
    }

    fn new(path: PathBuf, bytes: &[u8]) -> Store {
        Store {
            path,
            fingerprint: Mutex::new(fingerprint_of(bytes)),
        }
    }

    /// The remembered fingerprint. A panic while the mutex was held cannot
    /// leave it half-written (it is replaced in one assignment), so a
    /// poisoned mutex is still usable.
    fn fingerprint(&self) -> MutexGuard<'_, Fingerprint> {
        self.fingerprint
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    fn modified_externally(&self) -> Error {
        Error::ModifiedExternally {
            path: self.path.clone(),
        }
    }
}

fn read(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).map_err(io_error(path))
}

/// Reads and parses the document file at `path`, as [`Store::load`] does,
/// without locking it: the caller holds the lock.
pub(crate) fn read_document(path: &Path) -> Result<Document> {
    parse_document(path, &read(path)?)
}

/// Parses the bytes of a document file of any supported schema version.
fn parse_document(path: &Path, bytes: &[u8]) -> Result<Document> {
    let invalid = |source| Error::InvalidDocument {
        path: path.to_owned(),
        source,
    };
    let value: Value = serde_json::from_slice(bytes).map_err(invalid)?;
    let found = schema_version_of(path, &value)?;
    let parsed = match found.cmp(&SCHEMA_VERSION) {
        Ordering::Greater => {
            return Err(Error::UnsupportedSchema {
                found,
                supported: SCHEMA_VERSION,
            });
        }
        // Parsed again from the bytes so that errors carry line and column.
        Ordering::Equal => serde_json::from_slice(bytes),
        Ordering::Less => {
            serde_json::from_value(migrate(value, found, SCHEMA_VERSION, MIGRATIONS)?)
        }
    };
    let doc: Document = parsed.map_err(invalid)?;
    reject_duplicate_ids(&doc)?;
    Ok(doc)
}

/// The `schema_version` of a Portolano document; anything else is not a
/// document.
fn schema_version_of(path: &Path, value: &Value) -> Result<u32> {
    let not_a_document = || Error::NotADocument {
        path: path.to_owned(),
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

fn fingerprint_of(bytes: &[u8]) -> Fingerprint {
    Sha256::digest(bytes).into()
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
