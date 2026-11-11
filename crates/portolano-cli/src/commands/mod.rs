//! One module per subcommand (SPEC §10.2): each parses its arguments, calls
//! `portolano-ops` and prints the result.

mod add;
pub mod askpass;
mod common;
mod config;
mod connect;
mod delete;
mod edit;
mod import;
mod init;
mod list;
mod mv;
mod secret;
mod show;
mod tree;
mod validate;

use anyhow::Result;

pub use config::path as config_path;

use crate::args::Command;
use crate::context::Context;

/// Runs `command` and returns the exit code.
///
/// # Errors
/// The error of the command.
pub fn run(ctx: &Context, command: Command) -> Result<i32> {
    match command {
        Command::Init(args) => init::run(ctx, args),
        Command::Tree(args) => tree::run(ctx, args),
        Command::List(args) => list::run(ctx, args),
        Command::Show(args) => show::run(ctx, args),
        Command::Add(command) => add::run(ctx, command),
        Command::Edit(args) => edit::run(ctx, *args),
        Command::Move(args) => mv::run(ctx, args),
        Command::Delete(args) => delete::run(ctx, args),
        Command::Secret(command) => secret::run(ctx, command),
        Command::Connect(args) => connect::run(ctx, args),
        Command::Import(args) => import::run(ctx, args),
        Command::Validate => validate::run(ctx),
        Command::Config(command) => config::run(ctx, command),
        Command::Askpass { prompt } => Ok(askpass::run(prompt.as_deref().unwrap_or_default())),
    }
}
