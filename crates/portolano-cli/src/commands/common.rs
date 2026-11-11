//! Pieces shared by the commands: reading the common options of `add` and
//! `edit`, passwords, and the report of secrets left in the vault.

use std::fs;
use std::path::Path;

use anyhow::Result;
use portolano_core::Secret;
use portolano_ops::CleanupFailure;
use portolano_ops::nodes::{IconInput, NodeDetails};

use crate::args::{CommonArgs, IconArg, PasswordArgs};
use crate::failure::{CliError, EXIT_VAULT};
use crate::prompt;

/// The common options as details and secret properties (asked on the
/// terminal).
///
/// # Errors
/// Unreadable notes or icon file, a secret property that cannot be asked.
pub fn details(common: CommonArgs) -> Result<(NodeDetails, Vec<(String, Secret)>)> {
    let notes = match common.notes_file {
        Some(path) => Some(fs::read_to_string(&path).map_err(file_error(&path))?),
        None => common.notes,
    };
    let icon = common.icon.map(icon_input).transpose()?;
    let favorite = match (common.favorite, common.no_favorite) {
        (true, _) => Some(true),
        (_, true) => Some(false),
        _ => None,
    };
    let secret_properties = common
        .secret_props
        .into_iter()
        .map(|name| {
            let secret = prompt::new_secret(&format!("Valore della proprietà segreta '{name}'"))?;
            Ok((name, secret))
        })
        .collect::<Result<_>>()?;
    let details = NodeDetails {
        description: common.description,
        notes,
        icon,
        color: common.color,
        favorite,
        add_tags: common.tags,
        set_properties: common.props,
        ..NodeDetails::default()
    };
    Ok((details, secret_properties))
}

/// The password of `--ask-password` or `--password-stdin`, if any.
///
/// # Errors
/// No terminal, empty or mismatching input.
pub fn password(args: &PasswordArgs) -> Result<Option<Secret>> {
    if args.password_stdin {
        return Ok(Some(prompt::stdin_line()?));
    }
    if args.ask_password {
        return Ok(Some(prompt::new_secret("Password")?));
    }
    Ok(None)
}

/// Warns about secrets that stayed in the vault after a successful save and
/// returns the exit code: the vault code if any is left (DEC-022.7).
pub fn report_cleanup(failures: &[CleanupFailure]) -> i32 {
    for failure in failures {
        eprintln!(
            "avviso: il segreto {} non è stato cancellato dal vault: {}",
            failure.reference.as_str(),
            failure.reason
        );
    }
    if failures.is_empty() { 0 } else { EXIT_VAULT }
}

/// Wraps an I/O error on a file named by the user.
pub fn file_error(path: &Path) -> impl FnOnce(std::io::Error) -> CliError + '_ {
    move |source| CliError::File {
        path: path.to_owned(),
        source,
    }
}

fn icon_input(icon: IconArg) -> Result<IconInput> {
    Ok(match icon {
        IconArg::Builtin(name) => IconInput::Builtin(name),
        IconArg::Royal(name) => IconInput::Royal(name),
        IconArg::None => IconInput::Remove,
        IconArg::File(path) => IconInput::Image {
            bytes: fs::read(&path).map_err(file_error(&path))?,
            declared_mime: declared_mime(&path).to_owned(),
        },
    })
}

/// The type an image file claims by its extension; the icon store trusts
/// the bytes, so this only names the file in an error.
fn declared_mime(path: &Path) -> &'static str {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase);
    match extension.as_deref() {
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("tif" | "tiff") => "image/tiff",
        Some("svg") => "image/svg+xml",
        Some("pdf") => "application/pdf",
        _ => "application/octet-stream",
    }
}
