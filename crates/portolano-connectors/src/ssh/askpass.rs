//! Askpass protocol between `ssh` and Portolano (SPEC §7.2, "Protocollo
//! askpass", DEC-009). Both sides live here, so the variable names and the
//! message format have a single owner:
//! - the server, started by [`execute`](crate::execute), answers the password
//!   and passphrase prompts over a per-session Unix socket;
//! - the client, [`run_client`], runs inside `portolano` when `ssh` invokes it
//!   as `SSH_ASKPASS`.

// Stub (WP0): the parameters are used once WP4 implements the bodies.
#![expect(unused_variables)]

/// Set to `1` in the environment of `ssh`: `portolano` checks it before
/// parsing arguments and, if set, behaves as the askpass client.
pub const ASKPASS_ENV: &str = "PORTOLANO_ASKPASS";

/// Path of the Unix socket of the askpass server for the session.
pub const ASKPASS_SOCK_ENV: &str = "PORTOLANO_ASKPASS_SOCK";

/// One-time token of the session, 32 random bytes in hex. It authenticates
/// the client to the server and is not a secret of the user.
pub const ASKPASS_TOKEN_ENV: &str = "PORTOLANO_ASKPASS_TOKEN";

/// Askpass client: sends `prompt` (the first argument given by `ssh`) to the
/// server and prints the answer on stdout followed by a newline. When the
/// server answers with a fallback, asks the user on `/dev/tty` instead: with
/// echo for host-key confirmations (`(yes/no`), hidden otherwise.
///
/// Returns the process exit code: 0 when an answer was printed, 1 when there
/// is no TTY for the fallback or the exchange fails.
pub fn run_client(prompt: &str) -> i32 {
    todo!()
}
