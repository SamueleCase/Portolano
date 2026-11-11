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

use crate::error::{Error, Result};
use crate::model::{Document, Icon};
use crate::store::{Access, io_error, read_document, with_lock, write_atomically};

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
    /// [`Error::UnsupportedIcon`](crate::Error::UnsupportedIcon) for a type
    /// that is not accepted, [`Error::Io`](crate::Error::Io).
    pub fn add(&self, bytes: &[u8], mime: &str) -> Result<Icon> {
        let image = ImageType::detect(bytes).ok_or_else(|| Error::UnsupportedIcon {
            mime: mime.to_owned(),
        })?;
        let sha256 = hex::encode(Sha256::digest(bytes));
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

/// The accepted image types (SPEC §6.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImageType {
    Png,
    Jpeg,
    Gif,
    Tiff,
    Svg,
    Pdf,
}

/// Leading bytes of the binary types. SVG is text and has none.
const SIGNATURES: &[(&[u8], ImageType)] = &[
    (b"\x89PNG\r\n\x1a\n", ImageType::Png),
    (b"\xff\xd8\xff", ImageType::Jpeg),
    (b"GIF87a", ImageType::Gif),
    (b"GIF89a", ImageType::Gif),
    (b"II*\0", ImageType::Tiff),
    (b"MM\0*", ImageType::Tiff),
    (b"%PDF-", ImageType::Pdf),
];

/// How far into an SVG file the `<svg` element is looked for: enough for an
/// XML declaration, a DOCTYPE and the comments that editors put first.
const SVG_HEAD_LEN: usize = 4096;

impl ImageType {
    const ALL: [ImageType; 6] = [
        ImageType::Png,
        ImageType::Jpeg,
        ImageType::Gif,
        ImageType::Tiff,
        ImageType::Svg,
        ImageType::Pdf,
    ];

    /// The type of `bytes`, from their magic bytes.
    fn detect(bytes: &[u8]) -> Option<ImageType> {
        SIGNATURES
            .iter()
            .find(|(magic, _)| bytes.starts_with(magic))
            .map(|&(_, image)| image)
            .or_else(|| looks_like_svg(bytes).then_some(ImageType::Svg))
    }

    fn from_mime(mime: &str) -> Option<ImageType> {
        Self::ALL.into_iter().find(|image| image.mime() == mime)
    }

    fn from_extension(extension: &str) -> Option<ImageType> {
        Self::ALL
            .into_iter()
            .find(|image| image.extension() == extension)
    }

    fn mime(self) -> &'static str {
        match self {
            ImageType::Png => "image/png",
            ImageType::Jpeg => "image/jpeg",
            ImageType::Gif => "image/gif",
            ImageType::Tiff => "image/tiff",
            ImageType::Svg => "image/svg+xml",
            ImageType::Pdf => "application/pdf",
        }
    }

    fn extension(self) -> &'static str {
        match self {
            ImageType::Png => "png",
            ImageType::Jpeg => "jpg",
            ImageType::Gif => "gif",
            ImageType::Tiff => "tiff",
            ImageType::Svg => "svg",
            ImageType::Pdf => "pdf",
        }
    }

    fn file_name(self, sha256: &str) -> String {
        format!("{sha256}.{}", self.extension())
    }
}

/// SVG has no binary signature: it is XML whose root element is `<svg`.
/// The byte order mark, the XML declaration (and other processing
/// instructions), comments and the DOCTYPE may come before the root.
fn looks_like_svg(bytes: &[u8]) -> bool {
    let head = &bytes[..bytes.len().min(SVG_HEAD_LEN)];
    let mut rest = head
        .strip_prefix(b"\xef\xbb\xbf")
        .unwrap_or(head)
        .trim_ascii_start();
    while let Some(len) = prolog_item_len(rest) {
        rest = rest[len..].trim_ascii_start();
    }
    rest.strip_prefix(b"<svg").is_some_and(|after| {
        matches!(
            after.first(),
            Some(b'>' | b'/' | b' ' | b'\t' | b'\r' | b'\n')
        )
    })
}

/// The length of the XML prolog item that `rest` starts with, or `None` if
/// it starts with something else or the item is not terminated in the head.
fn prolog_item_len(rest: &[u8]) -> Option<usize> {
    if rest.starts_with(b"<?") {
        end_of(rest, b"?>")
    } else if rest.starts_with(b"<!--") {
        end_of(rest, b"-->")
    } else if rest.starts_with(b"<!DOCTYPE") {
        // An internal subset, `[ ... ]`, may contain `>` itself.
        let first = rest.iter().position(|&b| b == b'[' || b == b'>')?;
        if rest[first] == b'>' {
            return Some(first + 1);
        }
        let subset_end = first + end_of(&rest[first..], b"]")?;
        Some(subset_end + end_of(&rest[subset_end..], b">")?)
    } else {
        None
    }
}

/// The index just past the first `marker` in `haystack`.
fn end_of(haystack: &[u8], marker: &[u8]) -> Option<usize> {
    haystack
        .windows(marker.len())
        .position(|window| window == marker)
        .map(|start| start + marker.len())
}

fn is_sha256_hex(text: &str) -> bool {
    text.len() == 64 && text.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// Whether `path` is named `<sha256>.<extension of an accepted type>`.
fn is_stored_image(path: &Path) -> bool {
    let stem = path.file_stem().and_then(OsStr::to_str);
    let extension = path.extension().and_then(OsStr::to_str);
    stem.is_some_and(is_sha256_hex) && extension.and_then(ImageType::from_extension).is_some()
}

#[cfg(test)]
mod tests;
