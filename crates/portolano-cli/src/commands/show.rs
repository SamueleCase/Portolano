//! `portolano show`: the details of a node.

use anyhow::Result;
use portolano_ops::OpsError;
use portolano_ops::inspect::resolved_credential;
use serde_json::json;

use crate::args::ShowArgs;
use crate::context::Context;
use crate::output::{print_json, print_show};

/// Prints the node; with `--resolved` also its effective credential, with
/// the presence of the secrets in the vault and never their values.
///
/// # Errors
/// Missing document, path not found or ambiguous, vault errors.
pub fn run(ctx: &Context, args: ShowArgs) -> Result<i32> {
    let (_, doc) = ctx.open()?;
    let id = doc.resolve_path(&args.path)?;
    let node = doc
        .find(id)
        .ok_or(portolano_core::Error::NodeNotFound { id })?;
    let resolved = if args.resolved {
        let vault = ctx.vault();
        match resolved_credential(&doc, id, &vault) {
            Ok(view) => Some(Ok(view)),
            // A credential that does not resolve is part of what `show`
            // reports; only the vault failing is an error of the command.
            Err(OpsError::Core(error)) => Some(Err(error.to_string())),
            Err(error) => return Err(error.into()),
        }
    } else {
        None
    };
    if ctx.json {
        let resolved = match &resolved {
            None => serde_json::Value::Null,
            Some(Ok(view)) => serde_json::to_value(view)?,
            Some(Err(reason)) => json!({ "error": reason }),
        };
        print_json(&json!({
            "path": doc.path_of(id),
            "node": node,
            "resolved": resolved,
        }))?;
        return Ok(0);
    }
    print_show(&doc, node, &ctx.protocols, &resolved);
    Ok(0)
}
