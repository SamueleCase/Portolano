//! Launch plans: what to run to open a connection (SPEC §7.1, §7.2).
//!
//! A plan holds secrets only as [`SecretRef`]s; [`execute`](crate::execute)
//! reads them from the vault.

use std::path::PathBuf;

use portolano_core::SecretRef;

/// The steps that open a connection, run in order, and the warnings to show
/// before running them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LaunchPlan {
    /// Steps in execution order.
    pub steps: Vec<LaunchStep>,
    /// Warnings for the user, in Italian.
    pub warnings: Vec<String>,
}

/// One step of a [`LaunchPlan`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchStep {
    /// Adds a private key from the vault to `ssh-agent`
    /// (`ssh-add -t <lifetime_s> -`, key on stdin, passphrase via askpass).
    SshAddKey {
        /// The key content in the vault.
        key: SecretRef,
        /// The key passphrase in the vault.
        passphrase: Option<SecretRef>,
        /// Lifetime of the key in the agent, in seconds.
        lifetime_s: u32,
        /// The askpass program for the passphrase, as in
        /// [`AskpassSpec::program`].
        askpass_program: PathBuf,
    },
    /// Writes a temporary file, referenced by later steps as `{temp:<name>}`.
    WriteTempFile {
        /// File name inside the per-user temporary directory.
        name: String,
        /// File content; never a secret.
        contents: String,
        /// Unix permission bits, such as `0o600`.
        mode: u32,
    },
    /// Runs a program.
    Run {
        /// The program.
        program: PathBuf,
        /// Arguments; `{temp:<name>}` is replaced by the path of that file.
        args: Vec<String>,
        /// Extra environment variables; never secrets.
        env: Vec<(String, String)>,
        /// Secret written to the program's stdin, followed by a newline.
        stdin_secret: Option<SecretRef>,
        /// Secrets served to `ssh` through `SSH_ASKPASS`.
        askpass: Option<AskpassSpec>,
        /// Wait for the program to exit and return its exit code.
        wait: bool,
    },
    /// Runs `argv` in a new terminal window, as configured in `[terminal]`.
    OpenInTerminal {
        /// The command to run; never contains secrets.
        argv: Vec<String>,
    },
}

/// The secrets the askpass server may answer with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AskpassSpec {
    /// The program that `ssh` runs as `SSH_ASKPASS`: the `portolano`
    /// executable ([`Environment::self_exe`](crate::Environment::self_exe)).
    /// It must check [`is_client_invocation`](crate::ssh::askpass::is_client_invocation)
    /// (`PORTOLANO_ASKPASS=1`) before anything else and then call
    /// [`run_client`](crate::ssh::askpass::run_client).
    pub program: PathBuf,
    /// One entry per expected prompt; each is served at most once.
    pub entries: Vec<AskpassEntry>,
}

/// An expected askpass prompt and the secret that answers it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AskpassEntry {
    /// Which prompt this entry answers.
    pub matcher: PromptMatcher,
    /// The answer, in the vault.
    pub secret: SecretRef,
}

/// A prompt of OpenSSH that askpass can answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptMatcher {
    /// `user@host's password:` or the keyboard-interactive `(user@host) Password:`.
    Password {
        /// The user in the prompt.
        user: String,
        /// The host in the prompt.
        host: String,
    },
    /// `Enter passphrase for key '<path>':`.
    Passphrase {
        /// The key path in the prompt.
        key_path: String,
    },
}

/// Placeholder in [`LaunchStep::Run`] arguments for the path of the file
/// written by the [`LaunchStep::WriteTempFile`] step called `name`.
pub(crate) fn temp_placeholder(name: &str) -> String {
    format!("{{temp:{name}}}")
}

/// Renders a plan for `connect --print`: one quoted command line per step,
/// with secrets shown as `<vault:ref>`. Lines starting with `#` describe what
/// is not a command line (askpass answers, temporary files, new windows).
pub fn render(plan: &LaunchPlan) -> String {
    plan.steps
        .iter()
        .flat_map(render_step)
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_step(step: &LaunchStep) -> Vec<String> {
    match step {
        LaunchStep::SshAddKey {
            key,
            passphrase,
            lifetime_s,
            askpass_program,
        } => {
            let lifetime = lifetime_s.to_string();
            let command = shell_words::join(["ssh-add", "-t", lifetime.as_str(), "-"]);
            let mut line = format!("{command} < {}", vault_ref(key));
            if let Some(passphrase) = passphrase {
                line.push_str(&format!(
                    "  # passphrase via askpass ({}): {}",
                    askpass_program.display(),
                    vault_ref(passphrase)
                ));
            }
            vec![line]
        }
        LaunchStep::WriteTempFile {
            name,
            contents,
            mode,
        } => {
            let header = format!(
                "# file temporaneo {} (permessi {mode:04o}):",
                temp_placeholder(name)
            );
            std::iter::once(header)
                .chain(contents.lines().map(|line| format!("#   {line}")))
                .collect()
        }
        LaunchStep::Run { .. } => render_run(step),
        LaunchStep::OpenInTerminal { argv } => vec![format!(
            "{}  # in una nuova finestra del terminale",
            shell_words::join(argv)
        )],
    }
}

fn render_run(step: &LaunchStep) -> Vec<String> {
    let LaunchStep::Run {
        program,
        args,
        env,
        stdin_secret,
        askpass,
        wait,
    } = step
    else {
        return Vec::new();
    };
    let mut words: Vec<String> = env
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect();
    words.push(program.to_string_lossy().into_owned());
    words.extend(args.iter().cloned());
    let mut line = shell_words::join(&words);
    if let Some(secret) = stdin_secret {
        line.push_str(&format!(" < {}", vault_ref(secret)));
    }
    if !wait {
        line.push_str(" &");
    }
    let mut lines = vec![line];
    if let Some(spec) = askpass {
        lines.extend(spec.entries.iter().map(|entry| {
            let prompt = match &entry.matcher {
                PromptMatcher::Password { user, host } => format!("password di {user}@{host}"),
                PromptMatcher::Passphrase { key_path } => format!("passphrase di '{key_path}'"),
            };
            format!(
                "#   askpass ({}): {prompt} <- {}",
                spec.program.display(),
                vault_ref(&entry.secret)
            )
        }));
    }
    lines
}

fn vault_ref(reference: &SecretRef) -> String {
    format!("<vault:{}>", reference.as_str())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn secret_ref(text: &str) -> SecretRef {
        serde_json::from_value(serde_json::Value::String(text.to_owned())).unwrap()
    }

    #[test]
    fn render_quotes_arguments_and_shows_stdin_secrets_as_references() {
        let plan = LaunchPlan {
            steps: vec![LaunchStep::Run {
                program: PathBuf::from("/opt/homebrew/bin/sdl-freerdp"),
                args: vec!["/v:rdp.example.com:3389".into(), "/u:Mario Rossi".into()],
                env: vec![],
                stdin_secret: Some(secret_ref("ref-1")),
                askpass: None,
                wait: false,
            }],
            warnings: vec![],
        };

        let text = render(&plan);

        assert_eq!(
            text,
            "/opt/homebrew/bin/sdl-freerdp /v:rdp.example.com:3389 '/u:Mario Rossi' < <vault:ref-1> &"
        );
    }

    #[test]
    fn render_lists_askpass_entries_and_ssh_add_steps() {
        let plan = LaunchPlan {
            steps: vec![
                LaunchStep::SshAddKey {
                    key: secret_ref("key-ref"),
                    passphrase: Some(secret_ref("pass-ref")),
                    lifetime_s: 300,
                    askpass_program: PathBuf::from("/usr/local/bin/portolano"),
                },
                LaunchStep::Run {
                    program: PathBuf::from("/usr/bin/ssh"),
                    args: vec![
                        "-p".into(),
                        "22".into(),
                        "--".into(),
                        "web01.example.com".into(),
                    ],
                    env: vec![],
                    stdin_secret: None,
                    askpass: Some(AskpassSpec {
                        program: PathBuf::from("/usr/local/bin/portolano"),
                        entries: vec![AskpassEntry {
                            matcher: PromptMatcher::Password {
                                user: "admin".into(),
                                host: "web01.example.com".into(),
                            },
                            secret: secret_ref("pw-ref"),
                        }],
                    }),
                    wait: true,
                },
            ],
            warnings: vec![],
        };

        let text = render(&plan);

        assert_eq!(
            text,
            "ssh-add -t 300 - < <vault:key-ref>  \
             # passphrase via askpass (/usr/local/bin/portolano): <vault:pass-ref>\n\
             /usr/bin/ssh -p 22 -- web01.example.com\n\
             #   askpass (/usr/local/bin/portolano): password di admin@web01.example.com <- <vault:pw-ref>"
        );
    }

    #[test]
    fn render_shows_temporary_files_and_new_windows_as_comments() {
        let plan = LaunchPlan {
            steps: vec![
                LaunchStep::WriteTempFile {
                    name: "dc01.rdp".into(),
                    contents: "full address:s:192.0.2.10:3389\n".into(),
                    mode: 0o600,
                },
                LaunchStep::OpenInTerminal {
                    argv: vec![
                        "/usr/local/bin/portolano".into(),
                        "connect".into(),
                        "@abc".into(),
                    ],
                },
            ],
            warnings: vec![],
        };

        let text = render(&plan);

        assert_eq!(
            text,
            "# file temporaneo {temp:dc01.rdp} (permessi 0600):\n\
             #   full address:s:192.0.2.10:3389\n\
             /usr/local/bin/portolano connect @abc  # in una nuova finestra del terminale"
        );
    }
}
