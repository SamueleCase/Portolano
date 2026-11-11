//! `portolano import`: import a document of another program (SPEC §8.4).

use std::fs;

use anyhow::Result;
use portolano_import::{NonInteractive, SecretPrompter};
use portolano_ops::import::{ConflictPolicy, ImportRequest, import};

use super::common::{file_error, report_cleanup};
use crate::args::{ConflictArg, ImportArgs};
use crate::context::Context;
use crate::failure::CliError;
use crate::output::print_import;
use crate::prompt::{self, ImportPrompter};

/// Imports the file, prints the report and, with `--report`, saves it as
/// JSON.
///
/// # Errors
/// Unreadable file, unknown format, conflicts, destination errors, vault and
/// store errors.
pub fn run(ctx: &Context, args: ImportArgs) -> Result<i32> {
    let (store, mut doc) = ctx.open()?;
    let input = fs::read(&args.source).map_err(file_error(&args.source))?;
    let into = match &args.into {
        Some(path) => doc.resolve_location(path)?,
        None => None,
    };
    let request = ImportRequest {
        input: &input,
        file_name: args
            .source
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default(),
        format: args.format,
        into,
        on_conflict: match args.on_conflict {
            ConflictArg::Fail => ConflictPolicy::Fail,
            ConflictArg::Update => ConflictPolicy::Update,
            ConflictArg::NewIds => ConflictPolicy::NewIds,
        },
        include_trash: args.include_trash,
        dry_run: args.dry_run,
    };
    let vault = ctx.vault();
    let mut prompter: Box<dyn SecretPrompter> = if args.prompt_secrets {
        if !prompt::interactive() {
            return Err(CliError::NoTerminal {
                what: "le password dell'import".to_owned(),
            }
            .into());
        }
        Box::new(ImportPrompter)
    } else {
        Box::new(NonInteractive)
    };
    let result = import(
        &store,
        &mut doc,
        &vault,
        &ctx.importers,
        request,
        prompter.as_mut(),
    )?;
    if let Some(path) = &args.report {
        let json = serde_json::to_string_pretty(&result.report)? + "\n";
        fs::write(path, json).map_err(file_error(path))?;
    }
    let root_path = doc
        .path_of(result.root)
        .or_else(|| into.and_then(|id| doc.path_of(id)))
        .unwrap_or_else(|| "/".to_owned());
    print_import(&result, &root_path);
    Ok(report_cleanup(&result.cleanup_failures))
}
