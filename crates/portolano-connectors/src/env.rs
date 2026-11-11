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

/// `ssh` is looked up in the `PATH` unless the configuration names a program.
pub(crate) const DEFAULT_SSH_PROGRAM: &str = "ssh";

const SSH_SECTION: &str = "ssh";
const SSH_PROGRAM_KEY: &str = "program";
const RDP_SECTION: &str = "rdp";
const FREERDP_PROGRAM_KEY: &str = "freerdp_program";

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
    /// A program set in the configuration (`[ssh] program`,
    /// `[rdp] freerdp_program`) must be an absolute path to a regular,
    /// executable file that neither its group nor others can write
    /// (DEC-024.2): it receives the passwords, so it must not be replaceable
    /// by someone else.
    ///
    /// # Errors
    /// [`ConnectError::InvalidConfig`] for an invalid protocol section or a
    /// configured program that fails the checks above,
    /// [`ConnectError::Io`] if the running executable cannot be determined.
    pub fn discover(config: &AppConfig) -> Result<Environment, ConnectError> {
        let ssh_config = SshConfig::from_app_config(config)?;
        let rdp_config = RdpConfig::from_app_config(config)?;
        let path = path_dirs();
        let os = Os::current();

        let ssh = match ssh_config.configured_program() {
            Some(program) => Some(trusted(SSH_SECTION, SSH_PROGRAM_KEY, program)?),
            None => find_first(&[DEFAULT_SSH_PROGRAM], &path),
        };
        let freerdp = match rdp_config.configured_freerdp_program() {
            Some(program) => Some(trusted(RDP_SECTION, FREERDP_PROGRAM_KEY, program)?),
            None => {
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
            ssh,
            ssh_add: find_ssh_add(),
            freerdp,
            windows_app,
            self_exe,
        })
    }
}

/// `program` of the section `[section]` as a [`ConnectError::InvalidConfig`]
/// if it fails [`check_trusted_program`].
fn trusted(section: &str, key: &str, program: &Path) -> Result<PathBuf, ConnectError> {
    check_trusted_program(key, program)
        .map(|()| program.to_path_buf())
        .map_err(|reason| ConnectError::InvalidConfig {
            section: section.to_owned(),
            reason,
        })
}

/// Checks that the program configured as `key` is an absolute path
/// (without touching the filesystem), as [`Protocol::validate_config`]
/// requires.
///
/// [`Protocol::validate_config`]: crate::Protocol::validate_config
///
/// # Errors
/// A description of the problem, in Italian.
pub(crate) fn check_program_path(key: &str, program: &Path) -> Result<(), String> {
    if program.is_absolute() {
        Ok(())
    } else {
        Err(format!(
            "'{key}' deve essere un percorso assoluto, non '{}'",
            program.display()
        ))
    }
}

/// Checks that the program configured as `key` is an absolute path to a
/// regular, executable file that the group and others cannot write.
/// Symbolic links are followed: what matters is the file that runs.
///
/// # Errors
/// A description of the problem, in Italian.
fn check_trusted_program(key: &str, program: &Path) -> Result<(), String> {
    check_program_path(key, program)?;
    let shown = program.display();
    let metadata = program
        .metadata()
        .map_err(|_| format!("'{key}': '{shown}' non esiste o non è accessibile"))?;
    if !metadata.is_file() {
        return Err(format!("'{key}': '{shown}' non è un file regolare"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode();
        if mode & 0o022 != 0 {
            return Err(format!(
                "'{key}': '{shown}' è scrivibile dal gruppo o da altri: \
                 togli quel permesso (chmod go-w)"
            ));
        }
        if mode & 0o111 == 0 {
            return Err(format!("'{key}': '{shown}' non è eseguibile"));
        }
    }
    Ok(())
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
#[cfg(unix)]
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
    fn trusted_program_accepts_a_private_executable() {
        let dir = tempfile::tempdir().unwrap();
        let ssh = program(dir.path(), "my-ssh", 0o755);

        assert_eq!(check_trusted_program("program", &ssh), Ok(()));
    }

    #[test]
    fn trusted_program_must_be_absolute() {
        let error = check_trusted_program("program", Path::new("bin/ssh")).unwrap_err();

        assert!(error.contains("percorso assoluto"), "{error}");
    }

    #[test]
    fn trusted_program_must_exist() {
        let dir = tempfile::tempdir().unwrap();

        let error = check_trusted_program("program", &dir.path().join("missing")).unwrap_err();

        assert!(error.contains("non esiste"), "{error}");
    }

    #[test]
    fn trusted_program_must_be_a_regular_file() {
        let dir = tempfile::tempdir().unwrap();

        let error = check_trusted_program("program", dir.path()).unwrap_err();

        assert!(error.contains("non è un file regolare"), "{error}");
    }

    #[test]
    fn trusted_program_must_not_be_writable_by_the_group_or_others() {
        let dir = tempfile::tempdir().unwrap();
        for mode in [0o775, 0o757] {
            let ssh = program(dir.path(), &format!("ssh-{mode:o}"), mode);

            let error = check_trusted_program("program", &ssh).unwrap_err();

            assert!(error.contains("chmod go-w"), "{error}");
        }
    }

    #[test]
    fn trusted_program_must_be_executable() {
        let dir = tempfile::tempdir().unwrap();
        let ssh = program(dir.path(), "ssh", 0o644);

        let error = check_trusted_program("program", &ssh).unwrap_err();

        assert!(error.contains("non è eseguibile"), "{error}");
    }

    fn config_with(text: &str) -> AppConfig {
        let mut config = AppConfig::default();
        let table: toml::Table = toml::from_str(text).unwrap();
        config.protocols.extend(table);
        config
    }

    #[test]
    fn discover_uses_a_trusted_configured_program() {
        let dir = tempfile::tempdir().unwrap();
        let ssh = program(dir.path(), "probe-ssh", 0o700);
        let freerdp = program(dir.path(), "probe-freerdp", 0o500);
        let config = config_with(&format!(
            "[ssh]\nprogram = {:?}\n[rdp]\nfreerdp_program = {:?}\n",
            ssh.display().to_string(),
            freerdp.display().to_string()
        ));

        let env = Environment::discover(&config).unwrap();

        assert_eq!(env.ssh, Some(ssh));
        assert_eq!(env.freerdp, Some(freerdp));
    }

    #[test]
    fn discover_refuses_a_group_writable_freerdp() {
        let dir = tempfile::tempdir().unwrap();
        let freerdp = program(dir.path(), "probe-freerdp", 0o775);
        let config = config_with(&format!(
            "[rdp]\nfreerdp_program = {:?}\n",
            freerdp.display().to_string()
        ));

        let error = Environment::discover(&config).unwrap_err();

        assert!(
            matches!(&error, ConnectError::InvalidConfig { section, reason }
                if section == "rdp" && reason.contains("freerdp_program")),
            "{error}"
        );
    }

    #[test]
    fn discover_refuses_a_relative_ssh_program() {
        let config = config_with("[ssh]\nprogram = \"bin/ssh\"\n");

        let error = Environment::discover(&config).unwrap_err();

        assert!(
            matches!(&error, ConnectError::InvalidConfig { section, .. } if section == "ssh"),
            "{error}"
        );
    }
}
