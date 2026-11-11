//! Tests of plan execution with harmless programs (`/bin/sh` scripts): never
//! `ssh`, `ssh-add` or FreeRDP.

use std::fs;
use std::os::unix::fs::PermissionsExt;

use portolano_core::AppConfig;
use pretty_assertions::assert_eq;
use tempfile::TempDir;

use super::*;
use crate::plan::AskpassEntry;
use crate::testutil::{SENTINEL, TestVault};

/// A script in a temporary directory that receives the output file as `$1`.
fn script(dir: &TempDir, body: &str) -> PathBuf {
    let path = dir.path().join("probe.sh");
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    path
}

fn run(
    program: PathBuf,
    args: Vec<String>,
    stdin_secret: Option<SecretRef>,
    askpass: Option<AskpassSpec>,
) -> LaunchPlan {
    LaunchPlan {
        steps: vec![LaunchStep::Run {
            program,
            args,
            env: vec![("PORTOLANO_TEST".into(), "on".into())],
            stdin_secret,
            askpass,
            wait: true,
        }],
        warnings: vec![],
    }
}

fn password_askpass(secret: &SecretRef) -> AskpassSpec {
    AskpassSpec {
        entries: vec![AskpassEntry {
            matcher: PromptMatcher::Password {
                user: "admin".into(),
                host: "web01.example.com".into(),
            },
            secret: secret.clone(),
        }],
    }
}

fn execute_in(plan: &LaunchPlan, vault: &TestVault, temp: &TempDir) -> Result<i32, ConnectError> {
    let files = TempFiles::new(temp.path().join("files"));
    prepare_with(plan, vault, &AppConfig::default(), files)?.launch()
}

#[test]
fn exit_code_of_the_program_is_returned() {
    let temp = tempfile::tempdir().unwrap();
    let plan = run(
        PathBuf::from("/bin/sh"),
        vec!["-c".into(), "exit 7".into()],
        None,
        None,
    );

    let code = execute_in(&plan, &TestVault::default(), &temp).unwrap();

    assert_eq!(code, 7);
}

#[test]
fn stdin_secret_is_written_with_a_newline_and_closed() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("stdin.txt");
    let reference = SecretRef::new();
    let vault = TestVault::with(&[(&reference, SENTINEL)]);
    // `cat` ends only when stdin is closed.
    let program = script(&temp, "cat > \"$1\"");
    let plan = run(
        program,
        vec![output.to_string_lossy().into_owned()],
        Some(reference),
        None,
    );

    let code = execute_in(&plan, &vault, &temp).unwrap();

    assert_eq!(code, 0);
    assert_eq!(fs::read_to_string(output).unwrap(), format!("{SENTINEL}\n"));
}

#[test]
fn secrets_never_reach_argv_or_environment_of_the_child() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("seen.txt");
    let password = SecretRef::new();
    let stdin = SecretRef::new();
    let vault = TestVault::with(&[(&password, SENTINEL), (&stdin, SENTINEL)]);
    let program = script(
        &temp,
        "out=\"$1\"; env > \"$out\"; printf '%s\\n' \"$@\" >> \"$out\"; cat > /dev/null",
    );
    let plan = run(
        program,
        vec![output.to_string_lossy().into_owned()],
        Some(stdin),
        Some(password_askpass(&password)),
    );

    execute_in(&plan, &vault, &temp).unwrap();

    let seen = fs::read_to_string(output).unwrap();
    assert!(!seen.contains(SENTINEL));
    assert!(seen.contains("PORTOLANO_ASKPASS=1"));
    assert!(seen.contains("SSH_ASKPASS_REQUIRE=force"));
    assert!(seen.contains("PORTOLANO_TEST=on"));
}

#[test]
fn askpass_socket_is_removed_when_the_program_exits() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("socket.txt");
    let password = SecretRef::new();
    let vault = TestVault::with(&[(&password, SENTINEL)]);
    let program = script(
        &temp,
        "test -S \"$PORTOLANO_ASKPASS_SOCK\" && echo \"$PORTOLANO_ASKPASS_SOCK\" > \"$1\"",
    );
    let plan = run(
        program,
        vec![output.to_string_lossy().into_owned()],
        None,
        Some(password_askpass(&password)),
    );

    let code = execute_in(&plan, &vault, &temp).unwrap();

    let socket = fs::read_to_string(output).unwrap();
    assert_eq!(code, 0);
    assert!(!Path::new(socket.trim()).exists());
}

#[test]
fn prepared_launch_exposes_the_askpass_environment_until_dropped() {
    let temp = tempfile::tempdir().unwrap();
    let password = SecretRef::new();
    let vault = TestVault::with(&[(&password, SENTINEL)]);
    let plan = run(
        PathBuf::from("/usr/bin/true"),
        vec![],
        None,
        Some(password_askpass(&password)),
    );

    let prepared = prepare_with(
        &plan,
        &vault,
        &AppConfig::default(),
        TempFiles::new(temp.path().to_path_buf()),
    )
    .unwrap();
    let socket = prepared
        .env()
        .iter()
        .find(|(key, _)| key == ASKPASS_SOCK_ENV)
        .map(|(_, value)| PathBuf::from(value))
        .unwrap();
    let existed = socket.exists();
    drop(prepared);

    assert!(existed);
    assert!(!socket.exists());
    assert!(!socket.parent().unwrap().exists());
}

#[test]
fn prepared_launch_holds_the_stdin_secret_and_no_secret_in_args_or_env() {
    let temp = tempfile::tempdir().unwrap();
    let reference = SecretRef::new();
    let vault = TestVault::with(&[(&reference, SENTINEL)]);
    let plan = run(
        PathBuf::from("/usr/bin/true"),
        vec!["/v:192.0.2.10".into()],
        Some(reference),
        None,
    );

    let prepared = prepare_with(
        &plan,
        &vault,
        &AppConfig::default(),
        TempFiles::new(temp.path().to_path_buf()),
    )
    .unwrap();

    assert_eq!(prepared.program(), Path::new("/usr/bin/true"));
    assert_eq!(prepared.stdin_secret().map(Secret::expose), Some(SENTINEL));
    assert!(prepared.args().iter().all(|arg| !arg.contains(SENTINEL)));
    assert!(
        prepared
            .env()
            .iter()
            .all(|(_, value)| !value.contains(SENTINEL))
    );
}

#[test]
fn missing_secret_is_reported_before_starting_anything() {
    let temp = tempfile::tempdir().unwrap();
    let reference = SecretRef::new();
    let plan = run(
        PathBuf::from("/usr/bin/true"),
        vec![],
        Some(reference.clone()),
        None,
    );

    let result = execute_in(&plan, &TestVault::default(), &temp);

    assert!(
        matches!(result, Err(ConnectError::MissingSecret { reference: ref missing }) if *missing == reference),
        "{result:?}"
    );
}

#[test]
fn temporary_file_path_replaces_its_placeholder() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("copy.txt");
    let plan = LaunchPlan {
        steps: vec![
            LaunchStep::WriteTempFile {
                name: "dc01.rdp".into(),
                contents: "full address:s:192.0.2.10:3389\n".into(),
                mode: 0o600,
            },
            LaunchStep::Run {
                program: PathBuf::from("/bin/cp"),
                args: vec![
                    "{temp:dc01.rdp}".into(),
                    output.to_string_lossy().into_owned(),
                ],
                env: vec![],
                stdin_secret: None,
                askpass: None,
                wait: true,
            },
        ],
        warnings: vec![],
    };

    let code = execute_in(&plan, &TestVault::default(), &temp).unwrap();

    assert_eq!(code, 0);
    assert_eq!(
        fs::read_to_string(output).unwrap(),
        "full address:s:192.0.2.10:3389\n"
    );
}

#[test]
fn detached_program_returns_zero_without_waiting() {
    let temp = tempfile::tempdir().unwrap();
    let mut plan = run(
        PathBuf::from("/bin/sh"),
        vec!["-c".into(), "sleep 1; exit 3".into()],
        None,
        None,
    );
    if let Some(LaunchStep::Run { wait, .. }) = plan.steps.last_mut() {
        *wait = false;
    }

    let code = execute_in(&plan, &TestVault::default(), &temp).unwrap();

    assert_eq!(code, 0);
}

#[test]
fn empty_plan_is_rejected() {
    let temp = tempfile::tempdir().unwrap();

    let result = execute_in(&LaunchPlan::default(), &TestVault::default(), &temp);

    assert!(
        matches!(result, Err(ConnectError::InvalidPlan { .. })),
        "{result:?}"
    );
}

#[test]
fn command_before_the_last_step_is_rejected_without_running() {
    let temp = tempfile::tempdir().unwrap();
    let marker = temp.path().join("ran");
    let first = run(
        PathBuf::from("/usr/bin/touch"),
        vec![marker.to_string_lossy().into_owned()],
        None,
        None,
    );
    let mut plan = run(PathBuf::from("/usr/bin/true"), vec![], None, None);
    plan.steps.insert(0, first.steps[0].clone());

    let result = execute_in(&plan, &TestVault::default(), &temp);

    assert!(
        matches!(result, Err(ConnectError::InvalidPlan { .. })),
        "{result:?}"
    );
    assert!(!marker.exists());
}

#[test]
fn missing_program_is_a_spawn_error() {
    let temp = tempfile::tempdir().unwrap();
    let plan = run(temp.path().join("missing"), vec![], None, None);

    let result = execute_in(&plan, &TestVault::default(), &temp);

    assert!(
        matches!(result, Err(ConnectError::Spawn { .. })),
        "{result:?}"
    );
}
