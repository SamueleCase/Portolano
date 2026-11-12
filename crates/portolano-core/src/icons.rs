//! Store of custom icon images (SPEC §6.4).
//!
//! Images live in `<dir of the document>/<stem of the document>.icons/` as
//! `<sha256>.<ext>`. Accepted types: png, jpeg, gif, tiff, svg+xml, pdf; the
//! type is recognized from the magic bytes and the extension derived from it.

use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use sha2::{Digest, Sha256};

mod image_type;

use self::image_type::ImageType;
use crate::error::{Error, Result};
use crate::model::{Document, Icon};
use crate::store::{Access, io_error, read_document, with_lock, write_atomically};

/// The largest icon image accepted, in bytes. Icons are shown at most 52 px
/// wide and those of Royal documents weigh a few kilobytes; the limit bounds
/// what showing one makes the app read into memory, and an image refused
/// here is never stored only to be refused when shown.
pub const MAX_ICON_BYTES: u64 = 4 * 1024 * 1024;

/// The icon directory of one document. Obtained from
/// [`Store::icons`](crate::store::Store::icons).
#[derive(Debug, Clone)]
pub struct IconStore {
    /// The document the icons belong to; its lock guards [`IconStore::gc`].
    document: PathBuf,
    dir: PathBuf,
}

impl IconStore {
    /// The icon store of the document at `document`:
    /// `<dir>/<stem>.icons/`. The directory is created on the first
    /// [`IconStore::add`].
    pub(crate) fn for_document(document: &Path) -> IconStore {
        let stem = document.file_stem().unwrap_or_default();
        let mut name = stem.to_os_string();
        name.push(".icons");
        IconStore {
            document: document.to_owned(),
            dir: document.with_file_name(name),
        }
    }

    /// Stores an image, deduplicated by hash, and returns the matching
    /// [`Icon::Custom`].
    ///
    /// `mime` is the type declared by the caller. The stored type is the one
    /// recognized from the bytes, which wins when the two differ; `mime` is
    /// only reported when the bytes are not of an accepted type.
    ///
    /// # Errors
    /// [`Error::IconTooLarge`](crate::Error::IconTooLarge) for an image
    /// larger than [`MAX_ICON_BYTES`],
    /// [`Error::UnsupportedIcon`](crate::Error::UnsupportedIcon) for a type
    /// that is not accepted, [`Error::Io`](crate::Error::Io).
    pub fn add(&self, bytes: &[u8], mime: &str) -> Result<Icon> {
        let size = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        if size > MAX_ICON_BYTES {
            return Err(Error::IconTooLarge {
                size,
                max: MAX_ICON_BYTES,
            });
        }
        let image = ImageType::detect(bytes).ok_or_else(|| Error::UnsupportedIcon {
            mime: mime.to_owned(),
        })?;
        let sha256 = sha256_hex(bytes);
        let path = self.dir.join(image.file_name(&sha256));
        // Under the shared lock, so that `gc` cannot remove the file between
        // the check and the refresh of its modification time.
        with_lock(&self.document, Access::Shared, || {
            // The name is the hash of the content, so an existing file
            // already holds these bytes; touching it tells `gc` that it is in
            // use again.
            match touch(&path) {
                Ok(()) => Ok(()),
                Err(e) if e.kind() == io::ErrorKind::NotFound => {
                    fs::create_dir_all(&self.dir).map_err(io_error(&self.dir))?;
                    write_atomically(&path, bytes)
                }
                Err(source) => Err(io_error(&path)(source)),
            }
        })?;
        Ok(Icon::Custom {
            sha256,
            mime: image.mime().to_owned(),
        })
    }

    /// The file of a custom icon; `None` for other icon kinds, and for a
    /// custom icon whose hash or type is malformed.
    pub fn path_for(&self, icon: &Icon) -> Option<PathBuf> {
        let Icon::Custom { sha256, mime } = icon else {
            return None;
        };
        // The document may have been edited by hand: only a well-formed hash
        // becomes a file name, so that it cannot point outside the directory.
        if !is_sha256_hex(sha256) {
            return None;
        }
        let image = ImageType::from_mime(mime)?;
        Some(self.dir.join(image.file_name(sha256)))
    }

    /// The stored images, as [`Icon::Custom`], ordered by hash: the choices
    /// of the icon picker of the app. Files whose name is not that of a
    /// stored image are left out, as by [`IconStore::gc`], and so is anything
    /// that is not a regular file.
    ///
    /// # Errors
    /// [`Error::Io`](crate::Error::Io) if the directory cannot be read; a
    /// missing directory is an empty store.
    pub fn list(&self) -> Result<Vec<Icon>> {
        let entries = match fs::read_dir(&self.dir) {
            Ok(entries) => entries,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(source) => return Err(io_error(&self.dir)(source)),
        };
        let mut icons = Vec::new();
        for entry in entries {
            let entry = entry.map_err(io_error(&self.dir))?;
            // Only regular files: never a symlink, which the icon scheme
            // of the app would refuse to serve anyway.
            let is_file = entry
                .file_type()
                .map_err(io_error(&entry.path()))?
                .is_file();
            if let Some(icon) = stored_icon(&entry.path()).filter(|_| is_file) {
                icons.push(icon);
            }
        }
        icons.sort_by(|a, b| sha256_of(a).cmp(sha256_of(b)));
        Ok(icons)
    }

    /// Removes the images no longer used by `doc` and returns how many were
    /// removed. Files whose name is not that of a stored image are left
    /// alone: the store did not put them there.
    ///
    /// An image counts as used if `doc` or the document currently on disk
    /// uses it: `doc` may hold icons not saved yet, and the file may have
    /// been saved by another process since `doc` was loaded.
    ///
    /// It runs under the exclusive lock of the document, so no
    /// [`Store::save`](crate::store::Store::save) or [`IconStore::add`]
    /// happens while it reads the file and removes images. Images added or
    /// reused in the last ten minutes are kept even if unused: another
    /// process may have added them for a document it has not saved yet.
    /// Known limit: an image added by another process that is still not
    /// saved after the grace period is removed, and that process then saves
    /// a reference to a missing file.
    ///
    /// # Errors
    /// The errors of [`Store::load`](crate::store::Store::load) when the
    /// document on disk cannot be read or is not valid, in which case
    /// nothing is removed; [`Error::Io`](crate::Error::Io).
    pub fn gc(&self, doc: &Document) -> Result<usize> {
        with_lock(&self.document, Access::Exclusive, || {
            self.remove_unused(doc)
        })
    }

    fn remove_unused(&self, doc: &Document) -> Result<usize> {
        let candidates = self.old_images()?;
        // Reading the file and visiting the trees are only worth it when
        // something could go.
        if candidates.is_empty() {
            return Ok(0);
        }
        let on_disk = read_document(&self.document)?;
        let used: HashSet<PathBuf> = [doc, &on_disk]
            .into_iter()
            .flat_map(|document| document.walk())
            .filter_map(|(_, node)| node.icon.as_ref())
            .filter_map(|icon| self.path_for(icon))
            .collect();
        let mut removed = 0;
        for path in candidates.iter().filter(|path| !used.contains(*path)) {
            fs::remove_file(path).map_err(io_error(path))?;
            removed += 1;
        }
        Ok(removed)
    }

    /// The stored images last added or reused before the grace period.
    fn old_images(&self) -> Result<Vec<PathBuf>> {
        let entries = match fs::read_dir(&self.dir) {
            Ok(entries) => entries,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(source) => return Err(io_error(&self.dir)(source)),
        };
        let now = SystemTime::now();
        let mut old = Vec::new();
        for entry in entries {
            let path = entry.map_err(io_error(&self.dir))?.path();
            if !is_stored_image(&path) {
                continue;
            }
            let modified = fs::metadata(&path)
                .and_then(|metadata| metadata.modified())
                .map_err(io_error(&path))?;
            if !is_recent(modified, now) {
                old.push(path);
            }
        }
        Ok(old)
    }
}

/// How long [`IconStore::gc`] keeps an unused image after it was added or
/// reused. Another process (the app while the CLI runs, or the reverse) may
/// have added it for a document it has not saved yet; ten minutes cover any
/// realistic edit, and an image that is really unused goes at a later `gc`.
const GC_GRACE_PERIOD: Duration = Duration::from_secs(10 * 60);

/// Whether a file modified at `modified` is within the grace period. A time
/// in the future (clock changes) counts as recent: keeping a file is the
/// safe mistake.
fn is_recent(modified: SystemTime, now: SystemTime) -> bool {
    now.duration_since(modified)
        .map_or(true, |age| age < GC_GRACE_PERIOD)
}

/// Sets the modification time of an existing file to now.
fn touch(path: &Path) -> io::Result<()> {
    fs::File::options()
        .write(true)
        .open(path)?
        .set_modified(SystemTime::now())
}

/// The SHA-256 of `bytes` in lowercase hex: the stem of the file name that
/// [`IconStore::add`] gives to an image with these bytes.
pub fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// Whether `text` is a SHA-256 in lowercase hex, the only stem a stored
/// image may have.
pub fn is_sha256_hex(text: &str) -> bool {
    text.len() == 64 && text.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// The MIME type of the images stored with `extension` (`png`, `jpg`, `gif`,
/// `tiff`, `svg`, `pdf`); `None` for any other extension.
pub fn mime_for_extension(extension: &str) -> Option<&'static str> {
    ImageType::from_extension(extension).map(ImageType::mime)
}

/// Whether `path` is named `<sha256>.<extension of an accepted type>`.
fn is_stored_image(path: &Path) -> bool {
    stored_icon(path).is_some()
}

/// The icon stored at `path`, if it is named
/// `<sha256>.<extension of an accepted type>`.
fn stored_icon(path: &Path) -> Option<Icon> {
    let stem = path.file_stem().and_then(OsStr::to_str)?;
    let image = path
        .extension()
        .and_then(OsStr::to_str)
        .and_then(ImageType::from_extension)?;
    is_sha256_hex(stem).then(|| Icon::Custom {
        sha256: stem.to_owned(),
        mime: image.mime().to_owned(),
    })
}

/// The hash of a custom icon; empty for the other kinds.
fn sha256_of(icon: &Icon) -> &str {
    match icon {
        Icon::Custom { sha256, .. } => sha256,
        Icon::Builtin { .. } | Icon::Royal { .. } => "",
    }
}

#[cfg(test)]
mod tests;
