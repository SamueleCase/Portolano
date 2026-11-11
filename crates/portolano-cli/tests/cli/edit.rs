//! `edit` and `move`.

use predicates::str::contains;
use pretty_assertions::assert_eq;

use crate::testutil::*;

#[test]
fn edit_changes_fields_and_settings() {
    let workspace = Workspace::new();
    workspace.ok(&[
        "add",
        "ssh",
        "/web01",
        "--host",
        "web01.example.com",
        "--tag",
        "old",
    ]);
    workspace.ok(&[
        "edit",
        "/web01",
        "--name",
        "web02",
        "--host",
        "192.0.2.20",
        "--set",
        "x11=true",
        "--set",
        "options={\"Compression\":\"yes\"}",
        "--untag",
        "old",
        "--color",
        "#00FF00",
        "--prop",
        "Ticket=INC-1",
    ]);
    let web02 = node(&workspace, "/web02");
    assert_eq!(web02["host"], "192.0.2.20");
    assert_eq!(web02["settings"]["x11"], true);
    assert_eq!(web02["settings"]["options"]["Compression"], "yes");
    assert!(web02.get("tags").is_none());
    assert_eq!(web02["color"], "#00FF00");
    workspace.ok(&["edit", "/web02", "--unset", "x11", "--unprop", "Ticket"]);
    assert!(node(&workspace, "/web02")["settings"].get("x11").is_none());
}

#[test]
fn edit_with_settings_refused_by_the_protocol_exits_with_4() {
    let workspace = Workspace::new();
    workspace.ok(&["add", "ssh", "/web01", "--host", "web01.example.com"]);
    workspace
        .run(&["edit", "/web01", "--set", "x11=forse"])
        .code(4);
}

#[test]
fn edit_with_an_invalid_color_exits_with_4() {
    let workspace = Workspace::new();
    workspace.ok(&["add", "folder", "/a"]);
    workspace.run(&["edit", "/a", "--color", "verde"]).code(4);
}

#[test]
fn edit_cred_links_a_credential() {
    let workspace = Workspace::new();
    workspace.add_admin();
    workspace.ok(&["add", "ssh", "/web01", "--host", "web01.example.com"]);
    workspace.ok(&["edit", "/web01", "--cred", "/Admin"]);
    assert_eq!(node(&workspace, "/web01")["credential"]["mode"], "by_id");
}

#[test]
fn edit_user_and_domain_make_an_inline_credential() {
    let workspace = Workspace::new();
    workspace.ok(&[
        "add",
        "rdp",
        "/dc01",
        "--host",
        "192.0.2.10",
        "--cred",
        "prompt",
    ]);
    workspace.ok(&["edit", "/dc01", "--user", "operator", "--domain", "ACME"]);
    let credential = &node(&workspace, "/dc01")["credential"];
    assert_eq!(credential["mode"], "inline");
    assert_eq!(credential["username"], "operator");
    assert_eq!(credential["domain"], "ACME");
}

#[test]
fn edit_cred_with_user_is_a_usage_error() {
    let workspace = Workspace::new();
    workspace.ok(&["add", "ssh", "/web01", "--host", "web01.example.com"]);
    workspace
        .run(&["edit", "/web01", "--cred", "prompt", "--user", "x"])
        .code(2);
}

fn abc() -> Workspace {
    let workspace = Workspace::new();
    for name in ["/a", "/b", "/c"] {
        workspace.ok(&["add", "folder", name]);
    }
    workspace
}

#[test]
fn move_before_reorders_in_the_same_folder() {
    let workspace = abc();
    workspace.ok(&["move", "/c", "/", "--before", "/a"]);
    assert_eq!(names(&workspace, "/"), vec!["c", "a", "b"]);
}

#[test]
fn move_after_reorders_in_the_same_folder() {
    let workspace = abc();
    workspace.ok(&["move", "/a", "/", "--after", "/b"]);
    assert_eq!(names(&workspace, "/"), vec!["b", "a", "c"]);
}

#[test]
fn move_to_a_position_counts_from_zero() {
    let workspace = abc();
    workspace.ok(&["move", "/a", "/", "--position", "2"]);
    assert_eq!(names(&workspace, "/"), vec!["b", "c", "a"]);
}

#[test]
fn move_into_a_folder_prints_the_new_path() {
    abc()
        .run(&["move", "/a", "/b"])
        .success()
        .stdout(contains("/b/a"));
}

#[test]
fn move_into_a_descendant_is_refused() {
    let workspace = abc();
    workspace.ok(&["move", "/a", "/b"]);
    let assert = workspace.run(&["move", "/b", "/b/a"]).code(1);
    assert_eq!(
        stderr(&assert).trim(),
        "errore: impossibile spostare '/b' dentro sé stesso o un suo discendente ('/b/a')"
    );
}
