//! `connect`: plans printed without secrets, and real launches of probe
//! scripts standing in for `ssh` and FreeRDP (never the real clients).

use predicates::str::contains;
use pretty_assertions::assert_eq;

use crate::testutil::*;

/// A workspace whose `ssh` and FreeRDP are probe scripts.
fn with_probes(ssh_body: &str, freerdp_body: &str) -> Workspace {
    let workspace = Workspace::new();
    let ssh = workspace.script("probe-ssh", ssh_body);
    let freerdp = workspace.script("probe-freerdp", freerdp_body);
    workspace.write_config(&format!(
        "[ssh]\nprogram = \"{}\"\n\n[rdp]\nclient = \"freerdp\"\nfreerdp_program = \"{}\"\n",
        ssh.display(),
        freerdp.display()
    ));
    workspace
}

fn with_connections(workspace: &Workspace) {
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
        .run_with_stdin(
            &[
                "add",
                "rdp",
                "/dc01",
                "--host",
                "192.0.2.10",
                "--user",
                "administrator",
                "--password-stdin",
            ],
            &format!("{SENTINEL}\n"),
        )
        .success();
}

#[test]
fn print_shows_the_ssh_plan_without_the_secret() {
    let workspace = with_probes("exit 0", "exit 0");
    with_connections(&workspace);
    let assert = workspace.run(&["connect", "--print", "/web01"]).success();
    let output = all_output(&assert);
    assert!(output.contains("-l admin"));
    assert!(output.contains("web01.example.com"));
    assert!(output.contains("<vault:"));
    assert!(!output.contains(SENTINEL));
}

#[test]
fn print_shows_the_rdp_plan_without_the_secret() {
    let workspace = with_probes("exit 0", "exit 0");
    with_connections(&workspace);
    let assert = workspace.run(&["connect", "--print", "/dc01"]).success();
    let output = all_output(&assert);
    assert!(output.contains("/v:192.0.2.10:3389"));
    assert!(output.contains("/from-stdin:force"));
    assert!(!output.contains("/p:"));
    assert!(!output.contains(SENTINEL));
}

#[test]
fn print_of_a_new_window_relaunches_with_the_same_document() {
    let workspace = with_probes("exit 0", "exit 0");
    with_connections(&workspace);
    let output = workspace.ok(&["connect", "--print", "--new-window", "/web01"]);
    assert!(output.contains(&format!("--file {}", workspace.doc().display())));
    assert!(output.contains("--vault file"));
    assert!(output.contains(" connect @"));
    assert!(!output.contains("PORTOLANO_VAULT_PASSWORD"));
    assert!(!output.contains(TEST_MASTER_PASSWORD));
}

/// A probe that fails with 9 if the master password reached it, by name or
/// by value (DEC-023.1), and otherwise exits with 0.
const NO_MASTER_PASSWORD: &str = "[ -z \"${PORTOLANO_VAULT_PASSWORD+set}\" ] || exit 9\n\
     env | grep -qF test-only-master-password && exit 9\n\
     cat > /dev/null\n\
     exit 0";

#[test]
fn ssh_does_not_inherit_the_master_password() {
    let workspace = with_probes(NO_MASTER_PASSWORD, "exit 0");
    with_connections(&workspace);
    workspace.run(&["connect", "/web01"]).success();
}

#[test]
fn freerdp_does_not_inherit_the_master_password() {
    let workspace = with_probes("exit 0", NO_MASTER_PASSWORD);
    with_connections(&workspace);
    workspace.run(&["connect", "--wait", "/dc01"]).success();
}

#[test]
fn ssh_gets_the_password_through_askpass_and_its_exit_code_is_returned() {
    let workspace = with_probes(
        "answer=$(\"$SSH_ASKPASS\" \"admin@web01.example.com's password: \")\n\
         [ \"$answer\" = \"$EXPECTED_SECRET\" ] || exit 1\n\
         exit 7",
        "exit 0",
    );
    with_connections(&workspace);
    workspace
        .cmd()
        .env("EXPECTED_SECRET", SENTINEL)
        .args(["connect", "/web01"])
        .assert()
        .code(7);
}

#[test]
fn freerdp_reads_the_password_on_stdin() {
    let workspace = with_probes(
        "exit 0",
        "read -r answer\n[ \"$answer\" = \"$EXPECTED_SECRET\" ] || exit 1\nexit 0",
    );
    with_connections(&workspace);
    workspace
        .cmd()
        .env("EXPECTED_SECRET", SENTINEL)
        .args(["connect", "--wait", "/dc01"])
        .assert()
        .success();
}

#[test]
fn detached_client_reports_its_log() {
    let workspace = with_probes("exit 0", "cat > /dev/null\nexit 0");
    with_connections(&workspace);
    let assert = workspace.run(&["connect", "/dc01"]).success();
    assert!(stderr(&assert).contains("log del client: "));
}

#[test]
fn credential_asked_at_launch_without_a_terminal_lets_the_client_ask() {
    let workspace = with_probes("exit 0", "exit 0");
    workspace.ok(&[
        "add",
        "ssh",
        "/web01",
        "--host",
        "web01.example.com",
        "--cred",
        "prompt",
    ]);
    workspace
        .run(&["connect", "/web01"])
        .success()
        .stderr(contains("utente e password li chiederà il client"));
}

#[test]
fn missing_client_is_a_connection_error() {
    let workspace = Workspace::new();
    workspace.write_config("[ssh]\nprogram = \"/nonexistent/ssh\"\n");
    workspace.ok(&["add", "ssh", "/web01", "--host", "web01.example.com"]);
    workspace.run(&["connect", "/web01"]).code(6);
}

#[test]
fn group_writable_client_is_refused_before_launch() {
    use std::os::unix::fs::PermissionsExt;

    let workspace = with_probes("exit 0", "exit 0");
    with_connections(&workspace);
    let ssh = workspace.path("probe-ssh");
    std::fs::set_permissions(&ssh, std::fs::Permissions::from_mode(0o775)).unwrap();
    workspace
        .run(&["connect", "/web01"])
        .code(6)
        .stderr(contains("chmod go-w"));
}

#[test]
fn connect_to_a_folder_is_a_connection_error() {
    let workspace = Workspace::new();
    workspace.ok(&["add", "folder", "/a"]);
    let assert = workspace.run(&["connect", "/a"]).code(6);
    assert_eq!(stderr(&assert).trim(), "errore: '/a' non è una connessione");
}

#[test]
fn new_window_receives_options_from_the_environment_as_absolute_paths() {
    let workspace = with_probes("exit 0", "exit 0");
    with_connections(&workspace);
    let dir = std::fs::canonicalize(workspace.dir.path()).unwrap();
    let output = stdout(
        workspace
            .env_cmd()
            .args(["connect", "--print", "--new-window", "/web01"])
            .assert()
            .success(),
    );
    for expected in [
        format!("--file {}", dir.join("doc.json").display()),
        "--vault file".to_owned(),
        format!("--vault-file {}", dir.join("vault.json").display()),
        format!("--config {}", dir.join("config.toml").display()),
    ] {
        assert!(output.contains(&expected), "{expected} in {output}");
    }
    assert!(!output.contains(TEST_MASTER_PASSWORD));
}
