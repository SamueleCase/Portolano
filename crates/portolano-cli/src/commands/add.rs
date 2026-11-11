//! `portolano add` (implemented in a following commit).

use anyhow::Result;

use crate::args::AddCommand;
use crate::context::Context;

/// Runs `portolano add`.
pub fn run(_ctx: &Context, _command: AddCommand) -> Result<i32> {
    todo!("portolano add")
}
