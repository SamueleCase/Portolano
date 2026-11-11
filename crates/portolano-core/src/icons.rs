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

use sha2::{Digest, Sha256};

use crate::error::{Error, Result};
use crate::model::{Document, Icon};
use crate::store::{Access, io_error, with_lock, write_atomically};

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
        // The name is the hash of the content, so an existing file already
        // holds these bytes.
        if !path.try_exists().map_err(io_error(&path))? {
            fs::create_dir_all(&self.dir).map_err(io_error(&self.dir))?;
            write_atomically(&path, bytes)?;
        }
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
    /// It runs under the exclusive lock of the document, so it never races
    /// with a save. Known limit: an image that another process has added but
    /// not yet referenced in a saved document is unused for `doc` and is
    /// removed, so that process saves a reference to a missing file.
    ///
    /// # Errors
    /// [`Error::Io`](crate::Error::Io).
    pub fn gc(&self, doc: &Document) -> Result<usize> {
        with_lock(&self.document, Access::Exclusive, || {
            self.remove_unused(doc)
        })
    }

    fn remove_unused(&self, doc: &Document) -> Result<usize> {
        let entries = match fs::read_dir(&self.dir) {
            Ok(entries) => entries,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(0),
            Err(source) => {
                return Err(Error::Io {
                    path: self.dir.clone(),
                    source,
                });
            }
        };
        let used: HashSet<PathBuf> = doc
            .walk()
            .filter_map(|(_, node)| node.icon.as_ref())
            .filter_map(|icon| self.path_for(icon))
            .collect();
        let mut removed = 0;
        for entry in entries {
            let path = entry.map_err(io_error(&self.dir))?.path();
            if is_stored_image(&path) && !used.contains(&path) {
                fs::remove_file(&path).map_err(io_error(&path))?;
                removed += 1;
            }
        }
        Ok(removed)
    }
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

/// SVG has no binary signature: it is XML (optionally after a byte order
/// mark and whitespace) with an `<svg` element near the start.
fn looks_like_svg(bytes: &[u8]) -> bool {
    let head = &bytes[..bytes.len().min(SVG_HEAD_LEN)];
    let head = head.strip_prefix(b"\xef\xbb\xbf").unwrap_or(head);
    head.trim_ascii_start().starts_with(b"<") && head.windows(4).any(|window| window == b"<svg")
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
