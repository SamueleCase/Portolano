//! `portolano edit`: change a node.

use anyhow::Result;
use portolano_ops::nodes::{NodeEdit, edit_node};

use super::common::{details, report_cleanup};
use crate::args::EditArgs;
use crate::context::Context;

/// Applies the changes and saves; secrets no longer used are deleted from
/// the vault.
///
/// # Errors
/// Invalid input, path errors, vault and store errors.
pub fn run(ctx: &Context, args: EditArgs) -> Result<i32> {
    let (store, mut doc) = ctx.open()?;
    let id = doc.resolve_path(&args.path)?;
    let (mut details, secret_properties) = details(args.common)?;
    details.remove_tags = args.untag;
    details.remove_properties = args.unprop;
    let edit = NodeEdit {
        name: args.name,
        details,
        host: args.host,
        port: args.port,
        settings_set: args.set,
        settings_unset: args.unset,
        credential: args.cred.map(|cred| cred.0),
        username: args.user,
        domain: args.domain,
        secret_properties,
    };
    let vault = ctx.vault();
    let outcome = edit_node(&store, &mut doc, &vault, &ctx.protocols, id, edit)?;
    println!("Modificato {}", outcome.path);
    Ok(report_cleanup(&outcome.cleanup_failures))
}
