//! Askpass protocol between `ssh` and Portolano (SPEC §7.2, "Protocollo
//! askpass", DEC-009). Both sides live here, so the variable names and the
//! message format have a single owner:
//! - the server, started by [`execute`](crate::execute), answers the password
//!   and passphrase prompts over a per-session Unix socket;
//! - the client, [`run_client`], runs inside `portolano` when `ssh` invokes it
//!   as `SSH_ASKPASS`.
//!
//! Messages are one JSON object per line: the client sends
//! `{"token": …, "prompt": …}`, the server answers `{"answer": …}`,
//! `{"fallback": true}` or `{"error": …}`.

// The client, the server and the recognition of the prompts need Unix
// sockets; elsewhere only the constants and the placeholder server remain.
#[cfg(unix)]
mod client;
#[cfg(unix)]
mod prompt;
#[cfg(unix)]
mod server;

use portolano_core::Secret;
#[cfg(unix)]
use serde::{Deserialize, Serialize};

/// The key path that `ssh-add` shows for a key read from stdin; the askpass
/// entry of a [`SshAddKey`](crate::LaunchStep::SshAddKey) step uses it
/// (DEC-015).
pub(crate) const SSH_ADD_STDIN_KEY_PATH: &str = "(stdin)";
#[cfg(unix)]
pub(crate) use server::AskpassServer;
#[cfg(not(unix))]
pub(crate) use unsupported::AskpassServer;

use crate::plan::PromptMatcher;

/// Set to [`ASKPASS_ENABLED`] in the environment of `ssh`: `portolano`
/// checks it before parsing arguments and, if set, behaves as the askpass
/// client.
pub const ASKPASS_ENV: &str = "PORTOLANO_ASKPASS";

/// The value of [`ASKPASS_ENV`] that turns `portolano` into the askpass
/// client.
pub const ASKPASS_ENABLED: &str = "1";

/// Path of the Unix socket of the askpass server for the session.
pub const ASKPASS_SOCK_ENV: &str = "PORTOLANO_ASKPASS_SOCK";

/// One-time token of the session, 32 random bytes in hex. It authenticates
/// the client to the server and is not a secret of the user.
pub const ASKPASS_TOKEN_ENV: &str = "PORTOLANO_ASKPASS_TOKEN";

/// Upper bound of a message on the socket; longer lines are rejected.
#[cfg(unix)]
const MAX_MESSAGE_BYTES: usize = 64 * 1024;

/// Whether this process was started by `ssh` as its askpass program, that is
/// [`ASKPASS_ENV`] equals [`ASKPASS_ENABLED`]. `main` checks it before
/// parsing arguments and then calls [`run_client`].
pub fn is_client_invocation() -> bool {
    std::env::var_os(ASKPASS_ENV).is_some_and(|value| value == ASKPASS_ENABLED)
}

/// Askpass client: sends `prompt` (the first argument given by `ssh`) to the
/// server and prints the answer on stdout followed by a newline. When the
/// server answers with a fallback, asks the user on `/dev/tty` instead: with
/// echo for host-key confirmations (`(yes/no`), hidden otherwise.
///
/// Returns the process exit code: 0 when an answer was printed, 1 when there
/// is no TTY for the fallback or the exchange fails.
pub fn run_client(prompt: &str) -> i32 {
    #[cfg(unix)]
    match client::run(prompt) {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("portolano askpass: {error}");
            1
        }
    }
    #[cfg(not(unix))]
    {
        let _ = prompt;
        eprintln!("portolano askpass: non supportato su questa piattaforma");
        1
    }
}

/// A secret the server may answer with, already read from the vault.
// Without Unix sockets no server reads the fields; the plan code that builds
// them stays the same on every system.
#[cfg_attr(not(unix), allow(dead_code))]
pub(crate) struct ServedSecret {
    /// The prompt it answers.
    pub(crate) matcher: PromptMatcher,
    /// The answer.
    pub(crate) secret: Secret,
}

/// A request of the client.
#[cfg(unix)]
#[derive(Debug, Serialize, Deserialize)]
struct Request<S> {
    token: S,
    prompt: S,
}

/// The answer of the server. Serialized externally tagged:
/// `{"answer": "…"}`, `{"fallback": true}`, `{"error": "…"}`.
#[cfg(unix)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Reply<S> {
    Answer(S),
    Fallback(bool),
    Error(S),
}

/// Without Unix sockets there is no askpass server: plans that need one
/// fail with [`ConnectError::UnsupportedPlatform`](crate::ConnectError).
#[cfg(not(unix))]
mod unsupported {
    use std::path::Path;

    use super::ServedSecret;
    use crate::ConnectError;

    /// Placeholder of the askpass server on systems without Unix sockets.
    pub(crate) struct AskpassServer;

    impl AskpassServer {
        /// Always fails: askpass needs a Unix socket.
        pub(crate) fn start(_secrets: Vec<ServedSecret>) -> Result<AskpassServer, ConnectError> {
            Err(ConnectError::UnsupportedPlatform {
                feature: "l'askpass di ssh".to_owned(),
            })
        }

        /// Never called: [`AskpassServer::start`] always fails.
        pub(crate) fn child_env(&self, _program: &Path) -> Vec<(String, String)> {
            Vec::new()
        }
    }
}

#[cfg(test)]
#[cfg(unix)]
mod tests;
