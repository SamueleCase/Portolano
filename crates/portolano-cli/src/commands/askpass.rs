//! The askpass client (SPEC §7.2, DEC-009): `ssh` runs `portolano` with
//! `PORTOLANO_ASKPASS=1` and the prompt as first argument. The protocol
//! lives in `portolano_connectors::ssh::askpass`; this module only hands
//! over to it.

use portolano_connectors::ssh::askpass::{is_client_invocation, run_client};

/// If this process was started as askpass client, answers the prompt and
/// returns the exit code; checked before parsing the arguments, which are
/// not Portolano's.
pub fn run_if_invoked() -> Option<i32> {
    if !is_client_invocation() {
        return None;
    }
    let prompt = std::env::args_os()
        .nth(1)
        .map(|prompt| prompt.to_string_lossy().into_owned())
        .unwrap_or_default();
    Some(run_client(&prompt))
}

/// The hidden `askpass` command: the same client, with the prompt as
/// argument.
pub fn run(prompt: &str) -> i32 {
    run_client(prompt)
}
