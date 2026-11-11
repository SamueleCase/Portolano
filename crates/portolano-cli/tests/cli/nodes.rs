//! `add`.

use predicates::str::contains;
use pretty_assertions::assert_eq;

use crate::testutil::*;

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
fn add_ssh_with_a_command_running_option_is_refused() {
    let workspace = Workspace::new();
    workspace
        .run(&[
            "add",
            "ssh",
            "/web01",
            "--host",
            "web01.example.com",
            "-o",
            "ProxyCommand=nc %h %p",
        ])
        .code(4)
        .stderr(contains("~/.ssh/config"));
}

#[test]
fn edit_set_of_a_command_running_option_is_refused() {
    let workspace = Workspace::new();
    workspace.ok(&["add", "ssh", "/web01", "--host", "web01.example.com"]);
    workspace
        .run(&[
            "edit",
            "/web01",
            "--set",
            "options={\"LocalCommand\":\"id\"}",
        ])
        .code(4)
        .stderr(contains("'LocalCommand'"));
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

#[test]
fn missing_parent_folder_is_not_found() {
    Workspace::new().run(&["add", "folder", "/a/b"]).code(3);
}

#[test]
fn parents_option_creates_the_missing_folders() {
    let workspace = Workspace::new();
    workspace.ok(&["add", "folder", "-p", "/a/b"]);
    assert_eq!(node(&workspace, "/a")["type"], "folder");
}

#[test]
fn name_used_by_a_sibling_is_refused() {
    let workspace = Workspace::new();
    workspace.ok(&["add", "folder", "/a"]);
    workspace
        .run(&["add", "folder", "/a"])
        .code(1)
        .stderr(contains("esiste già"));
}
