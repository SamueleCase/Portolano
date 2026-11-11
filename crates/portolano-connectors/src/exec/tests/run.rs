//! Running a program: exit code, stdin, environment, invalid plans.

use std::fs;
use std::path::PathBuf;

use portolano_core::vault::MemoryVault;
use pretty_assertions::assert_eq;

use super::*;
use crate::testutil::{SENTINEL, vault_with};

#[test]
fn exit_code_of_the_program_is_returned() {
    let temp = tempfile::tempdir().unwrap();
    let plan = run(
        PathBuf::from("/bin/sh"),
        vec!["-c".into(), "exit 7".into()],
        None,
        None,
    );

    let code = execute_in(&plan, &MemoryVault::default(), &temp).unwrap();

    assert_eq!(code, 7);
}

#[test]
fn stdin_secret_is_written_with_a_newline_and_closed() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("stdin.txt");
    let reference = SecretRef::new();
    let vault = vault_with(&[(&reference, SENTINEL)]);
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
    let vault = vault_with(&[(&password, SENTINEL), (&stdin, SENTINEL)]);
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
    let vault = vault_with(&[(&password, SENTINEL)]);
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
fn empty_plan_is_rejected() {
    let temp = tempfile::tempdir().unwrap();

    let result = execute_in(&LaunchPlan::default(), &MemoryVault::default(), &temp);

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

    let result = execute_in(&plan, &MemoryVault::default(), &temp);

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

    let result = execute_in(&plan, &MemoryVault::default(), &temp);

    assert!(
        matches!(result, Err(ConnectError::Spawn { .. })),
        "{result:?}"
    );
}
