//! Launch plans for Windows App: a temporary `.rdp` file without password
//! (SPEC §7.3, "Windows App").

use std::path::Path;

use super::settings::Resolution;
use super::{RdpTarget, account};
use crate::ConnectError;
use crate::plan::{LaunchPlan, LaunchStep, temp_placeholder};
use crate::protocol::check_value;

/// Permissions of the `.rdp` file: it names the user and the host.
const RDP_FILE_MODE: u32 = 0o600;

/// The macOS launcher of applications.
const OPEN: &str = "/usr/bin/open";

/// The plan that writes the `.rdp` file and opens it with Windows App.
/// `fallback` says that `auto` chose Windows App because FreeRDP is missing.
///
/// # Errors
/// [`ConnectError::InvalidValue`] if a value written to the file contains a
/// line break or another control character, which would add lines.
pub(super) fn plan(
    app: &Path,
    target: &RdpTarget,
    fallback: bool,
) -> Result<LaunchPlan, ConnectError> {
    check_file_values(target)?;
    let name = format!("{}.rdp", file_stem(target.name));
    let mut warnings = Vec::new();
    if fallback {
        warnings.push(
            "FreeRDP non trovato: si usa Windows App e la password verrà richiesta \
             (installa FreeRDP con `brew install freerdp`)"
                .to_owned(),
        );
    } else if target.credential.password.is_some() {
        warnings
            .push("Windows App non riceve la password salvata: la chiederà all'avvio".to_owned());
    }
    Ok(LaunchPlan {
        steps: vec![
            LaunchStep::WriteTempFile {
                name: name.clone(),
                contents: rdp_file(target),
                mode: RDP_FILE_MODE,
            },
            LaunchStep::Run {
                program: Path::new(OPEN).to_path_buf(),
                args: vec![
                    "-a".to_owned(),
                    app.to_string_lossy().into_owned(),
                    temp_placeholder(&name),
                ],
                env: Vec::new(),
                stdin_secret: None,
                askpass: None,
                // `open` returns as soon as the app has the file.
                wait: true,
            },
        ],
        warnings,
    })
}

/// Rejects control characters (such as `\r` and `\n`) in every value of the
/// document that goes into the `.rdp` file.
fn check_file_values(target: &RdpTarget) -> Result<(), ConnectError> {
    let id = target.node_id;
    check_value(id, "host", target.host, &[])?;
    if let Some(user) = &target.credential.username {
        check_value(id, "username", user, &[])?;
    }
    if let Some(domain) = &target.credential.domain {
        check_value(id, "domain", domain, &[])?;
    }
    if let Some(gateway) = &target.gateway {
        check_value(id, "gateway.host", gateway.host, &[])?;
    }
    Ok(())
}

/// The content of the `.rdp` file; never a password.
fn rdp_file(target: &RdpTarget) -> String {
    let settings = target.settings;
    let mut lines = vec![format!("full address:s:{}:{}", target.host, target.port)];
    if let Some(account) = account(&target.credential) {
        lines.push(format!("username:s:{account}"));
    }
    if let Some(gateway) = &target.gateway {
        lines.push(format!("gatewayhostname:s:{}", gateway.address()));
        lines.push("gatewayusagemethod:i:1".to_owned());
    }
    let screen_mode = if settings.resolution == Resolution::Fullscreen {
        2
    } else {
        1
    };
    lines.push(format!("screen mode id:i:{screen_mode}"));
    if let Resolution::Fixed { width, height } = settings.resolution {
        lines.push(format!("desktopwidth:i:{width}"));
        lines.push(format!("desktopheight:i:{height}"));
    }
    if let Some(depth) = settings.color_depth {
        lines.push(format!("session bpp:i:{depth}"));
    }
    lines.push(format!(
        "redirectclipboard:i:{}",
        u8::from(settings.clipboard)
    ));
    lines.push(format!("use multimon:i:{}", u8::from(settings.multimon)));
    lines.push(format!(
        "administrative session:i:{}",
        u8::from(settings.admin)
    ));
    let mut contents = lines.join("\n");
    contents.push('\n');
    contents
}

/// A file name from a node name: only letters, digits, `-`, `_` and spaces
/// survive, so the name cannot leave the temporary directory.
fn file_stem(name: &str) -> String {
    let stem: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || matches!(c, '-' | '_' | ' ') {
                c
            } else {
                '_'
            }
        })
        .collect();
    let stem = stem.trim();
    if stem.is_empty() {
        "connessione".to_owned()
    } else {
        stem.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn file_stem_replaces_path_separators_and_dots() {
        assert_eq!(
            file_stem("../ACME/dc01.example.com"),
            "___ACME_dc01_example_com"
        );
    }

    #[test]
    fn empty_file_stem_gets_a_default() {
        assert_eq!(file_stem("///"), "___");
        assert_eq!(file_stem("  "), "connessione");
    }
}
