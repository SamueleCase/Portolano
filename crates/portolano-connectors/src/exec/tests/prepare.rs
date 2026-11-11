//! The prepared launch and the preliminary steps.

use std::fs;
use std::path::PathBuf;

use portolano_core::AppConfig;
use portolano_core::vault::MemoryVault;
use pretty_assertions::assert_eq;

use super::*;
use crate::testutil::{SENTINEL, vault_with};

#[test]
fn ssh_askpass_is_the_program_of_the_plan() {
    let temp = tempfile::tempdir().unwrap();
    let password = SecretRef::new();
    let vault = vault_with(&[(&password, SENTINEL)]);
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

    let askpass = prepared
        .env()
        .iter()
        .find(|(key, _)| key == "SSH_ASKPASS")
        .map(|(_, value)| value.as_str());
    assert_eq!(askpass, Some(ASKPASS_PROGRAM));
}

#[test]
fn prepared_launch_exposes_the_askpass_environment_until_dropped() {
    let temp = tempfile::tempdir().unwrap();
    let password = SecretRef::new();
    let vault = vault_with(&[(&password, SENTINEL)]);
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
    let vault = vault_with(&[(&reference, SENTINEL)]);
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

    let result = execute_in(&plan, &MemoryVault::default(), &temp);

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

    let code = execute_in(&plan, &MemoryVault::default(), &temp).unwrap();

    assert_eq!(code, 0);
    assert_eq!(
        fs::read_to_string(output).unwrap(),
        "full address:s:192.0.2.10:3389\n"
    );
}

#[test]
fn plan_without_temporary_files_does_not_clean_the_directory() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path().join("files");
    fs::create_dir(&dir).unwrap();
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
    let stale = dir.join("old.rdp");
    fs::File::create(&stale)
        .unwrap()
        .set_modified(std::time::SystemTime::now() - std::time::Duration::from_secs(3600))
        .unwrap();
    let plan = run(PathBuf::from("/usr/bin/true"), vec![], None, None);

    execute_in(&plan, &MemoryVault::default(), &temp).unwrap();

    assert!(stale.exists());
}
