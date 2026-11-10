//! Launch plans: what to run to open a connection (SPEC §7.1, §7.2).
//!
//! A plan holds secrets only as [`SecretRef`]s; [`execute`](crate::execute)
//! reads them from the vault.

// Stub (WP0): the parameters are used once WP4 implements the bodies.
#![expect(unused_variables)]

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

/// Renders a plan for `connect --print`: one quoted command line per step,
/// with secrets shown as `<vault:ref>`.
pub fn render(plan: &LaunchPlan) -> String {
    todo!()
}
