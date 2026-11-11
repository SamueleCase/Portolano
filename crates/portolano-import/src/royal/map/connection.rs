//! SSH and RDP connections, with Secure Gateways as jump hosts and Remote
//! Desktop Gateways (SPEC §8.3.5).
//!
//! The settings are written with the names of `SshSettings` and
//! `RdpSettings` (SPEC §7.2–7.3): the importer depends on the core only.
//! The fields that link a connection to a gateway were never observed in a
//! real document (`docs/royal-format.md` §4.2): their names are hypotheses,
//! so a link built from them keeps the original value in `extra` and makes
//! the node partial.

use portolano_core::Connection;
use serde_json::{Map, Value, json};
use uuid::Uuid;

use super::context::{ObjectContext, SecretCollector};
use super::credential;
use super::references::References;
use crate::ImportError;

/// [ipo] The Secure Gateway a connection goes through.
const SECURE_GATEWAY_ID: &str = "SecureGatewayID";
/// [ipo] The reusable Remote Desktop Gateway of an RDP connection.
const RDS_GATEWAY_ID: &str = "RDSGatewayID";
/// [ipo] Remote Desktop Gateway written in the RDP connection itself.
const GATEWAY_HOST: &str = "GatewayHostName";
/// [ipo] When the gateway is used, with the values of the `.rdp` files:
/// 1 always, 2 when the host is not reachable directly, 0 and 4 never.
const GATEWAY_USAGE: &str = "GatewayUsageMethod";

const UNVERIFIED_JUMP_HOST: &str = "jump host ricavato dal Secure Gateway con campi Royal non verificati: controlla il collegamento";
const UNVERIFIED_GATEWAY: &str =
    "gateway RDP ricavato da campi Royal non verificati: controlla host, porta e credenziale";

/// Royal flags that map to boolean `SshSettings`.
const SSH_FLAGS: [(&str, &str); 4] = [
    ("SSHAllowAgentForwarding", "agent_forwarding"),
    ("SSHEnableX11Forwarding", "x11"),
    ("SSHEnableTrustedX11Forwarding", "x11_trusted"),
    ("SSHEnableCompression", "compression"),
];

/// A `RoyalSSHConnection` or a `RoyalSecureGateway`. Telnet and serial
/// connections are unsupported and never get here.
///
/// # Errors
/// The error of the prompter.
pub(super) fn ssh(
    ctx: &mut ObjectContext<'_>,
    references: &References,
    secrets: &mut SecretCollector<'_>,
) -> Result<Connection, ImportError> {
    ctx.fields.take_raw("IsTelnetConnection");
    ctx.fields.take_raw("IsSerialPortConnection");
    let (host, port) = address(ctx, "Port");
    let credential = credential::spec(ctx, references, secrets)?;
    let mut settings = Map::new();
    for (field, setting) in SSH_FLAGS {
        if ctx.fields.take_flag(field) {
            settings.insert(setting.to_owned(), Value::Bool(true));
        }
    }
    if let Some(seconds) = ctx
        .fields
        .take_parsed::<u32>("KeepAliveInterval")
        .filter(|seconds| *seconds > 0)
    {
        settings.insert("keepalive_s".to_owned(), seconds.into());
    }
    if let Some(gateway) = secure_gateway(ctx, references) {
        settings.insert("jump_hosts".to_owned(), json!([{ "$ref": gateway }]));
    }
    Ok(Connection {
        protocol: "ssh".to_owned(),
        host,
        port,
        credential,
        settings,
    })
}

/// A `RoyalRDSConnection`.
///
/// # Errors
/// The error of the prompter.
pub(super) fn rdp(
    ctx: &mut ObjectContext<'_>,
    references: &References,
    secrets: &mut SecretCollector<'_>,
) -> Result<Connection, ImportError> {
    let (host, port) = address(ctx, "RDPPort");
    let credential = credential::spec(ctx, references, secrets)?;
    let mut settings = Map::new();
    // A size of 0 means "fit the window", like an absent one.
    let width = ctx.fields.take_parsed::<u32>("DesktopWidth").unwrap_or(0);
    let height = ctx.fields.take_parsed::<u32>("DesktopHeight").unwrap_or(0);
    if width > 0 && height > 0 {
        let resolution = json!({ "mode": "fixed", "width": width, "height": height });
        settings.insert("resolution".to_owned(), resolution);
    }
    if let Some(depth) = ctx
        .fields
        .take_parsed::<u8>("ColorDepth")
        .filter(|d| *d > 0)
    {
        settings.insert("color_depth".to_owned(), depth.into());
    }
    if let Some(scale) = ctx
        .fields
        .take_parsed::<u32>("ScaleFactor")
        .filter(|s| *s > 0)
    {
        settings.insert("scale".to_owned(), scale.into());
    }
    if let Some(mac) = ctx.fields.take("PhysicalAddress") {
        settings.insert("wol_mac".to_owned(), mac.into());
    }
    if let Some(gateway) = rds_gateway(ctx, references) {
        settings.insert("gateway".to_owned(), gateway);
    }
    if ctx.fields.peek(SECURE_GATEWAY_ID).is_some() {
        ctx.add_reason("Secure Gateway non supportato per le connessioni RDP: conservato in extra");
    }
    Ok(Connection {
        protocol: "rdp".to_owned(),
        host,
        port,
        credential,
        settings,
    })
}

/// `URI` holds only the host; the port has its own field. Port 0 means the
/// protocol's default.
fn address(ctx: &mut ObjectContext<'_>, port_field: &str) -> (String, Option<u16>) {
    let host = ctx.fields.take("URI").unwrap_or_default().to_owned();
    if host.is_empty() {
        ctx.add_reason("host mancante");
    }
    let port = ctx
        .fields
        .take_parsed::<u16>(port_field)
        .filter(|port| *port > 0);
    (host, port)
}

/// The Secure Gateway of a connection, if it is imported too.
fn secure_gateway(ctx: &mut ObjectContext<'_>, references: &References) -> Option<Uuid> {
    let value = ctx.fields.peek(SECURE_GATEWAY_ID)?;
    let Some(gateway) = Uuid::parse_str(value)
        .ok()
        .filter(|id| references.secure_gateways.contains(id))
    else {
        ctx.add_reason("Secure Gateway non trovato nel documento: jump host non importato");
        return None;
    };
    ctx.fields.take_raw(SECURE_GATEWAY_ID);
    ctx.keep_original(SECURE_GATEWAY_ID, value);
    ctx.add_reason(UNVERIFIED_JUMP_HOST);
    Some(gateway)
}

/// The Remote Desktop Gateway of an RDP connection, as `RdpGateway`
/// settings. The credential of the gateway is left to inherit: where Royal
/// keeps it is not known.
fn rds_gateway(ctx: &mut ObjectContext<'_>, references: &References) -> Option<Value> {
    if let Some(value) = ctx.fields.peek(RDS_GATEWAY_ID) {
        let Some(address) = Uuid::parse_str(value)
            .ok()
            .and_then(|id| references.rds_gateways.get(&id))
        else {
            ctx.add_reason("gateway RDP non trovato nel documento o senza host: non importato");
            return None;
        };
        ctx.fields.take_raw(RDS_GATEWAY_ID);
        ctx.keep_original(RDS_GATEWAY_ID, value);
        ctx.add_reason(UNVERIFIED_GATEWAY);
        return Some(gateway_settings(&address.host, address.port));
    }

    let host = ctx.fields.peek(GATEWAY_HOST)?;
    match ctx.fields.peek(GATEWAY_USAGE) {
        Some(usage @ ("1" | "2")) => {
            ctx.fields.take_raw(GATEWAY_HOST);
            ctx.fields.take_raw(GATEWAY_USAGE);
            ctx.keep_original(GATEWAY_HOST, host);
            ctx.keep_original(GATEWAY_USAGE, usage);
            ctx.add_reason(UNVERIFIED_GATEWAY);
            let (host, port) = split_host_port(host);
            Some(gateway_settings(host, port))
        }
        Some("0" | "4") => None,
        _ => {
            ctx.add_reason("gateway RDP con modalità d'uso sconosciuta: dati conservati in extra");
            None
        }
    }
}

fn gateway_settings(host: &str, port: Option<u16>) -> Value {
    let mut gateway = Map::new();
    gateway.insert("host".to_owned(), host.into());
    if let Some(port) = port {
        gateway.insert("port".to_owned(), port.into());
    }
    Value::Object(gateway)
}

/// `host:port` as written in RDP gateway names; anything else is a host.
fn split_host_port(value: &str) -> (&str, Option<u16>) {
    value
        .rsplit_once(':')
        .and_then(|(host, port)| Some((host, Some(port.parse().ok()?))))
        .filter(|(host, _)| !host.is_empty() && !host.contains(':'))
        .unwrap_or((value, None))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn split_host_port_reads_a_port_suffix() {
        assert_eq!(
            split_host_port("gw.example.com:4443"),
            ("gw.example.com", Some(4443))
        );
    }

    #[test]
    fn split_host_port_keeps_a_plain_host() {
        assert_eq!(split_host_port("gw.example.com"), ("gw.example.com", None));
    }

    #[test]
    fn split_host_port_leaves_ipv6_addresses_alone() {
        assert_eq!(split_host_port("2001:db8::1"), ("2001:db8::1", None));
    }
}
