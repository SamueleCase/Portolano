//! `portolano`: command-line interface of Portolano.
//!
//! A thin layer over `portolano-ops`: argument parsing, terminal I/O and
//! output formatting only (SPEC §2, §10, DEC-018.2).

#![allow(
    dead_code,
    unused_imports,
    reason = "the commands are implemented in the following commits"
)]

mod args;
mod commands;
mod context;
mod failure;
mod output;
mod prompt;

use std::process::exit;

use anyhow::Result;
use clap::Parser;
use tracing::Level;

use crate::args::Cli;
use crate::context::Context;
use crate::failure::CliError;

fn main() {
    let cli = Cli::parse();
    init_logging(cli.global.verbose);
    let code = run(cli).unwrap_or_else(|error| failure::report(&error));
    exit(code);
}

fn run(cli: Cli) -> Result<i32> {
    if cli.global.json && !cli.command.is_read_only() {
        return Err(CliError::JsonNotSupported.into());
    }
    let ctx = Context::new(&cli.global)?;
    commands::run(&ctx, cli.command)
}

/// Logs go to stderr; each `-v` shows one more level.
fn init_logging(verbosity: u8) {
    let level = match verbosity {
        0 => Level::WARN,
        1 => Level::INFO,
        2 => Level::DEBUG,
        _ => Level::TRACE,
    };
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_max_level(level)
        .with_target(false)
        .init();
}
