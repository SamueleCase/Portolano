//! `add`, `edit`, `move` and `delete`.

use predicates::str::contains;
use pretty_assertions::assert_eq;
use serde_json::Value;

use crate::testutil::*;

fn node(workspace: &Workspace, path: &str) -> Value {
    let text = workspace.ok(&["show", "--json", path]);
    serde_json::from_str::<Value>(&text).unwrap()["node"].clone()
}

fn names(workspace: &Workspace, path: &str) -> Vec<String> {
    let text = workspace.ok(&["list", "--json", path]);
    let entries: Value = serde_json::from_str(&text).unwrap();
    entries
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| entry["depth"] == 0)
        .map(|entry| entry["name"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn add_ssh_with_options_jump_and_key_file() {
    let workspace = Workspace::new();
    workspace.ok(&["add", "ssh", "/bastion", "--host", "bastion.example.com"]);
    workspace.ok(&[
        "add",
        "ssh",
        "/web01",
        "--host",
        "web01.example.com",
        "--port",
        "2222",
        "--user",
        "deploy",
        "--jump",
        "/bastion",
        "--jump",
        "ops@jump.example.com:2200",
        "--key",
        "~/.ssh/id_ed25519",
        "--agent-forwarding",
        "--keepalive",
        "30",
        "-o",
        "StrictHostKeyChecking=yes",
        "--tag",
        "prod",
    ]);
    let web01 = node(&workspace, "/web01");
    assert_eq!(web01["port"], 2222);
    assert_eq!(web01["credential"]["mode"], "inline");
    assert_eq!(web01["credential"]["username"], "deploy");
    let settings = &web01["settings"];
    assert_eq!(
        settings["jump_hosts"][1]["address"],
        "ops@jump.example.com:2200"
    );
    assert!(settings["jump_hosts"][0]["$ref"].is_string());
    assert_eq!(settings["options"]["IdentityFile"], "~/.ssh/id_ed25519");
    assert_eq!(settings["options"]["IdentitiesOnly"], "yes");
    assert_eq!(settings["keepalive_s"], 30);
    assert_eq!(web01["tags"][0], "prod");
}

#[test]
fn jump_to_a_node_that_is_not_ssh_is_refused() {
    let workspace = Workspace::new();
    workspace.ok(&["add", "folder", "/Rete"]);
    workspace
        .run(&[
            "add",
            "ssh",
            "/web01",
            "--host",
            "web01.example.com",
            "--jump",
            "/Rete",
        ])
        .code(4);
    workspace
        .run(&[
            "add",
            "ssh",
            "/web01",
            "--host",
            "web01.example.com",
            "--jump",
            "/nessuno",
        ])
        .code(3);
}

#[test]
fn add_rdp_with_gateway_and_password_from_stdin() {
    let workspace = Workspace::new();
    workspace.add_admin();
    workspace
        .run_with_stdin(
            &[
                "add",
                "rdp",
                "/dc01",
                "--host",
                "192.0.2.10",
                "--user",
                "administrator",
                "--domain",
                "ACME",
                "--password-stdin",
                "--resolution",
                "1920x1080",
                "--gateway",
                "gw.example.com:443",
                "--gateway-cred",
                "/Admin",
            ],
            &format!("{SENTINEL}\n"),
        )
        .success();
    let dc01 = node(&workspace, "/dc01");
    assert_eq!(dc01["credential"]["domain"], "ACME");
    assert!(dc01["credential"]["password"].is_string());
    assert_eq!(dc01["settings"]["gateway"]["port"], 443);
    assert_eq!(dc01["settings"]["gateway"]["credential"]["mode"], "by_id");
    assert_eq!(dc01["settings"]["resolution"]["width"], 1920);
    assert!(!workspace.doc_text().contains(SENTINEL));
}

#[test]
fn add_with_invalid_settings_exits_with_4() {
    let workspace = Workspace::new();
    workspace
        .run(&[
            "add",
            "rdp",
            "/dc01",
            "--host",
            "192.0.2.10",
            "--scale",
            "120",
        ])
        .code(4);
}

#[test]
fn add_into_a_missing_folder_needs_parents() {
    let workspace = Workspace::new();
    workspace.run(&["add", "folder", "/a/b"]).code(3);
    workspace.ok(&["add", "folder", "-p", "/a/b"]);
    workspace
        .run(&["add", "folder", "/a/b"])
        .code(1)
        .stderr(contains("esiste già"));
}

#[test]
fn add_cred_with_key_from_stdin() {
    let workspace = Workspace::new();
    workspace
        .run_with_stdin(
            &["add", "cred", "/Deploy", "--user", "deploy", "--key-stdin"],
            "-----BEGIN OPENSSH PRIVATE KEY-----\nsentinel-key-body\n-----END OPENSSH PRIVATE KEY-----\n",
        )
        .success();
    assert_eq!(node(&workspace, "/Deploy")["private_key"]["kind"], "vault");
    assert!(!workspace.doc_text().contains("sentinel-key-body"));
}

#[test]
fn ask_password_without_a_terminal_fails() {
    let workspace = Workspace::new();
    workspace
        .run(&["add", "cred", "/Admin", "--ask-password"])
        .code(1)
        .stderr(contains("nessun terminale"));
}

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
    workspace
        .run(&["edit", "/web01", "--color", "verde"])
        .code(4);
}

#[test]
fn move_and_reorder() {
    let workspace = Workspace::new();
    for name in ["/a", "/b", "/c"] {
        workspace.ok(&["add", "folder", name]);
    }
    workspace.ok(&["move", "/c", "/", "--before", "/a"]);
    assert_eq!(names(&workspace, "/"), vec!["c", "a", "b"]);
    workspace.ok(&["move", "/c", "/", "--position", "2"]);
    assert_eq!(names(&workspace, "/"), vec!["a", "b", "c"]);
    workspace
        .run(&["move", "/a", "/b"])
        .success()
        .stdout(contains("/b/a"));
    workspace.run(&["move", "/b", "/b/a"]).code(1);
}

#[test]
fn delete_needs_confirmation_without_a_terminal() {
    let workspace = Workspace::new();
    workspace.ok(&["add", "folder", "/a"]);
    workspace
        .run(&["delete", "/a"])
        .code(1)
        .stderr(contains("-y"));
    workspace.ok(&["delete", "-y", "/a"]);
    workspace.run(&["show", "/a"]).code(3);
}

#[test]
fn delete_of_a_folder_with_content_needs_recursive() {
    let workspace = Workspace::new();
    workspace.ok(&["add", "folder", "-p", "/a/b"]);
    workspace
        .run(&["delete", "-y", "/a"])
        .code(1)
        .stderr(contains("-r"));
    workspace.ok(&["delete", "-y", "-r", "/a"]);
}

#[test]
fn delete_of_a_referenced_credential_is_refused_then_forced() {
    let workspace = Workspace::new();
    workspace.add_admin();
    workspace.ok(&[
        "add",
        "ssh",
        "/web01",
        "--host",
        "web01.example.com",
        "--cred",
        "/Admin",
    ]);
    workspace
        .run(&["delete", "-y", "/Admin"])
        .code(1)
        .stderr(contains("/web01 -> /Admin"))
        .stderr(contains("--force"));
    workspace
        .run(&["delete", "-y", "--force", "/Admin"])
        .success()
        .stdout(contains("1 segreti cancellati"))
        .stdout(contains("chiedono le credenziali"));
    assert_eq!(node(&workspace, "/web01")["credential"]["mode"], "prompt");
}

#[test]
fn delete_of_a_referenced_jump_host_with_force_leaves_a_dangling_reference() {
    let workspace = Workspace::new();
    workspace.ok(&["add", "ssh", "/bastion", "--host", "bastion.example.com"]);
    workspace.ok(&[
        "add",
        "ssh",
        "/web01",
        "--host",
        "web01.example.com",
        "--jump",
        "/bastion",
    ]);
    workspace.run(&["delete", "-y", "/bastion"]).code(1);
    workspace
        .run(&["delete", "-y", "--force", "/bastion"])
        .success()
        .stdout(contains("restano riferimenti"))
        .stdout(contains("/web01 -> /bastion"));
    workspace
        .run(&["validate"])
        .code(4)
        .stdout(contains("dangling_ref"));
}

#[test]
fn forced_deletion_of_two_credentials_used_by_one_node_reports_no_leftover() {
    let workspace = Workspace::new();
    workspace.ok(&["add", "cred", "-p", "/Credenziali/Admin", "--user", "admin"]);
    workspace.ok(&["add", "cred", "/Credenziali/Gateway", "--user", "gw"]);
    workspace.ok(&[
        "add",
        "rdp",
        "/dc01",
        "--host",
        "192.0.2.10",
        "--cred",
        "/Credenziali/Admin",
        "--gateway",
        "gw.example.com",
        "--gateway-cred",
        "/Credenziali/Gateway",
    ]);
    let output = workspace.ok(&["delete", "-y", "-r", "--force", "/Credenziali"]);
    assert!(output.contains("1 nodi ora chiedono le credenziali"));
    assert!(!output.contains("restano riferimenti"));
}

#[test]
fn credential_path_to_a_folder_exits_with_4() {
    let workspace = Workspace::new();
    workspace.ok(&["add", "folder", "/Rete"]);
    workspace
        .run(&[
            "add",
            "ssh",
            "/web01",
            "--host",
            "web01.example.com",
            "--cred",
            "/Rete",
        ])
        .code(4)
        .stderr(contains("non è una credenziale"));
}

#[test]
fn blank_name_exits_with_4() {
    Workspace::new().run(&["add", "folder", "/ "]).code(4);
}

#[test]
fn tree_deeper_than_the_maximum_exits_with_4() {
    let path = "/l".repeat(portolano_core::MAX_DEPTH + 1);
    Workspace::new()
        .run(&["add", "folder", "-p", &path])
        .code(4)
        .stderr(contains("profondità massima"));
}

#[test]
fn password_with_a_linked_credential_is_a_usage_error() {
    let workspace = Workspace::new();
    workspace.add_admin();
    workspace
        .run_with_stdin(
            &[
                "add",
                "ssh",
                "/web01",
                "--host",
                "web01.example.com",
                "--cred",
                "/Admin",
                "--password-stdin",
            ],
            "x\n",
        )
        .code(2);
}
