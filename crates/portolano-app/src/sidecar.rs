//! The `portolano` CLI shipped with the app (SPEC-UI §2, §4.10): the askpass
//! program of `ssh` and the program that a new terminal window runs.
//!
//! It sits next to the executable of the app, in development
//! (`target/debug/`, built by `beforeDevCommand`) as in the bundle
//! (`Portolano.app/Contents/MacOS/`), so one rule finds it in both.

use std::io;
use std::path::{Path, PathBuf};

/// The file name of the CLI.
const CLI_NAME: &str = "portolano";

/// Why the CLI cannot be used.
#[derive(Debug, thiserror::Error)]
pub enum SidecarError {
    /// The path of the running executable is unknown.
    #[error("percorso dell'eseguibile dell'app non determinabile")]
    UnknownExecutable(#[source] io::Error),
    /// The CLI is not next to the app.
    #[error(
        "binario della CLI mancante: '{}' (in sviluppo si compila con `cargo build -p portolano-cli`)",
        .path.display()
    )]
    Missing {
        /// Where it should be.
        path: PathBuf,
    },
}

/// The CLI next to the running executable.
///
/// # Errors
/// [`SidecarError::UnknownExecutable`], [`SidecarError::Missing`].
pub fn cli_path() -> Result<PathBuf, SidecarError> {
    let executable = std::env::current_exe().map_err(SidecarError::UnknownExecutable)?;
    cli_next_to(&executable)
}

/// The CLI in the directory of `executable`, if it is an executable file.
fn cli_next_to(executable: &Path) -> Result<PathBuf, SidecarError> {
    let file_name = format!("{CLI_NAME}{}", std::env::consts::EXE_SUFFIX);
    let path = executable.with_file_name(file_name);
    if is_executable_file(&path) {
        Ok(path)
    } else {
        Err(SidecarError::Missing { path })
    }
}

#[cfg(unix)]
fn is_executable_file(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    path.metadata()
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable_file(path: &Path) -> bool {
    path.is_file()
}

#[cfg(test)]
#[cfg(unix)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    use pretty_assertions::assert_eq;
    use tempfile::TempDir;

    use super::*;

    fn file_in(dir: &TempDir, name: &str, mode: u32) -> PathBuf {
        let path = dir.path().join(name);
        fs::write(&path, "#!/bin/sh\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(mode)).unwrap();
        path
    }

    #[test]
    fn cli_next_to_the_app_is_found() {
        let dir = TempDir::new().unwrap();
        let app = file_in(&dir, "portolano-app", 0o755);
        let cli = file_in(&dir, "portolano", 0o755);

        assert_eq!(cli_next_to(&app).unwrap(), cli);
    }

    #[test]
    fn missing_cli_is_reported_with_its_expected_path() {
        let dir = TempDir::new().unwrap();
        let app = file_in(&dir, "portolano-app", 0o755);

        let error = cli_next_to(&app).unwrap_err();

        assert!(
            matches!(&error, SidecarError::Missing { path } if *path == dir.path().join("portolano"))
        );
        assert!(error.to_string().contains("cargo build -p portolano-cli"));
    }

    #[test]
    fn cli_that_is_not_executable_counts_as_missing() {
        let dir = TempDir::new().unwrap();
        let app = file_in(&dir, "portolano-app", 0o755);
        file_in(&dir, "portolano", 0o644);

        assert!(matches!(
            cli_next_to(&app),
            Err(SidecarError::Missing { .. })
        ));
    }
}
