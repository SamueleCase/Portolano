//! Tests of plan execution with harmless programs (`/bin/sh` scripts): never
//! `ssh`, `ssh-add` or FreeRDP.

use std::fs;
use std::os::unix::fs::PermissionsExt;

use portolano_core::AppConfig;
use tempfile::TempDir;

use portolano_core::vault::MemoryVault;

use super::*;
use crate::plan::AskpassEntry;

/// A script in a temporary directory that receives the output file as `$1`.
pub(super) fn script(dir: &TempDir, body: &str) -> PathBuf {
    let path = dir.path().join("probe.sh");
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    path
}

pub(super) fn run(
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

/// The askpass program of the test plans; never executed.
pub(super) const ASKPASS_PROGRAM: &str = "/opt/example/bin/portolano";

pub(super) fn password_askpass(secret: &SecretRef) -> AskpassSpec {
    AskpassSpec {
        program: PathBuf::from(ASKPASS_PROGRAM),
        entries: vec![AskpassEntry {
            matcher: PromptMatcher::Password {
                user: "admin".into(),
                host: "web01.example.com".into(),
            },
            secret: secret.clone(),
        }],
    }
}

pub(super) fn execute_in(
    plan: &LaunchPlan,
    vault: &MemoryVault,
    temp: &TempDir,
) -> Result<i32, ConnectError> {
    let files = TempFiles::new(temp.path().join("files"));
    prepare_with(plan, vault, &AppConfig::default(), files)?.launch()
}

mod detached;
mod prepare;
mod run;
