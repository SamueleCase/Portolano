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
mod failure;
mod output;
mod prompt;

use clap::Parser;

fn main() {
    let _cli = args::Cli::parse();
}
