//! `portolano config` (implemented in a following commit).

use anyhow::Result;

use crate::args::ConfigCommand;
use crate::context::Context;

/// Runs `portolano config`.
pub fn run(_ctx: &Context, _command: ConfigCommand) -> Result<i32> {
    todo!("portolano config")
}
