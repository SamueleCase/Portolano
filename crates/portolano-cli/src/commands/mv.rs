//! `portolano move`: move a node, or reorder it in its folder.

use anyhow::Result;
use portolano_ops::moves::{Placement, move_node};

use crate::args::MoveArgs;
use crate::context::Context;
use crate::failure::with_paths;
use crate::output::visible;

/// Moves the node and saves.
///
/// # Errors
/// Path errors, a move into the node itself, a name already used in the
/// destination, store errors.
pub fn run(ctx: &Context, args: MoveArgs) -> Result<i32> {
    let (store, mut doc) = ctx.open()?;
    let id = doc.resolve_path(&args.path)?;
    let destination = doc.resolve_location(&args.destination)?;
    let placement = match (args.position, &args.before, &args.after) {
        (Some(index), _, _) => Placement::Index(index),
        (_, Some(before), _) => Placement::Before(doc.resolve_path(before)?),
        (_, _, Some(after)) => Placement::After(doc.resolve_path(after)?),
        _ => Placement::End,
    };
    let path = move_node(&store, &mut doc, id, destination, placement)
        .map_err(|error| with_paths(error, &doc))?;
    println!("Spostato in {}", visible(&path));
    Ok(0)
}
