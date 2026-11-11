//! Launch plans for the FreeRDP client: arguments, password on stdin,
//! gateway (SPEC §7.3, "FreeRDP", DEC-020).
//!
//! Option names checked with `sdl-freerdp /help` of FreeRDP 3.33.1: the
//! gateway option is `/gateway:` (the SPEC's `/gw:` does not exist) and there
//! is no explicit option to reuse the credentials on the gateway; leaving
//! `u:`/`d:` out of `/gateway:` lets FreeRDP use the destination credentials.

use std::path::PathBuf;

use portolano_core::{CredentialOrigin, ResolvedCredential};

use super::settings::{CertPolicy, RdpSettings, Resolution};
use super::{GatewayLogin, RdpTarget};
use crate::ConnectError;
use crate::plan::{LaunchPlan, LaunchStep};
use crate::protocol::{check_value, host_port};

/// Reads the credentials from stdin before connecting. Without `:force`
/// FreeRDP reads them only when the server asks, which may never happen and
/// makes the outcome depend on the server.
const FROM_STDIN: &str = "/from-stdin:force";

/// Log level of a detached FreeRDP, whose stderr goes to a log file
/// (option checked with `sdl-freerdp /help` of FreeRDP 3.33.1).
const DETACHED_LOG_LEVEL: &str = "/log-level:WARN";

/// Where the messages of a detached FreeRDP go. The plan is pure, so the
/// exact path is given by `PreparedLaunch::log_path` when executing.
const DETACHED_LOG_WARNING: &str = "i messaggi d'errore di FreeRDP vengono salvati in un log nella cartella temporanea di Portolano";

/// Separators of the `/gateway:` sub-options: a value containing one would
/// add or change a sub-option.
const GATEWAY_FORBIDDEN: [char; 1] = [','];

/// The plan that runs FreeRDP; the password, if any, goes to its stdin.
///
/// FreeRDP runs detached unless `wait`, or unless it will have to ask the
/// user for a password (DEC-025): the destination has none, or the gateway
/// uses another credential. Detached, it would have no terminal to ask on,
/// so it then runs in the foreground with a warning that says why.
///
/// # Errors
/// [`ConnectError::InvalidValue`] for a gateway value that would break the
/// `/gateway:` option.
pub(super) fn plan(
    program: PathBuf,
    target: &RdpTarget,
    cert: CertPolicy,
    wait: bool,
) -> Result<LaunchPlan, ConnectError> {
    let mut warnings = Vec::new();
    let mut args = vec![format!("/v:{}", host_port(target.host, target.port))];
    if let Some(user) = &target.credential.username {
        args.push(format!("/u:{user}"));
    }
    if let Some(domain) = &target.credential.domain {
        args.push(format!("/d:{domain}"));
    }
    args.push(format!("/cert:{}", cert.as_freerdp_value()));
    args.extend(display_args(target.settings));
    let mut gateway_prompts = false;
    if let Some(gateway) = &target.gateway {
        let (arg, prompts) = gateway_arg(target, gateway, &mut warnings)?;
        args.push(arg);
        gateway_prompts = prompts;
    }
    // The password never goes to argv (`/p:`): FreeRDP reads it from stdin,
    // and only with a user, as for SSH, so it cannot be sent for another one.
    let stdin_secret = match (&target.credential.password, &target.credential.username) {
        (Some(password), Some(_)) => Some(password.clone()),
        (Some(_), None) => {
            warnings.push(
                "la credenziale ha una password ma nessun utente: \
                 la password salvata non viene usata e FreeRDP la chiederà"
                    .to_owned(),
            );
            None
        }
        (None, _) => None,
    };
    if stdin_secret.is_some() {
        args.push(FROM_STDIN.to_owned());
    }
    let asks = match (stdin_secret.is_none(), gateway_prompts) {
        (true, true) => Some("la password della destinazione e quella del gateway"),
        (true, false) => Some("la password della destinazione"),
        (false, true) => Some("la password del gateway"),
        (false, false) => None,
    };
    if let Some(what) = asks.filter(|_| !wait) {
        warnings.push(format!(
            "FreeRDP chiederà {what}: viene eseguito in primo piano in questo terminale \
             invece di staccarsi"
        ));
    }
    let wait = wait || asks.is_some();
    if !wait {
        // Its stderr goes to a log (see `PreparedLaunch::log_path`): only
        // warnings and errors, never debug dumps of the settings, even with
        // `WLOG_LEVEL=DEBUG` in the environment.
        args.push(DETACHED_LOG_LEVEL.to_owned());
        warnings.push(DETACHED_LOG_WARNING.to_owned());
    }

    Ok(LaunchPlan {
        steps: vec![LaunchStep::Run {
            program,
            args,
            env: Vec::new(),
            stdin_secret,
            askpass: None,
            wait,
        }],
        warnings,
    })
}

/// Size, scale, color, monitors, session and clipboard options.
fn display_args(settings: &RdpSettings) -> Vec<String> {
    let mut args = vec![match settings.resolution {
        Resolution::Fit => "/dynamic-resolution".to_owned(),
        Resolution::Fullscreen => "/f".to_owned(),
        Resolution::Fixed { width, height } => format!("/size:{width}x{height}"),
    }];
    if let Some(scale) = settings.scale {
        args.push(format!("/scale:{scale}"));
    }
    if let Some(depth) = settings.color_depth {
        args.push(format!("/bpp:{depth}"));
    }
    if settings.multimon {
        args.push("/multimon".to_owned());
    }
    if settings.admin {
        args.push("/admin".to_owned());
    }
    let clipboard = if settings.clipboard {
        "+clipboard"
    } else {
        "-clipboard"
    };
    args.push(clipboard.to_owned());
    args
}

/// `/gateway:g:<host>[:<port>][,u:<user>][,d:<domain>]`, never with `p:`,
/// and whether FreeRDP will ask for the gateway login: it does unless the
/// gateway uses the credential of the destination.
fn gateway_arg(
    target: &RdpTarget,
    gateway: &GatewayLogin,
    warnings: &mut Vec<String>,
) -> Result<(String, bool), ConnectError> {
    let id = target.node_id;
    check_value(id, "gateway.host", gateway.host, &GATEWAY_FORBIDDEN)?;
    let mut arg = format!("/gateway:g:{}", gateway.address());
    let login = &gateway.credential;
    if login.origin == CredentialOrigin::Prompt {
        warnings.push(format!(
            "credenziale del gateway {} da chiedere all'avvio: la chiederà FreeRDP",
            gateway.host
        ));
        return Ok((arg, true));
    }
    if same_login(login, &target.credential) {
        return Ok((arg, false));
    }
    if let Some(user) = &login.username {
        check_value(id, "gateway.username", user, &GATEWAY_FORBIDDEN)?;
        arg.push_str(&format!(",u:{user}"));
    }
    if let Some(domain) = &login.domain {
        check_value(id, "gateway.domain", domain, &GATEWAY_FORBIDDEN)?;
        arg.push_str(&format!(",d:{domain}"));
    }
    // Without a user or a domain FreeRDP logs in to the gateway with the
    // credentials of the destination (DEC-020), so it asks nothing more.
    let other_login = login.username.is_some() || login.domain.is_some();
    if other_login {
        warnings.push(format!(
            "il gateway {} usa una credenziale diversa da quella di destinazione: \
             la sua password la chiederà FreeRDP",
            gateway.host
        ));
    }
    Ok((arg, other_login))
}

fn same_login(gateway: &ResolvedCredential, target: &ResolvedCredential) -> bool {
    gateway.username == target.username
        && gateway.domain == target.domain
        && gateway.password == target.password
}
