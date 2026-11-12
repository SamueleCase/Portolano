//! The `portolano-icon://` scheme, which serves the custom icons of the open
//! document to `<img>` elements (SPEC-UI §4.7, DEC-024).
//!
//! Only names of stored images are accepted, `<sha256>.<ext>`, and only from
//! the icon directory of the open document: the frontend cannot read any
//! other file through it.

use std::fs;
use std::io;

use portolano_core::IconStore;
use portolano_core::model::Icon;
use tauri::http::{Response, StatusCode, header};

/// The name of the scheme; on macOS the URLs are
/// `portolano-icon://localhost/<sha256>.<ext>`.
pub const SCHEME: &str = "portolano-icon";

/// Length of a SHA-256 in lowercase hex.
const SHA256_HEX_LEN: usize = 64;

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
    match fs::read(&file) {
        Ok(bytes) => image(mime, bytes),
        Err(error) if error.kind() == io::ErrorKind::NotFound => empty(StatusCode::NOT_FOUND),
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
    let is_hash = sha256.len() == SHA256_HEX_LEN
        && sha256
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !is_hash {
        return None;
    }
    let mime = match extension {
        "png" => "image/png",
        "jpg" => "image/jpeg",
        "gif" => "image/gif",
        "tiff" => "image/tiff",
        "svg" => "image/svg+xml",
        "pdf" => "application/pdf",
        _ => return None,
    };
    Some((sha256, mime))
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
