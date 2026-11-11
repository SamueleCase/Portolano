//! `portolano list`: the nodes below a folder as a table.

use anyhow::Result;
use portolano_ops::inspect::{TypeFilter, entries};

use crate::args::ListArgs;
use crate::context::Context;
use crate::output::{print_json, print_table};

/// Prints `TIPO  PERCORSO  RIEPILOGO` for every node below the path.
///
/// # Errors
/// Missing document, path not found or ambiguous.
pub fn run(ctx: &Context, args: ListArgs) -> Result<i32> {
    let (_, doc) = ctx.open()?;
    let start = match &args.path {
        Some(path) => doc.resolve_location(path)?,
        None => None,
    };
    let filter = args.node_type.map(TypeFilter::from);
    let found = entries(&doc, start, &ctx.protocols, None, filter.as_ref())?;
    if ctx.json {
        print_json(&found)?;
        return Ok(0);
    }
    let rows: Vec<Vec<String>> = found
        .iter()
        .map(|entry| {
            vec![
                entry.node_type.label().to_owned(),
                entry.path.clone(),
                entry.summary.clone(),
            ]
        })
        .collect();
    print_table(&["TIPO", "PERCORSO", "RIEPILOGO"], &rows);
    Ok(0)
}
