//! `config show`, `config set` and `config path`.

use predicates::str::contains;
use pretty_assertions::assert_eq;

use crate::testutil::*;

#[test]
fn config_set_is_shown_by_config_show() {
    let workspace = Workspace::new();
    workspace.ok(&["config", "set", "rdp.client", "freerdp"]);
    workspace.ok(&["config", "set", "terminal.app", "iterm"]);
    let shown = workspace.ok(&["config", "show"]);
    assert!(shown.contains("client = \"freerdp\""));
    assert!(shown.contains("app = \"iterm\""));
}

#[test]
fn config_path_prints_the_configuration_file() {
    let workspace = Workspace::new();
    let path = workspace.ok(&["config", "path"]);
    assert_eq!(path.trim(), workspace.path("config.toml").to_string_lossy());
}

#[test]
fn config_set_refuses_a_value_rejected_by_the_protocol() {
    let workspace = Workspace::new();
    workspace
        .run(&["config", "set", "rdp.client", "telepatia"])
        .code(4);
    assert!(!workspace.path("config.toml").exists());
}

#[test]
fn config_set_refuses_a_relative_program() {
    let workspace = Workspace::new();
    workspace
        .run(&["config", "set", "ssh.program", "bin/ssh"])
        .code(4)
        .stderr(contains("percorso assoluto"));
}

#[test]
fn config_set_refuses_an_unknown_key() {
    let workspace = Workspace::new();
    workspace
        .run(&["config", "set", "vnc.program", "x"])
        .code(4)
        .stderr(contains("chiave di configurazione sconosciuta"));
}

#[test]
fn config_path_works_with_an_invalid_configuration() {
    let workspace = Workspace::new();
    workspace.write_config("questo non è toml = = =\n");
    let path = workspace.ok(&["config", "path"]);
    assert_eq!(path.trim(), workspace.path("config.toml").to_string_lossy());
    workspace
        .run(&["tree"])
        .code(1)
        .stderr(contains("configurazione"));
}
