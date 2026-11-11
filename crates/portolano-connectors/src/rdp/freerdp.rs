//! Launch plans for the FreeRDP client: arguments, password on stdin,
//! gateway (SPEC §7.3, "FreeRDP").
//!
//! Option names checked with `sdl-freerdp /help` of FreeRDP 3.33.1: the
//! gateway option is `/gateway:` (the SPEC's `/gw:` does not exist) and there
//! is no explicit option to reuse the credentials on the gateway; leaving
//! `u:`/`d:` out of `/gateway:` lets FreeRDP use the destination credentials.

use std::path::PathBuf;

use portolano_core::ResolvedCredential;

use super::settings::{CertPolicy, Resolution};
use super::{GatewayLogin, RdpTarget};
use crate::plan::{LaunchPlan, LaunchStep};
use crate::protocol::host_port;

/// Reads the credentials from stdin before connecting. Without `:force`
/// FreeRDP reads them only when the server asks, which may never happen and
/// makes the outcome depend on the server.
const FROM_STDIN: &str = "/from-stdin:force";

/// The plan that runs FreeRDP; the password, if any, goes to its stdin.
pub(super) fn plan(
    program: PathBuf,
    target: &RdpTarget,
    cert: CertPolicy,
    wait: bool,
) -> LaunchPlan {
    let settings = target.settings;
    let mut warnings = Vec::new();
    let mut args = vec![format!("/v:{}", host_port(target.host, target.port))];
    if let Some(user) = &target.credential.username {
        args.push(format!("/u:{user}"));
    }
    if let Some(domain) = &target.credential.domain {
        args.push(format!("/d:{domain}"));
    }
    args.push(format!("/cert:{}", cert.as_freerdp_value()));
    args.push(match settings.resolution {
        Resolution::Fit => "/dynamic-resolution".to_owned(),
        Resolution::Fullscreen => "/f".to_owned(),
        Resolution::Fixed { width, height } => format!("/size:{width}x{height}"),
    });
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
    args.push(
        if settings.clipboard {
            "+clipboard"
        } else {
            "-clipboard"
        }
        .to_owned(),
    );
    if let Some(gateway) = &target.gateway {
        args.push(gateway_arg(gateway, &target.credential, &mut warnings));
    }
    // The password never goes to argv (`/p:`): FreeRDP reads it from stdin.
    let stdin_secret = target.credential.password.clone();
    if stdin_secret.is_some() {
        args.push(FROM_STDIN.to_owned());
    }

    LaunchPlan {
        steps: vec![LaunchStep::Run {
            program,
            args,
            env: Vec::new(),
            stdin_secret,
            askpass: None,
            wait,
        }],
        warnings,
    }
}

/// `/gateway:g:<host>[:<port>][,u:<user>][,d:<domain>]`, never with `p:`.
fn gateway_arg(
    gateway: &GatewayLogin,
    target: &ResolvedCredential,
    warnings: &mut Vec<String>,
) -> String {
    let mut arg = format!("/gateway:g:{}", gateway.address());
    let login = &gateway.credential;
    let same_login = login.username == target.username
        && login.domain == target.domain
        && login.password == target.password;
    if same_login {
        return arg;
    }
    if let Some(user) = &login.username {
        arg.push_str(&format!(",u:{user}"));
    }
    if let Some(domain) = &login.domain {
        arg.push_str(&format!(",d:{domain}"));
    }
    if login.password.is_some() {
        warnings.push(format!(
            "la password del gateway {} è diversa da quella di destinazione: la chiederà FreeRDP",
            gateway.host
        ));
    }
    arg
}
