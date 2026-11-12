use pretty_assertions::assert_eq;

use super::*;
use crate::commands::test_support::TestApp;
use crate::error::ErrorCode;

const PNG: &[u8] = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR";

fn choose(app: &TestApp, name: &str, bytes: &[u8]) {
    let path = app.dir.path().join(name);
    fs::write(&path, bytes).unwrap();
    app.state.chosen().set(FileKind::Icon, &path);
}

#[test]
fn chosen_image_is_stored_and_listed() {
    let app = TestApp::with_document();
    choose(&app, "server.png", PNG);

    let icon = add(&app.state).unwrap();

    assert!(matches!(&icon, IconView::Custom { mime, .. } if mime == "image/png"));
    assert_eq!(custom(&app.state).unwrap(), vec![icon]);
}

#[test]
fn image_larger_than_the_limit_is_refused_before_reading_it() {
    let app = TestApp::with_document();
    let mut big = PNG.to_vec();
    big.resize(usize::try_from(MAX_ICON_BYTES).unwrap() + 1, 0);
    choose(&app, "enorme.png", &big);

    let error = add(&app.state).unwrap_err();

    assert_eq!(error.code, ErrorCode::Invalid);
    assert!(custom(&app.state).unwrap().is_empty());
}

#[test]
fn file_that_is_not_an_image_is_refused() {
    let app = TestApp::with_document();
    choose(&app, "note.txt", b"non un'immagine");

    let error = add(&app.state).unwrap_err();

    assert_eq!(error.code, ErrorCode::Invalid);
}

#[test]
fn icons_need_an_open_document() {
    let app = TestApp::new();

    assert_eq!(custom(&app.state).unwrap_err().code, ErrorCode::NoDocument);
}
