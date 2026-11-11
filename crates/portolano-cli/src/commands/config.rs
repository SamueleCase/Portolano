//! `portolano config`: show, change and locate the configuration (SPEC §9).

use std::path::Path;

use anyhow::Result;
use portolano_ops::config::set_config_value;
use portolano_ops::locations::config_path;
use serde_json::json;

use crate::args::{ConfigCommand, GlobalArgs};
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
        ConfigCommand::Path => print_path(&ctx.config_path, ctx.json)?,
    }
    Ok(0)
}

/// `config path`: only locates the file, so that it works even when the
/// file is not valid (it runs before the configuration is loaded).
///
/// # Errors
/// No home directory for the default location.
pub fn path(global: &GlobalArgs) -> Result<i32> {
    print_path(&config_path(global.config.as_deref())?, global.json)?;
    Ok(0)
}

fn print_path(path: &Path, json: bool) -> Result<()> {
    if json {
        print_json(&json!({ "config": path }))
    } else {
        println!("{}", path.display());
        Ok(())
    }
}
