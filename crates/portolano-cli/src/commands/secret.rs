//! `portolano secret`: set, clear and check the secrets of a node.

use anyhow::Result;
use portolano_ops::secrets::{SecretSlot, clear_secret, secret_status, set_secret};

use super::common::report_cleanup;
use crate::args::SecretCommand;
use crate::context::Context;
use crate::failure::CliError;
use crate::output::print_json;
use crate::prompt;

/// Runs a `secret` subcommand.
///
/// # Errors
/// Path errors, a secret the node cannot hold, input, vault and store
/// errors.
pub fn run(ctx: &Context, command: SecretCommand) -> Result<i32> {
    let (store, mut doc) = ctx.open()?;
    let vault = ctx.vault();
    match command {
        SecretCommand::Set { path, field, stdin } => {
            let id = doc.resolve_path(&path)?;
            let slot = field.0;
            let secret = match (&slot, stdin) {
                (SecretSlot::PrivateKey, true) => prompt::stdin_all()?,
                (SecretSlot::PrivateKey, false) => return Err(CliError::KeyNeedsStdin.into()),
                (_, true) => prompt::stdin_line()?,
                (_, false) => prompt::new_secret(&format!("Valore per {} ({path})", label(&slot)))?,
            };
            set_secret(&store, &mut doc, &vault, id, slot.clone(), secret)?;
            println!("Salvato nel vault: {} di {path}", label(&slot));
            Ok(0)
        }
        SecretCommand::Clear { path, field } => {
            let id = doc.resolve_path(&path)?;
            let outcome = clear_secret(&store, &mut doc, &vault, id, field.0.clone())?;
            println!("Eliminato: {} di {path}", label(&field.0));
            Ok(report_cleanup(&outcome.cleanup_failures))
        }
        SecretCommand::Status { path } => {
            let id = doc.resolve_path(&path)?;
            let statuses = secret_status(&doc, &vault, id)?;
            if ctx.json {
                print_json(&statuses)?;
                return Ok(0);
            }
            if statuses.is_empty() {
                println!("{path}: nessun segreto");
            }
            for status in &statuses {
                let state = if status.present {
                    "presente nel vault"
                } else {
                    "assente nel vault"
                };
                println!("{}: {state}", label(&status.slot));
            }
            Ok(0)
        }
    }
}

fn label(slot: &SecretSlot) -> String {
    match slot {
        SecretSlot::Password => "password".to_owned(),
        SecretSlot::Passphrase => "passphrase".to_owned(),
        SecretSlot::PrivateKey => "chiave privata".to_owned(),
        SecretSlot::Property(name) => format!("proprietà {name}"),
    }
}
