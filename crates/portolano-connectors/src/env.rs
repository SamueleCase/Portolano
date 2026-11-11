//! Discovery of the client programs available on this machine.

use std::path::{Path, PathBuf};

use portolano_core::AppConfig;

use crate::ConnectError;
use crate::rdp::RdpConfig;
use crate::safe_path::check_trusted_program;
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
    /// Programs of the configuration that were refused; the matching field
    /// above is `None`. Only the protocol that needs one fails (DEC-026.3).
    pub rejected: Vec<RejectedProgram>,
}

/// A client program set in the configuration that Portolano refuses to run,
/// or a configuration section that could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectedProgram {
    /// The configuration section, such as `"ssh"`.
    pub section: String,
    /// Why, in Italian.
    pub reason: String,
}

impl Environment {
    /// Looks for the client programs, honoring the overrides in `config`.
    ///
    /// A program set in the configuration (`[ssh] program`,
    /// `[rdp] freerdp_program`) must pass the checks of
    /// `safe_path::check_trusted_program` (DEC-024.2, DEC-026.3): it receives
    /// the passwords, so it must not be replaceable by someone else. A
    /// program that fails them, or a section that cannot be read, is listed
    /// in [`Environment::rejected`] instead of failing the discovery, so that
    /// a bad `[ssh]` section does not prevent RDP connections and vice versa.
    ///
    /// # Errors
    /// [`ConnectError::Io`] if the running executable cannot be determined.
    pub fn discover(config: &AppConfig) -> Result<Environment, ConnectError> {
        let path = path_dirs();
        let os = Os::current();
        let mut rejected = Vec::new();

        let ssh_program = SshConfig::from_app_config(config)
            .map(|ssh| ssh.configured_program().map(Path::to_path_buf));
        let ssh = match configured(SSH_SECTION, SSH_PROGRAM_KEY, ssh_program) {
            Ok(Some(program)) => Some(program),
            Ok(None) => find_first(&[DEFAULT_SSH_PROGRAM], &path),
            Err(rejection) => {
                rejected.push(rejection);
                None
            }
        };
        let freerdp_program = RdpConfig::from_app_config(config)
            .map(|rdp| rdp.configured_freerdp_program().map(Path::to_path_buf));
        let freerdp = match configured(RDP_SECTION, FREERDP_PROGRAM_KEY, freerdp_program) {
            Ok(Some(program)) => Some(program),
            Ok(None) => {
                let mut dirs = path.clone();
                dirs.extend(FREERDP_EXTRA_DIRS.iter().map(PathBuf::from));
                find_first(&FREERDP_PROGRAMS, &dirs)
            }
            Err(rejection) => {
                rejected.push(rejection);
                None
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
            rejected,
        })
    }

    /// The refusal of the program of `section`, as the error of a plan that
    /// needs it.
    pub(crate) fn rejection(&self, section: &str) -> Option<ConnectError> {
        self.rejected
            .iter()
            .find(|rejected| rejected.section == section)
            .map(|rejected| ConnectError::InvalidConfig {
                section: rejected.section.clone(),
                reason: rejected.reason.clone(),
            })
    }
}

/// The program set as `[section] key`, checked; `Ok(None)` when the section
/// sets none.
fn configured(
    section: &str,
    key: &str,
    program: Result<Option<PathBuf>, ConnectError>,
) -> Result<Option<PathBuf>, RejectedProgram> {
    let rejected = |reason| RejectedProgram {
        section: section.to_owned(),
        reason,
    };
    let program = program.map_err(|error| {
        rejected(match error {
            ConnectError::InvalidConfig { reason, .. } => reason,
            other => other.to_string(),
        })
    })?;
    match program {
        Some(program) => check_trusted_program(key, &program)
            .map(|()| Some(program))
            .map_err(rejected),
        None => Ok(None),
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
    fn a_rejected_freerdp_leaves_ssh_usable() {
        let dir = tempfile::tempdir().unwrap();
        let ssh = program(dir.path(), "probe-ssh", 0o700);
        let freerdp = program(dir.path(), "probe-freerdp", 0o775);
        let config = config_with(&format!(
            "[ssh]\nprogram = {:?}\n[rdp]\nfreerdp_program = {:?}\n",
            ssh.display().to_string(),
            freerdp.display().to_string()
        ));

        let env = Environment::discover(&config).unwrap();

        assert_eq!(env.ssh, Some(ssh));
        assert_eq!(env.freerdp, None);
        let error = env.rejection("rdp").unwrap();
        assert!(
            matches!(&error, ConnectError::InvalidConfig { section, reason }
                if section == "rdp" && reason.contains("freerdp_program")),
            "{error}"
        );
        assert!(env.rejection("ssh").is_none());
    }

    #[test]
    fn a_rejected_ssh_program_is_recorded_for_ssh_only() {
        let config = config_with("[ssh]\nprogram = \"bin/ssh\"\n");

        let env = Environment::discover(&config).unwrap();

        assert_eq!(env.ssh, None);
        assert!(env.rejection("ssh").is_some());
        assert!(env.rejection("rdp").is_none());
    }

    #[test]
    fn an_unreadable_section_is_recorded_as_rejected() {
        let config = config_with("[ssh]\nprogramm = \"ssh\"\n");

        let env = Environment::discover(&config).unwrap();

        assert_eq!(env.ssh, None);
        assert!(env.rejection("ssh").is_some());
    }
}
