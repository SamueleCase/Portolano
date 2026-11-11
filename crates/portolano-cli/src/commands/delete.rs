//! `portolano delete`: delete a node and its subtree (DEC-018.1).

use anyhow::Result;
use portolano_ops::delete::{DeleteOptions, delete, preview_delete};

use super::common::report_cleanup;
use crate::args::DeleteArgs;
use crate::context::Context;
use crate::failure::CliError;
use crate::prompt;

/// Checks the deletion, asks for confirmation without `-y`, deletes and
/// saves, then removes the secrets of the subtree from the vault.
///
/// # Errors
/// A folder with content without `-r`, references without `--force`, no
/// confirmation, vault and store errors.
pub fn run(ctx: &Context, args: DeleteArgs) -> Result<i32> {
    let (store, mut doc) = ctx.open()?;
    let id = doc.resolve_path(&args.path)?;
    let preview = preview_delete(&doc, id)?;
    let options = DeleteOptions {
        recursive: args.recursive,
        force: args.force,
    };
    preview.check(options)?;
    if !args.yes {
        if !prompt::interactive() {
            return Err(CliError::ConfirmationRequired.into());
        }
        let question = format!(
            "Eliminare {} ({} elementi contenuti, {} segreti)?",
            preview.path, preview.descendants, preview.secrets
        );
        if !prompt::confirm(&question)? {
            return Err(CliError::Aborted.into());
        }
    }
    let vault = ctx.vault();
    let outcome = delete(&store, &mut doc, &vault, id, options)?;
    println!(
        "Eliminato {}: {} nodi, {} segreti cancellati dal vault",
        preview.path, outcome.removed_nodes, outcome.secrets_deleted
    );
    if !outcome.detached.is_empty() {
        println!(
            "{} nodi ora chiedono le credenziali all'avvio",
            outcome.detached.len()
        );
    }
    if preview.references.len() > outcome.detached.len() {
        println!("restano riferimenti al nodo eliminato: `portolano validate` li elenca");
    }
    Ok(report_cleanup(&outcome.cleanup_failures))
}
