//! The `portolano-icon://` scheme, which serves the custom icons of the open
//! document to `<img>` elements (SPEC-UI §4.7, DEC-024).
//!
//! Only names of stored images are accepted, `<sha256>.<ext>`, only regular
//! files of the icon directory of the open document are read, and only bytes
//! whose hash is the name are served: the frontend cannot read any other
//! file through it, even one that a symlink or a hand-edited directory puts
//! there.

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;

use portolano_core::IconStore;
use portolano_core::icons::{MAX_ICON_BYTES, is_sha256_hex, mime_for_extension, sha256_hex};
use portolano_core::model::Icon;
use tauri::http::{Response, StatusCode, header};

/// The name of the scheme; on macOS the URLs are
/// `portolano-icon://localhost/<sha256>.<ext>`.
pub const SCHEME: &str = "portolano-icon";

/// The response to a request for `path` (the path of the URL, with its
/// leading `/`), read from `icons`, the icon store of the open document.
pub fn respond(icons: Option<&IconStore>, path: &str) -> Response<Vec<u8>> {
    let Some((sha256, mime)) = parse_name(path.strip_prefix('/').unwrap_or(path)) else {
        return empty(StatusCode::BAD_REQUEST);
    };
    let icon = Icon::Custom {
        sha256: sha256.to_owned(),
        mime: mime.to_owned(),
    };
    let Some(file) = icons.and_then(|icons| icons.path_for(&icon)) else {
        return empty(StatusCode::NOT_FOUND);
    };
    match read_icon(&file) {
        Ok(Some(bytes)) if sha256_hex(&bytes) == sha256 => image(mime, bytes),
        // Missing, not a regular file, too large, or not the image the
        // name promises.
        Ok(_) => empty(StatusCode::NOT_FOUND),
        Err(error) => {
            tracing::warn!(%error, file = %file.display(), "lettura dell'icona non riuscita");
            empty(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// The hash and the MIME type of a stored image name,
/// `^[0-9a-f]{64}\.(png|jpg|gif|tiff|svg|pdf)$`; `None` for anything else.
fn parse_name(name: &str) -> Option<(&str, &'static str)> {
    let (sha256, extension) = name.split_once('.')?;
    if !is_sha256_hex(sha256) {
        return None;
    }
    Some((sha256, mime_for_extension(extension)?))
}

/// The bytes of `file` if it is a regular file (not a symlink) of at most
/// [`MAX_ICON_BYTES`]; `None` if it is missing or not acceptable.
fn read_icon(file: &Path) -> io::Result<Option<Vec<u8>>> {
    let metadata = match fs::symlink_metadata(file) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    if !metadata.is_file() || metadata.len() > MAX_ICON_BYTES {
        return Ok(None);
    }
    // The file may change between the check and the read: the read stays
    // bounded, and the caller serves the bytes only if their hash is the
    // name, so a swapped file is refused all the same.
    let mut bytes = Vec::new();
    File::open(file)?
        .take(MAX_ICON_BYTES + 1)
        .read_to_end(&mut bytes)?;
    Ok(u64::try_from(bytes.len())
        .is_ok_and(|len| len <= MAX_ICON_BYTES)
        .then_some(bytes))
}

fn image(mime: &str, bytes: Vec<u8>) -> Response<Vec<u8>> {
    let builder = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime)
        .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff");
    // An SVG opened as a document instead of in an `<img>` could run
    // scripts: its own policy forbids everything.
    let builder = if mime == "image/svg+xml" {
        builder.header(header::CONTENT_SECURITY_POLICY, "default-src 'none'")
    } else {
        builder
    };
    builder
        .body(bytes)
        .unwrap_or_else(|_| empty(StatusCode::INTERNAL_SERVER_ERROR))
}

fn empty(status: StatusCode) -> Response<Vec<u8>> {
    let mut response = Response::new(Vec::new());
    *response.status_mut() = status;
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        header::HeaderValue::from_static("nosniff"),
    );
    response
}

#[cfg(test)]
mod tests;
