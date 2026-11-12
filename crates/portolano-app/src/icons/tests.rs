use portolano_core::Store;
use pretty_assertions::assert_eq;
use tempfile::TempDir;

use super::*;

/// The PNG signature followed by a few bytes: enough for the icon store,
/// which recognizes the type from the magic bytes.
const PNG: &[u8] = b"\x89PNG\r\n\x1a\nnot-a-real-image";
const SVG: &[u8] = b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>";

/// The icon store of a new document in `dir`, holding `bytes`; returns the
/// store and the URL path of the image.
fn store_with(dir: &TempDir, bytes: &[u8], mime: &str) -> (IconStore, String) {
    let (store, _) = Store::create(dir.path().join("portolano.json"), "Portolano").unwrap();
    let icons = store.icons();
    let Icon::Custom { sha256, .. } = icons.add(bytes, mime).unwrap() else {
        panic!("the icon store returns custom icons");
    };
    let extension = if mime == "image/png" { "png" } else { "svg" };
    (icons, format!("/{sha256}.{extension}"))
}

fn header(response: &Response<Vec<u8>>, name: header::HeaderName) -> Option<&str> {
    response
        .headers()
        .get(name)
        .map(|value| value.to_str().unwrap())
}

#[test]
fn stored_image_is_served_with_its_type() {
    let dir = TempDir::new().unwrap();
    let (icons, path) = store_with(&dir, PNG, "image/png");

    let response = respond(Some(&icons), &path);

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.body(), PNG);
    assert_eq!(header(&response, header::CONTENT_TYPE), Some("image/png"));
    assert_eq!(
        header(&response, header::X_CONTENT_TYPE_OPTIONS),
        Some("nosniff")
    );
}

#[test]
fn svg_is_served_with_a_policy_that_forbids_everything() {
    let dir = TempDir::new().unwrap();
    let (icons, path) = store_with(&dir, SVG, "image/svg+xml");

    let response = respond(Some(&icons), &path);

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        header(&response, header::CONTENT_SECURITY_POLICY),
        Some("default-src 'none'")
    );
}

#[test]
fn name_that_is_not_a_stored_image_is_refused() {
    let dir = TempDir::new().unwrap();
    let (icons, _) = store_with(&dir, PNG, "image/png");
    let upper = format!("/{}.png", "A".repeat(64));
    let short = format!("/{}.png", "a".repeat(63));
    let other_type = format!("/{}.html", "a".repeat(64));

    for path in [
        "/../portolano.json",
        "/portolano.json",
        &upper,
        &short,
        &other_type,
    ] {
        assert_eq!(
            respond(Some(&icons), path).status(),
            StatusCode::BAD_REQUEST,
            "{path}"
        );
    }
}

#[test]
fn image_missing_from_the_store_is_not_found() {
    let dir = TempDir::new().unwrap();
    let (icons, _) = store_with(&dir, PNG, "image/png");

    let response = respond(Some(&icons), &format!("/{}.png", "0".repeat(64)));

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[test]
fn nothing_is_served_without_an_open_document() {
    let response = respond(None, &format!("/{}.png", "0".repeat(64)));

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
