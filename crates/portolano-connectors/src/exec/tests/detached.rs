//! Programs that detach from `connect` (FreeRDP by default).

use std::fs;
use std::path::PathBuf;

use portolano_core::AppConfig;
use portolano_core::vault::MemoryVault;
use pretty_assertions::assert_eq;

use super::*;
use crate::testutil::{SENTINEL, vault_with};

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

    let code = execute_in(&plan, &MemoryVault::default(), &temp).unwrap();

    assert_eq!(code, 0);
}

#[test]
fn detached_program_gets_its_stdin_secret_and_runs_to_completion() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("detached.txt");
    let reference = SecretRef::new();
    let vault = vault_with(&[(&reference, SENTINEL)]);
    let program = script(
        &temp,
        "echo noise; cat > \"$1.tmp\" && mv \"$1.tmp\" \"$1\"",
    );
    let mut plan = run(
        program,
        vec![output.to_string_lossy().into_owned()],
        Some(reference),
        None,
    );
    if let Some(LaunchStep::Run { wait, .. }) = plan.steps.last_mut() {
        *wait = false;
    }

    let code = execute_in(&plan, &vault, &temp).unwrap();
    wait_for(&output);

    assert_eq!(code, 0);
    assert_eq!(fs::read_to_string(output).unwrap(), format!("{SENTINEL}\n"));
}

#[test]
fn detached_program_without_secret_reads_an_empty_stdin() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("stdin.txt");
    // With an inherited terminal or pipe, `cat` would never end.
    let program = script(&temp, "cat > \"$1.tmp\" && mv \"$1.tmp\" \"$1\"");
    let plan = detach(run(
        program,
        vec![output.to_string_lossy().into_owned()],
        None,
        None,
    ));

    execute_in(&plan, &MemoryVault::default(), &temp).unwrap();
    wait_for(&output);

    assert_eq!(fs::read_to_string(output).unwrap(), "");
}

#[test]
fn detached_program_writes_its_errors_to_a_private_log() {
    let temp = tempfile::tempdir().unwrap();
    let done = temp.path().join("done");
    let program = script(&temp, "echo 'connessione rifiutata' >&2; touch \"$1\"");
    let plan = detach(run(
        program,
        vec![done.to_string_lossy().into_owned()],
        None,
        None,
    ));
    let files = TempFiles::new(temp.path().join("files"));

    let prepared =
        prepare_with(&plan, &MemoryVault::default(), &AppConfig::default(), files).unwrap();
    let log = prepared.log_path().map(Path::to_path_buf).unwrap();
    prepared.launch().unwrap();
    wait_for(&done);

    assert_eq!(log.parent(), Some(temp.path().join("files").as_path()));
    assert_eq!(fs::read_to_string(&log).unwrap(), "connessione rifiutata\n");
}

#[test]
fn waited_program_has_no_log() {
    let temp = tempfile::tempdir().unwrap();
    let plan = run(PathBuf::from("/usr/bin/true"), vec![], None, None);
    let files = TempFiles::new(temp.path().join("files"));

    let prepared =
        prepare_with(&plan, &MemoryVault::default(), &AppConfig::default(), files).unwrap();

    assert!(prepared.log_path().is_none());
    assert!(!temp.path().join("files").exists());
}

fn detach(mut plan: LaunchPlan) -> LaunchPlan {
    if let Some(LaunchStep::Run { wait, .. }) = plan.steps.last_mut() {
        *wait = false;
    }
    plan
}

/// Waits up to ten seconds for a detached program to create `path`.
fn wait_for(path: &Path) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !path.exists() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}
