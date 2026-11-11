//! `portolano tree`: the tree of the document or of a folder.

use anyhow::Result;
use portolano_ops::inspect::{NodeType, entries, summary};

use crate::args::TreeArgs;
use crate::context::Context;
use crate::failure::CliError;
use crate::output::{Header, print_json, render_tree};

/// Prints the tree; with `--json` the model of the subtree.
///
/// # Errors
/// Missing document, path not found or ambiguous.
pub fn run(ctx: &Context, args: TreeArgs) -> Result<i32> {
    if ctx.json && args.depth.is_some() {
        // The JSON form is the model of the subtree, which has no depth.
        return Err(CliError::Incompatible {
            first: "--depth",
            second: "--json",
        }
        .into());
    }
    let (_, doc) = ctx.open()?;
    let start = match &args.path {
        Some(path) => doc.resolve_location(path)?,
        None => None,
    };
    let node = start.and_then(|id| doc.find(id));
    if ctx.json {
        match node {
            Some(node) => print_json(node)?,
            None => print_json(&doc)?,
        }
        return Ok(0);
    }
    let found = entries(&doc, start, &ctx.protocols, args.depth, None)?;
    let header = match node {
        Some(node) => Header::node(
            &NodeType::of(node),
            &node.name,
            node.id,
            summary(&doc, node, &ctx.protocols),
            args.ids,
        ),
        None => Header {
            label: doc.name.clone(),
            summary: String::new(),
        },
    };
    println!("{}", render_tree(header, &found, args.ids));
    Ok(0)
}
