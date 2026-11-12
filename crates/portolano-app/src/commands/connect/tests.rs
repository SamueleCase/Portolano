use serde_json::json;

use super::*;
use crate::commands::test_support::{SENTINEL, TestApp};
use crate::error::ErrorCode;

#[test]
fn preview_shows_the_plan_with_secrets_as_references() {
    let app = TestApp::with_document();
    let web01 = app.add(json!({
        "parent": null,
        "name": "web01",
        "kind": {
            "type": "connection", "protocol": "rdp", "host": "192.0.2.10", "port": null,
            "credential": { "mode": "inline", "username": "operatore", "domain": null }, "settings": {},
        },
        "secrets": { "password": SENTINEL },
    }));

    let preview = preview(&app.state, web01);

    // Without FreeRDP on the machine the plan cannot be made: either way
    // nothing may contain the password.
    match preview {
        Ok(preview) => {
            assert!(preview.command.contains("192.0.2.10"));
            assert!(!preview.command.contains(SENTINEL));
        }
        Err(error) => {
            assert_eq!(error.code, ErrorCode::Connect);
            assert!(!error.message.contains(SENTINEL));
        }
    }
}

#[test]
fn preview_of_a_folder_is_a_connect_error() {
    let app = TestApp::with_document();
    let folder = app.add(json!({
        "parent": null,
        "name": "Prod",
        "kind": { "type": "folder", "credential": { "mode": "inherit" } },
    }));

    let error = preview(&app.state, folder).unwrap_err();

    assert_eq!(error.code, ErrorCode::Connect);
}
