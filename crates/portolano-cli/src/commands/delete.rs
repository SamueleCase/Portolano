//! `portolano delete` (implemented in a following commit).

use anyhow::Result;

use crate::args::DeleteArgs;
use crate::context::Context;

/// Runs `portolano delete`.
pub fn run(_ctx: &Context, _args: DeleteArgs) -> Result<i32> {
    todo!("portolano delete")
}
