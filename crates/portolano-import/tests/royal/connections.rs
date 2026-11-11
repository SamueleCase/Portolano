//! SSH and RDP connections and gateways (SPEC §8.3.5).

use portolano_core::NodeKind;
use portolano_import::ItemStatus;
use pretty_assertions::assert_eq;
use serde_json::json;

use crate::testutil::*;

fn settings(outcome: &portolano_import::ImportOutcome, n: u32) -> serde_json::Value {
    serde_json::Value::Object(connection(&outcome.root, n).settings.clone())
}

#[test]
fn ssh_maps_host_port_and_settings() {
    let xml = document(&[object(
        "RoyalSSHConnection",
        2,
        1,
        &[
            ("URI", "web01.example.com"),
            ("Port", "2222"),
            ("SSHAllowAgentForwarding", "True"),
            ("SSHEnableX11Forwarding", "True"),
            ("SSHEnableTrustedX11Forwarding", "False"),
            ("SSHEnableCompression", "True"),
            ("KeepAliveInterval", "30"),
            ("IsTelnetConnection", "False"),
        ],
    )]);

    let outcome = import(&xml);

    let ssh = connection(&outcome.root, 2);
    assert_eq!(ssh.protocol, "ssh");
    assert_eq!(ssh.host, "web01.example.com");
    assert_eq!(ssh.port, Some(2222));
    assert_eq!(
        settings(&outcome, 2),
        json!({ "agent_forwarding": true, "x11": true, "compression": true, "keepalive_s": 30 })
    );
    assert_eq!(status(&outcome, 2), ItemStatus::Imported);
}

#[test]
fn ssh_without_keepalive_or_port_uses_the_defaults() {
    let xml = document(&[object(
        "RoyalSSHConnection",
        2,
        1,
        &[
            ("URI", "web01.example.com"),
            ("Port", "0"),
            ("KeepAliveInterval", "0"),
        ],
    )]);

    let outcome = import(&xml);

    assert_eq!(connection(&outcome.root, 2).port, None);
    assert_eq!(settings(&outcome, 2), json!({}));
}

#[test]
fn terminal_fields_go_to_extra_and_are_counted() {
    let xml = document(&[object(
        "RoyalSSHConnection",
        2,
        1,
        &[("URI", "web01.example.com"), ("FontNameOSX", "Menlo")],
    )]);

    let outcome = import(&xml);

    assert_eq!(
        find(&outcome.root, uuid(2)).extra["royal.FontNameOSX"],
        "Menlo"
    );
    assert_eq!(
        outcome.report.unmapped_fields["RoyalSSHConnection"]["FontNameOSX"],
        1
    );
}

#[test]
fn telnet_and_serial_connections_are_unsupported() {
    let xml = document(&[
        object(
            "RoyalSSHConnection",
            2,
            1,
            &[("IsTelnetConnection", "True")],
        ),
        object(
            "RoyalSSHConnection",
            3,
            1,
            &[("IsSerialPortConnection", "True")],
        ),
    ]);

    let outcome = import(&xml);

    for n in [2, 3] {
        assert!(is_unsupported(find(&outcome.root, uuid(n))));
        assert_eq!(status(&outcome, n), ItemStatus::Unsupported);
    }
    assert_eq!(
        reasons(&outcome, 2),
        vec!["connessione Telnet non supportata"]
    );
}

#[test]
fn a_connection_without_host_is_partial() {
    let xml = document(&[object("RoyalSSHConnection", 2, 1, &[("Name", "vuota")])]);

    let outcome = import(&xml);

    assert_eq!(reasons(&outcome, 2), vec!["host mancante"]);
}

#[test]
fn a_secure_gateway_becomes_a_jump_host_of_its_users() {
    let xml = document(&[
        object(
            "RoyalSecureGateway",
            3,
            1,
            &[("URI", "jump.example.com"), ("Port", "22")],
        ),
        object(
            "RoyalSSHConnection",
            2,
            1,
            &[("URI", "10.example.com"), ("SecureGatewayID", &id(3))],
        ),
    ]);

    let outcome = import(&xml);

    let gateway = connection(&outcome.root, 3);
    assert_eq!(
        (gateway.protocol.as_str(), gateway.host.as_str()),
        ("ssh", "jump.example.com")
    );
    assert_eq!(
        settings(&outcome, 2)["jump_hosts"],
        json!([{ "$ref": id(3) }])
    );
    let node = find(&outcome.root, uuid(2));
    assert_eq!(node.extra["royal.SecureGatewayID"], id(3).as_str());
    assert_eq!(status(&outcome, 2), ItemStatus::Partial);
    assert!(
        !outcome
            .report
            .unmapped_fields
            .contains_key("RoyalSSHConnection")
    );
}

#[test]
fn a_missing_secure_gateway_is_kept_in_extra() {
    let xml = document(&[object(
        "RoyalSSHConnection",
        2,
        1,
        &[("URI", "a.example.com"), ("SecureGatewayID", &id(50))],
    )]);

    let outcome = import(&xml);

    assert_eq!(settings(&outcome, 2), json!({}));
    assert_eq!(
        find(&outcome.root, uuid(2)).extra["royal.SecureGatewayID"],
        id(50).as_str()
    );
    assert_eq!(
        reasons(&outcome, 2),
        vec!["Secure Gateway non trovato nel documento: jump host non importato"]
    );
}

fn rdp(fields: &[(&str, &str)]) -> String {
    let mut all = vec![("URI", "192.0.2.10")];
    all.extend_from_slice(fields);
    document(&[object("RoyalRDSConnection", 2, 1, &all)])
}

#[test]
fn rdp_maps_host_port_and_display_settings() {
    let xml = rdp(&[
        ("RDPPort", "3390"),
        ("DesktopWidth", "1920"),
        ("DesktopHeight", "1080"),
        ("ColorDepth", "24"),
        ("ScaleFactor", "140"),
        ("PhysicalAddress", "00:00:5E:00:53:01"),
    ]);

    let outcome = import(&xml);

    let rdp = connection(&outcome.root, 2);
    assert_eq!((rdp.protocol.as_str(), rdp.port), ("rdp", Some(3390)));
    assert_eq!(
        settings(&outcome, 2),
        json!({
            "resolution": { "mode": "fixed", "width": 1920, "height": 1080 },
            "color_depth": 24,
            "scale": 140,
            "wol_mac": "00:00:5E:00:53:01"
        })
    );
}

#[test]
fn rdp_with_zero_size_and_scale_fits_the_window() {
    let xml = rdp(&[
        ("DesktopWidth", "0"),
        ("DesktopHeight", "0"),
        ("ScaleFactor", "0"),
    ]);

    let outcome = import(&xml);

    assert_eq!(settings(&outcome, 2), json!({}));
}

#[test]
fn a_reusable_rd_gateway_is_copied_into_the_connection() {
    let xml = document(&[
        object(
            "RoyalRDSGateway",
            3,
            1,
            &[("URI", "rdgw.example.com"), ("Port", "443")],
        ),
        object(
            "RoyalRDSConnection",
            2,
            1,
            &[("URI", "ts.example.com"), ("RDSGatewayID", &id(3))],
        ),
    ]);

    let outcome = import(&xml);

    assert_eq!(
        settings(&outcome, 2)["gateway"],
        json!({ "host": "rdgw.example.com", "port": 443 })
    );
    assert_eq!(status(&outcome, 2), ItemStatus::Partial);
    assert!(matches!(
        find(&outcome.root, uuid(3)).kind,
        NodeKind::Unsupported(_)
    ));
}

#[test]
fn an_inline_rd_gateway_in_use_is_mapped() {
    let xml = rdp(&[
        ("GatewayHostName", "gw.example.com:8443"),
        ("GatewayUsageMethod", "1"),
    ]);

    let outcome = import(&xml);

    assert_eq!(
        settings(&outcome, 2)["gateway"],
        json!({ "host": "gw.example.com", "port": 8443 })
    );
    let extra = &find(&outcome.root, uuid(2)).extra;
    assert_eq!(extra["royal.GatewayHostName"], "gw.example.com:8443");
    assert_eq!(status(&outcome, 2), ItemStatus::Partial);
}

#[test]
fn a_disabled_inline_rd_gateway_is_only_kept_in_extra() {
    let xml = rdp(&[
        ("GatewayHostName", "gw.example.com"),
        ("GatewayUsageMethod", "0"),
    ]);

    let outcome = import(&xml);

    assert_eq!(settings(&outcome, 2), json!({}));
    assert_eq!(
        find(&outcome.root, uuid(2)).extra["royal.GatewayHostName"],
        "gw.example.com"
    );
    assert_eq!(status(&outcome, 2), ItemStatus::Imported);
}

#[test]
fn an_rd_gateway_with_unknown_usage_is_partial() {
    let xml = rdp(&[("GatewayHostName", "gw.example.com")]);

    let outcome = import(&xml);

    assert_eq!(settings(&outcome, 2), json!({}));
    assert_eq!(status(&outcome, 2), ItemStatus::Partial);
}

#[test]
fn a_secure_gateway_on_rdp_is_not_supported() {
    let xml = rdp(&[("SecureGatewayID", &id(3))]);

    let outcome = import(&xml);

    assert_eq!(
        reasons(&outcome, 2),
        vec!["Secure Gateway non supportato per le connessioni RDP: conservato in extra"]
    );
}
