//! `portolano init`: create an empty document.

use anyhow::Result;
use portolano_ops::document::init_document;

use crate::args::InitArgs;
use crate::context::Context;
use crate::output::visible;

/// Creates the document at the effective location.
///
/// # Errors
/// An existing file without `--force`, I/O errors.
pub fn run(ctx: &Context, args: InitArgs) -> Result<i32> {
    let path = &ctx.locations.document;
    let (_, doc) = init_document(path, &args.name, args.force)?;
    println!(
        "Documento '{}' creato in {}",
        visible(&doc.name),
        visible(&path.display().to_string())
    );
    Ok(0)
}
