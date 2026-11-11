//! Usage errors, help, `--json` and logs.

use predicates::str::contains;

use crate::document::sample;
use crate::testutil::*;

#[test]
fn json_on_a_write_command_is_a_usage_error() {
    Workspace::new()
        .run(&["--json", "add", "folder", "/x"])
        .code(2)
        .stderr(contains("--json"));
}

#[test]
fn missing_argument_is_a_usage_error_in_italian() {
    Workspace::new()
        .run(&["add", "ssh", "/web01"])
        .code(2)
        .stderr(contains("manca l'argomento obbligatorio --host"));
}

#[test]
fn unknown_command_is_a_usage_error_in_italian() {
    Workspace::new()
        .run(&["conect", "/web01"])
        .code(2)
        .stderr(contains("comando sconosciuto 'conect'"));
}

#[test]
fn help_lists_every_command_of_the_spec_in_italian() {
    let help = Workspace::empty().ok(&["--help"]);
    assert!(help.contains("Uso: portolano"));
    assert!(help.contains("Mostra l'aiuto"));
    for command in [
        "init", "tree", "list", "show", "add", "edit", "move", "delete", "secret", "connect",
        "import", "validate", "config",
    ] {
        assert!(help.contains(&format!("\n  {command} ")), "{command}");
    }
}

#[test]
fn vault_file_with_the_keychain_backend_is_a_usage_error() {
    let workspace = Workspace::new();
    workspace
        .bare_cmd()
        .args(["tree", "--vault", "keychain", "--vault-file"])
        .arg(workspace.path("vault.json"))
        .assert()
        .code(2)
        .stderr(contains("vale solo con il backend 'file'"));
}

#[test]
fn tree_json_with_depth_is_a_usage_error() {
    sample()
        .run(&["tree", "--json", "--depth", "1"])
        .code(2)
        .stderr(contains("--depth non si può usare insieme a --json"));
}
#[test]
fn connect_print_with_json_is_a_usage_error() {
    sample()
        .run(&["--json", "connect", "--print", "/Clienti/ACME/web01"])
        .code(2)
        .stderr(contains("--json"));
}

#[test]
fn verbose_logs_contain_no_secret() {
    let workspace = Workspace::new();
    let added = workspace
        .cmd()
        .args([
            "-vvv",
            "add",
            "cred",
            "/Admin",
            "--user",
            "admin",
            "--password-stdin",
        ])
        .write_stdin(format!("{SENTINEL}\n"))
        .assert()
        .success();
    let shown = workspace
        .run(&["-v", "show", "--resolved", "/Admin"])
        .success();
    for output in [all_output(&added), all_output(&shown)] {
        assert!(!output.contains(SENTINEL));
        assert!(!output.contains(TEST_MASTER_PASSWORD));
    }
}
