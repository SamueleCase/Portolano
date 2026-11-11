//! Execution of launch plans (SPEC §7.2, §7.3), in two phases (DEC-016.2):
//! [`prepare`] runs the preliminary steps and starts the askpass server,
//! [`PreparedLaunch::launch`] starts the interactive program. [`execute`]
//! does both, for the CLI; an app that runs `ssh` in its own terminal calls
//! [`prepare`] and spawns the program itself.

mod temp_files;

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};

use portolano_core::{AppConfig, Secret, SecretRef, Vault};

use self::temp_files::TempFiles;
use crate::ConnectError;
use crate::env::{Os, find_ssh_add};
use crate::plan::{AskpassSpec, LaunchPlan, LaunchStep, PromptMatcher};
use crate::ssh::askpass::{
    ASKPASS_ENV, ASKPASS_SOCK_ENV, ASKPASS_TOKEN_ENV, AskpassServer, SSH_ADD_STDIN_KEY_PATH,
    ServedSecret,
};
use crate::terminal::launcher;

/// Exit code of `ssh-add` when it cannot reach the agent.
const SSH_ADD_NO_AGENT: i32 = 2;

/// Runs the steps of `plan`, reading the referenced secrets from `vault`
/// only when needed and passing them through stdin or askpass.
///
/// Returns the exit code of the last waited program, or 0 if it detached.
///
/// # Errors
/// [`ConnectError::MissingSecret`], [`ConnectError::AgentUnavailable`],
/// [`ConnectError::Spawn`], [`ConnectError::Io`], [`ConnectError::Askpass`],
/// vault errors.
pub fn execute(
    plan: &LaunchPlan,
    vault: &dyn Vault,
    config: &AppConfig,
) -> Result<i32, ConnectError> {
    prepare(plan, vault, config)?.launch()
}

/// Runs the preliminary steps of `plan` (keys added to `ssh-agent`,
/// temporary files) and prepares its last step, the interactive program:
/// its secrets are read from `vault` and, for `ssh`, the askpass server is
/// started.
///
/// The result tells which program to start, with which arguments and
/// environment; while it lives the askpass server answers. Dropping it
/// closes the socket, removes its directory and zeroes the secrets.
///
/// # Errors
/// As [`execute`]; [`ConnectError::InvalidPlan`] for a plan that does not
/// end with its only `Run` or `OpenInTerminal` step;
/// [`ConnectError::ClientFailed`] if `ssh-add` fails.
pub fn prepare(
    plan: &LaunchPlan,
    vault: &dyn Vault,
    config: &AppConfig,
) -> Result<PreparedLaunch, ConnectError> {
    prepare_with(plan, vault, config, TempFiles::per_user())
}

fn prepare_with(
    plan: &LaunchPlan,
    vault: &dyn Vault,
    config: &AppConfig,
    mut temp_files: TempFiles,
) -> Result<PreparedLaunch, ConnectError> {
    let Some((last, preliminary)) = plan.steps.split_last() else {
        return Err(invalid_plan("il piano non contiene passi"));
    };
    for step in preliminary {
        match step {
            LaunchStep::SshAddKey {
                key,
                passphrase,
                lifetime_s,
                askpass_program,
            } => add_key(
                vault,
                key,
                passphrase.as_ref(),
                *lifetime_s,
                askpass_program,
            )?,
            LaunchStep::WriteTempFile {
                name,
                contents,
                mode,
            } => temp_files.write(name, contents, *mode)?,
            LaunchStep::Run { .. } | LaunchStep::OpenInTerminal { .. } => {
                return Err(invalid_plan("un comando può essere solo l'ultimo passo"));
            }
        }
    }
    prepare_step(last, vault, config, &temp_files)
}

/// Prepares a `Run` or `OpenInTerminal` step.
fn prepare_step(
    step: &LaunchStep,
    vault: &dyn Vault,
    config: &AppConfig,
    temp_files: &TempFiles,
) -> Result<PreparedLaunch, ConnectError> {
    match step {
        LaunchStep::Run {
            program,
            args,
            env,
            stdin_secret,
            askpass,
            wait,
        } => PreparedLaunch::new(
            program.clone(),
            args.iter().map(|arg| temp_files.substitute(arg)).collect(),
            env.clone(),
            stdin_secret
                .as_ref()
                .map(|reference| read_secret(vault, reference))
                .transpose()?,
            askpass
                .as_ref()
                .map(|spec| askpass_secrets(vault, spec))
                .transpose()?,
            *wait,
        ),
        LaunchStep::OpenInTerminal { argv } => {
            let launcher = launcher(Os::current(), &config.terminal, argv)?;
            PreparedLaunch::new(
                launcher.program,
                launcher.args,
                Vec::new(),
                None,
                None,
                true,
            )
        }
        LaunchStep::SshAddKey { .. } | LaunchStep::WriteTempFile { .. } => Err(invalid_plan(
            "l'ultimo passo deve essere un comando (Run o OpenInTerminal)",
        )),
    }
}

/// Adds a key from the vault to `ssh-agent`: `ssh-add -t <lifetime> -`, the
/// key on stdin and its passphrase through askpass, never a file (SPEC §7.2).
fn add_key(
    vault: &dyn Vault,
    key: &SecretRef,
    passphrase: Option<&SecretRef>,
    lifetime_s: u32,
    askpass_program: &Path,
) -> Result<(), ConnectError> {
    if std::env::var_os("SSH_AUTH_SOCK").is_none_or(|socket| socket.is_empty()) {
        return Err(ConnectError::AgentUnavailable);
    }
    let program = find_ssh_add().ok_or_else(|| ConnectError::ClientNotFound {
        client: "ssh-add".to_owned(),
        hint: "serve per le chiavi salvate nel vault: installa OpenSSH".to_owned(),
    })?;
    let askpass = match passphrase {
        Some(reference) => Some(Askpass {
            program: askpass_program.to_path_buf(),
            secrets: vec![ServedSecret {
                matcher: PromptMatcher::Passphrase {
                    key_path: SSH_ADD_STDIN_KEY_PATH.to_owned(),
                },
                secret: read_secret(vault, reference)?,
            }],
        }),
        None => None,
    };
    let args = vec!["-t".to_owned(), lifetime_s.to_string(), "-".to_owned()];
    let key = read_secret(vault, key)?;
    match PreparedLaunch::new(program, args, Vec::new(), Some(key), askpass, true)?.launch()? {
        0 => Ok(()),
        SSH_ADD_NO_AGENT => Err(ConnectError::AgentUnavailable),
        code => Err(ConnectError::ClientFailed {
            program: "ssh-add".to_owned(),
            code,
        }),
    }
}

/// The askpass program of a step and the secrets it may answer with, read
/// from the vault.
struct Askpass {
    program: PathBuf,
    secrets: Vec<ServedSecret>,
}

fn askpass_secrets(vault: &dyn Vault, spec: &AskpassSpec) -> Result<Askpass, ConnectError> {
    let secrets = spec
        .entries
        .iter()
        .map(|entry| {
            Ok(ServedSecret {
                matcher: entry.matcher.clone(),
                secret: read_secret(vault, &entry.secret)?,
            })
        })
        .collect::<Result<_, ConnectError>>()?;
    Ok(Askpass {
        program: spec.program.clone(),
        secrets,
    })
}

fn read_secret(vault: &dyn Vault, reference: &SecretRef) -> Result<Secret, ConnectError> {
    vault
        .get(reference)?
        .ok_or_else(|| ConnectError::MissingSecret {
            reference: reference.clone(),
        })
}

/// The interactive program of a plan, ready to start (DEC-016.2).
///
/// It does not implement `Debug`: it holds secrets. Dropping it stops the
/// askpass server, removes its socket and directory and zeroes the secrets.
pub struct PreparedLaunch {
    program: PathBuf,
    args: Vec<String>,
    env: Vec<(String, String)>,
    stdin_secret: Option<Secret>,
    wait: bool,
    askpass: Option<AskpassServer>,
}

impl PreparedLaunch {
    fn new(
        program: PathBuf,
        args: Vec<String>,
        mut env: Vec<(String, String)>,
        stdin_secret: Option<Secret>,
        askpass: Option<Askpass>,
        wait: bool,
    ) -> Result<PreparedLaunch, ConnectError> {
        let askpass = match askpass {
            Some(askpass) if !askpass.secrets.is_empty() => {
                let server = AskpassServer::start(askpass.secrets)?;
                env.extend(server.child_env(&askpass.program));
                Some(server)
            }
            _ => None,
        };
        Ok(PreparedLaunch {
            program,
            args,
            env,
            stdin_secret,
            wait,
            askpass,
        })
    }

    /// The program to start.
    pub fn program(&self) -> &Path {
        &self.program
    }

    /// Its arguments; they never contain secrets.
    pub fn args(&self) -> &[String] {
        &self.args
    }

    /// Variables to add to the inherited environment, including those of
    /// askpass; they never contain secrets.
    pub fn env(&self) -> &[(String, String)] {
        &self.env
    }

    /// The secret to write to the program's stdin, followed by a newline,
    /// before closing it (the FreeRDP password). A caller that starts the
    /// program itself must not copy it elsewhere.
    pub fn stdin_secret(&self) -> Option<&Secret> {
        self.stdin_secret.as_ref()
    }

    /// Whether the caller should wait for the program to exit.
    pub fn wait(&self) -> bool {
        self.wait
    }

    /// Starts the program, writes the stdin secret and, if required, waits.
    /// Returns its exit code (`128 + signal` if killed), or 0 if it detached.
    ///
    /// # Errors
    /// [`ConnectError::Spawn`] if the program cannot start,
    /// [`ConnectError::Io`] if writing stdin or waiting fails.
    pub fn launch(self) -> Result<i32, ConnectError> {
        // The askpass server must outlive the program it answers, so a
        // program that uses it is always waited for.
        let detached = !self.wait && self.askpass.is_none();
        let mut command = self.command(detached);
        let mut child = command.spawn().map_err(|source| ConnectError::Spawn {
            program: self.program.clone(),
            source,
        })?;
        if let (Some(secret), Some(mut stdin)) = (&self.stdin_secret, child.stdin.take()) {
            write_secret(&mut stdin, secret).map_err(|source| ConnectError::Io {
                context: format!("scrittura sullo stdin di '{}'", self.program.display()),
                source,
            })?;
        }
        if detached {
            // Reaped in the background, so it does not linger as a zombie
            // while this process lives; when this process exits, the system
            // adopts it.
            std::thread::spawn(move || child.wait());
            return Ok(0);
        }
        let status = child.wait().map_err(|source| ConnectError::Io {
            context: format!("attesa di '{}'", self.program.display()),
            source,
        })?;
        Ok(exit_code(status))
    }

    /// The command to spawn. A detached program writes nowhere (it would
    /// scribble over the terminal after `connect` returns) and gets its own
    /// process group, so that Ctrl-C in the terminal does not reach it.
    fn command(&self, detached: bool) -> Command {
        let mut command = Command::new(&self.program);
        command.args(&self.args);
        // A child must never inherit askpass variables of another session.
        for name in [ASKPASS_ENV, ASKPASS_SOCK_ENV, ASKPASS_TOKEN_ENV] {
            command.env_remove(name);
        }
        command.envs(self.env.iter().map(|(key, value)| (key, value)));
        if self.stdin_secret.is_some() {
            command.stdin(Stdio::piped());
        }
        if detached {
            command.stdout(Stdio::null()).stderr(Stdio::null());
            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                command.process_group(0);
            }
        }
        command
    }
}

/// Writes `secret` and a newline (unless it already ends with one, as a
/// private key does); dropping `stdin` afterwards closes it. A program that
/// exits without reading is not an error: its exit code tells the story.
fn write_secret(stdin: &mut impl Write, secret: &Secret) -> io::Result<()> {
    let value = secret.expose();
    let result = stdin.write_all(value.as_bytes()).and_then(|()| {
        if value.ends_with('\n') {
            Ok(())
        } else {
            stdin.write_all(b"\n")
        }
    });
    match result {
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        other => other,
    }
}

fn exit_code(status: ExitStatus) -> i32 {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return 128 + signal;
        }
    }
    status.code().unwrap_or(1)
}

fn invalid_plan(reason: &str) -> ConnectError {
    ConnectError::InvalidPlan {
        reason: reason.to_owned(),
    }
}

#[cfg(test)]
#[cfg(unix)]
mod tests;
