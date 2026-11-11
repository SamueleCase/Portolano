//! `portolano config`: show, change and locate the configuration (SPEC §9).

use anyhow::Result;
use portolano_ops::config::set_config_value;
use serde_json::json;

use crate::args::ConfigCommand;
use crate::context::Context;
use crate::output::print_json;

/// Runs a `config` subcommand.
///
/// # Errors
/// An unknown key or an invalid value (exit 4), I/O errors.
pub fn run(ctx: &Context, command: ConfigCommand) -> Result<i32> {
    match command {
        ConfigCommand::Show => {
            if ctx.json {
                print_json(&ctx.config)?;
            } else {
                print!("{}", toml::to_string_pretty(&ctx.config)?);
            }
        }
        ConfigCommand::Set { key, value } => {
            let config = set_config_value(&ctx.config, &key, &value, &ctx.protocols)?;
            config.save(&ctx.config_path)?;
            println!("{key} impostato in {}", ctx.config_path.display());
        }
        ConfigCommand::Path => {
            if ctx.json {
                print_json(&json!({ "config": ctx.config_path }))?;
            } else {
                println!("{}", ctx.config_path.display());
            }
        }
    }
    Ok(0)
}
