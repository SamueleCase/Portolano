//! `portolano secret` (implemented in a following commit).

use anyhow::Result;

use crate::args::SecretCommand;
use crate::context::Context;

/// Runs `portolano secret`.
pub fn run(_ctx: &Context, _command: SecretCommand) -> Result<i32> {
    todo!("portolano secret")
}
