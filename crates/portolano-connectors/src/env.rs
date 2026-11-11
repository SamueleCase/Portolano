//! Discovery of the client programs available on this machine.

use std::path::{Path, PathBuf};

use portolano_core::AppConfig;

use crate::ConnectError;
use crate::rdp::RdpConfig;
use crate::ssh::SshConfig;

/// FreeRDP programs, in order of preference (SPEC §7.3).
const FREERDP_PROGRAMS: [&str; 4] = ["sdl-freerdp3", "sdl-freerdp", "xfreerdp3", "xfreerdp"];

/// Directories searched for FreeRDP after the `PATH`: Homebrew installs
/// there, and GUI launches often have a minimal `PATH`.
const FREERDP_EXTRA_DIRS: [&str; 2] = ["/opt/homebrew/bin", "/usr/local/bin"];

/// Windows App bundles, in order of preference (SPEC §7.3).
const WINDOWS_APP_BUNDLES: [&str; 2] = [
    "/Applications/Windows App.app",
    "/Applications/Microsoft Remote Desktop.app",
];

/// Operating system, as far as launching clients is concerned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    /// macOS.
    MacOs,
    /// Linux.
    Linux,
    /// Windows.
    Windows,
    /// Any other system.
    Other,
}

impl Os {
    /// The system this build runs on.
    pub(crate) fn current() -> Os {
        if cfg!(target_os = "macos") {
            Os::MacOs
        } else if cfg!(target_os = "linux") {
            Os::Linux
        } else if cfg!(windows) {
            Os::Windows
        } else {
            Os::Other
        }
    }
}

/// The programs a plan can use. Built by [`Environment::discover`]; tests
/// build it by hand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Environment {
    /// The current operating system.
    pub os: Os,
    /// `ssh`, from `[ssh] program` or the `PATH`.
    pub ssh: Option<PathBuf>,
    /// `ssh-add`.
    pub ssh_add: Option<PathBuf>,
    /// The FreeRDP client, from `[rdp] freerdp_program` or the search of
    /// SPEC §7.3.
    pub freerdp: Option<PathBuf>,
    /// The Windows App (or Microsoft Remote Desktop) bundle.
    pub windows_app: Option<PathBuf>,
    /// The running `portolano` executable, used for askpass and new windows.
    pub self_exe: PathBuf,
}

impl Environment {
    /// Looks for the client programs, honoring the overrides in `config`.
    ///
    /// # Errors
    /// [`ConnectError::InvalidConfig`] for an invalid protocol section,
    /// [`ConnectError::Io`] if the running executable cannot be determined.
    pub fn discover(config: &AppConfig) -> Result<Environment, ConnectError> {
        let ssh_config = SshConfig::from_app_config(config)?;
        let rdp_config = RdpConfig::from_app_config(config)?;
        let path = path_dirs();
        let os = Os::current();

        let freerdp = match rdp_config.freerdp_program {
            Some(program) if !program.as_os_str().is_empty() => locate(&program, &path),
            _ => {
                let mut dirs = path.clone();
                dirs.extend(FREERDP_EXTRA_DIRS.iter().map(PathBuf::from));
                find_first(&FREERDP_PROGRAMS, &dirs)
            }
        };
        let windows_app = match os {
            Os::MacOs => WINDOWS_APP_BUNDLES
                .iter()
                .map(PathBuf::from)
                .find(|bundle| bundle.is_dir()),
            _ => None,
        };
        let self_exe = std::env::current_exe().map_err(|source| ConnectError::Io {
            context: "individuazione dell'eseguibile di Portolano".to_owned(),
            source,
        })?;

        Ok(Environment {
            os,
            ssh: locate(&ssh_config.program, &path),
            ssh_add: find_ssh_add(),
            freerdp,
            windows_app,
            self_exe,
        })
    }
}

/// `ssh-add` in the `PATH`.
pub(crate) fn find_ssh_add() -> Option<PathBuf> {
    find_first(&["ssh-add"], &path_dirs())
}

fn path_dirs() -> Vec<PathBuf> {
    std::env::var_os("PATH")
        .map(|path| std::env::split_paths(&path).collect())
        .unwrap_or_default()
}

/// `program` itself if it is a path, otherwise its first match in `dirs`.
fn locate(program: &Path, dirs: &[PathBuf]) -> Option<PathBuf> {
    if program.components().count() > 1 {
        return is_executable(program).then(|| program.to_path_buf());
    }
    program.to_str().and_then(|name| find_first(&[name], dirs))
}

/// The first executable among `names`, in order of preference; each name is
/// looked up in all `dirs` before the next one.
fn find_first(names: &[&str], dirs: &[PathBuf]) -> Option<PathBuf> {
    names
        .iter()
        .flat_map(|name| dirs.iter().map(move |dir| dir.join(name)))
        .find(|candidate| is_executable(candidate))
}

fn is_executable(path: &Path) -> bool {
    let Ok(metadata) = path.metadata() else {
        return false;
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        metadata.is_file()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    use pretty_assertions::assert_eq;

    use super::*;

    fn program(dir: &Path, name: &str, mode: u32) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, "#!/bin/sh\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(mode)).unwrap();
        path
    }

    #[test]
    fn find_first_prefers_the_earlier_name_over_the_earlier_dir() {
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        program(first.path(), "xfreerdp", 0o755);
        let preferred = program(second.path(), "sdl-freerdp", 0o755);
        let dirs = [first.path().to_path_buf(), second.path().to_path_buf()];

        let found = find_first(&FREERDP_PROGRAMS, &dirs);

        assert_eq!(found, Some(preferred));
    }

    #[test]
    fn find_first_skips_files_that_are_not_executable() {
        let dir = tempfile::tempdir().unwrap();
        program(dir.path(), "sdl-freerdp3", 0o644);
        let executable = program(dir.path(), "xfreerdp3", 0o755);

        let found = find_first(&FREERDP_PROGRAMS, &[dir.path().to_path_buf()]);

        assert_eq!(found, Some(executable));
    }

    #[test]
    fn locate_accepts_an_explicit_path() {
        let dir = tempfile::tempdir().unwrap();
        let ssh = program(dir.path(), "my-ssh", 0o755);

        let found = locate(&ssh, &[]);

        assert_eq!(found, Some(ssh));
    }

    #[test]
    fn locate_rejects_a_missing_explicit_path() {
        let dir = tempfile::tempdir().unwrap();

        let found = locate(&dir.path().join("missing"), &[]);

        assert_eq!(found, None);
    }
}
