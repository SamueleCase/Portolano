//! `init`, `tree`, `list`, `show`, `config`, global options and exit codes.

use predicates::str::contains;
use pretty_assertions::assert_eq;
use serde_json::Value;

use crate::testutil::*;

fn sample() -> Workspace {
    let workspace = Workspace::new();
    workspace.ok(&["add", "folder", "-p", "/Clienti/ACME"]);
    workspace.add_admin();
    workspace.ok(&[
        "add",
        "ssh",
        "/Clienti/ACME/web01",
        "--host",
        "web01.example.com",
        "--cred",
        "/Admin",
    ]);
    workspace.ok(&[
        "add",
        "rdp",
        "/Clienti/ACME/dc01",
        "--host",
        "192.0.2.10",
        "--user",
        "administrator",
        "--domain",
        "ACME",
    ]);
    workspace
}

#[test]
fn init_creates_the_document_and_refuses_to_overwrite_it() {
    let workspace = Workspace::empty();
    workspace
        .run(&["init", "--name", "Infra"])
        .success()
        .stdout(contains("Infra"));
    workspace
        .run(&["init"])
        .code(1)
        .stderr(contains("esiste già"));
    workspace.run(&["init", "--force"]).success();
    assert!(workspace.path("doc.json.bak").exists());
}

#[test]
fn missing_document_suggests_init() {
    let workspace = Workspace::empty();
    workspace
        .run(&["tree"])
        .code(1)
        .stderr(contains("portolano init"));
}

#[test]
fn tree_draws_folders_connections_and_credentials() {
    let text = sample().ok(&["tree"]);
    assert_eq!(
        text,
        "Portolano\n\
         ├── 📁 Clienti\n\
         │   └── 📁 ACME\n\
         │       ├── [ssh] web01  admin@web01.example.com:22\n\
         │       └── [rdp] dc01   ACME\\administrator@192.0.2.10\n\
         └── 🔑 Admin             admin\n"
    );
}

#[test]
fn tree_of_a_folder_with_depth_and_ids() {
    let workspace = sample();
    let text = workspace.ok(&["tree", "/Clienti", "--depth", "1", "--ids"]);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with("📁 Clienti  @"));
    assert!(lines[1].starts_with("└── 📁 ACME  @"));
}

#[test]
fn tree_json_is_the_model_without_secrets() {
    let workspace = sample();
    let json: Value = serde_json::from_str(&workspace.ok(&["tree", "--json"])).unwrap();
    assert_eq!(json["format"], "portolano");
    assert!(!json.to_string().contains(SENTINEL));
}

#[test]
fn list_filters_by_type() {
    let workspace = sample();
    let text = workspace.ok(&["list", "--type", "rdp"]);
    assert_eq!(
        text,
        "TIPO  PERCORSO            RIEPILOGO\n\
         rdp   /Clienti/ACME/dc01  ACME\\administrator@192.0.2.10\n"
    );
}

#[test]
fn list_json_has_one_entry_per_node() {
    let workspace = sample();
    let json: Value = serde_json::from_str(&workspace.ok(&["list", "--json"])).unwrap();
    assert_eq!(json.as_array().unwrap().len(), 5);
}

#[test]
fn show_resolved_tells_presence_without_the_secret() {
    let workspace = sample();
    let assert = workspace
        .run(&["show", "/Clienti/ACME/web01", "--resolved"])
        .success();
    let output = all_output(&assert);
    assert!(output.contains("credenziale /Admin"));
    assert!(output.contains("presente nel vault"));
    assert!(!output.contains(SENTINEL));
}

#[test]
fn show_json_includes_the_resolved_credential() {
    let workspace = sample();
    let text = workspace.ok(&["show", "--json", "--resolved", "/Clienti/ACME/web01"]);
    let json: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(json["resolved"]["password"], "present");
    assert_eq!(json["path"], "/Clienti/ACME/web01");
}

#[test]
fn unknown_path_exits_with_3() {
    sample().run(&["show", "/Clienti/nessuno"]).code(3);
}

#[test]
fn ambiguous_id_prefix_lists_the_candidates() {
    let workspace = Workspace::new();
    let doc = workspace.doc_text();
    let mut json: Value = serde_json::from_str(&doc).unwrap();
    let node = |id: &str, name: &str| {
        serde_json::json!({
            "id": id, "name": name, "type": "folder",
            "created": "2026-11-10T10:00:00Z", "modified": "2026-11-10T10:00:00Z"
        })
    };
    json["nodes"] = serde_json::json!([
        node("abcdef00-0000-4000-8000-000000000001", "a"),
        node("abcdef00-0000-4000-8000-000000000002", "b"),
    ]);
    std::fs::write(workspace.doc(), json.to_string()).unwrap();
    workspace
        .run(&["show", "@abcdef00"])
        .code(3)
        .stderr(contains("candidati"));
}

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
fn config_set_show_and_path() {
    let workspace = Workspace::new();
    workspace.ok(&["config", "set", "rdp.client", "freerdp"]);
    workspace.ok(&["config", "set", "terminal.app", "iterm"]);
    let shown = workspace.ok(&["config", "show"]);
    assert!(shown.contains("client = \"freerdp\""));
    assert!(shown.contains("app = \"iterm\""));
    let path = workspace.ok(&["config", "path"]);
    assert_eq!(path.trim(), workspace.path("config.toml").to_string_lossy());
}

#[test]
fn config_set_refuses_invalid_values_and_unknown_keys() {
    let workspace = Workspace::new();
    workspace
        .run(&["config", "set", "rdp.client", "telepatia"])
        .code(4);
    workspace
        .run(&["config", "set", "vnc.program", "x"])
        .code(4);
    assert!(!workspace.path("config.toml").exists());
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

#[test]
fn tree_json_of_a_path_is_that_node() {
    let workspace = sample();
    let text = workspace.ok(&["tree", "--json", "/Clienti/ACME"]);
    let json: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(json["name"], "ACME");
    assert_eq!(json["children"].as_array().unwrap().len(), 2);
}

#[test]
fn tree_json_with_depth_is_a_usage_error() {
    sample()
        .run(&["tree", "--json", "--depth", "1"])
        .code(2)
        .stderr(contains("--depth non si può usare insieme a --json"));
}
