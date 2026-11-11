//! Users and hosts that the plan refuses or accepts (DEC-024.3, DEC-026.2).

use pretty_assertions::assert_eq;

use super::*;

#[test]
fn jump_host_with_a_comma_is_rejected() {
    let settings = SshSettings::default();
    let jumps = vec![Jump::Endpoint(endpoint(
        "gw.example.com,evil.example.com",
        22,
        login(None, None),
    ))];
    let target = endpoint("web01.example.com", 22, login(None, None));

    let result = build(&launch(target, jumps, &settings));

    assert!(
        matches!(result, Err(ConnectError::InvalidValue { ref field, .. }) if field == "host"),
        "{result:?}"
    );
}

#[test]
fn jump_host_with_shell_characters_is_rejected() {
    let settings = SshSettings::default();
    for host in [
        "gw.example.com;id",
        "$(id).example.com",
        "gw`id`",
        "gw'x",
        "gw\"x",
        "a|b",
        "a&b",
        "a<b",
        "a>b",
        "a\\b",
    ] {
        let jumps = vec![Jump::Endpoint(endpoint(host, 22, login(None, None)))];
        let target = endpoint("web01.example.com", 22, login(None, None));

        let result = build(&launch(target, jumps, &settings));

        assert!(
            matches!(result, Err(ConnectError::InvalidValue { .. })),
            "{host}: {result:?}"
        );
    }
}

#[test]
fn jump_user_with_a_space_or_control_character_is_rejected() {
    let settings = SshSettings::default();
    for user in ["ops admin", "ops\nadmin"] {
        let jumps = vec![Jump::Endpoint(endpoint(
            "gw.example.com",
            22,
            login(Some(user), None),
        ))];
        let target = endpoint("web01.example.com", 22, login(None, None));

        let result = build(&launch(target, jumps, &settings));

        assert!(
            matches!(result, Err(ConnectError::InvalidValue { ref field, .. }) if field == "username"),
            "{result:?}"
        );
    }
}

#[test]
fn target_host_with_a_control_or_shell_character_is_rejected() {
    let settings = SshSettings::default();
    for host in ["web01\u{1b}[2J.example.com", "web01.example.com;id", "a b"] {
        let target = endpoint(host, 22, login(None, None));

        let result = build(&launch(target, vec![], &settings));

        assert!(
            matches!(result, Err(ConnectError::InvalidValue { ref field, .. }) if field == "host"),
            "{host:?}: {result:?}"
        );
    }
}

#[test]
fn target_user_with_a_control_or_shell_character_is_rejected() {
    let settings = SshSettings::default();
    for user in [
        "admin\r",
        "admin$(id)",
        "admin;id",
        "a`id`",
        "a'b",
        "a\"b",
        "a|b",
        "a&b",
        "a<b",
        "a>b",
        "a{b}",
        "-oProxyCommand=x",
        "mario -oProxyCommand=x",
        "admin\\",
    ] {
        let target = endpoint("web01.example.com", 22, login(Some(user), None));

        let result = build(&launch(target, vec![], &settings));

        assert!(
            matches!(result, Err(ConnectError::InvalidValue { ref field, .. }) if field == "username"),
            "{user:?}: {result:?}"
        );
    }
}

#[test]
fn target_user_with_a_windows_domain_or_a_space_is_accepted() {
    let settings = SshSettings::default();
    for user in ["ACME\\administrator", "mario rossi", "ad,min"] {
        let target = endpoint("web01.example.com", 22, login(Some(user), None));

        let plan = build(&launch(target, vec![], &settings)).unwrap();

        assert!(
            run_args(&plan).windows(2).any(|pair| pair == ["-l", user]),
            "{user:?}"
        );
    }
}

#[test]
fn jump_user_keeps_the_stricter_rule() {
    let settings = SshSettings::default();
    let jumps = vec![Jump::Endpoint(endpoint(
        "gw.example.com",
        22,
        login(Some("ACME\\ops"), None),
    ))];
    let target = endpoint("web01.example.com", 22, login(None, None));

    let result = build(&launch(target, jumps, &settings));

    assert!(
        matches!(result, Err(ConnectError::InvalidValue { ref field, .. }) if field == "username"),
        "{result:?}"
    );
}

#[test]
fn ipv6_target_is_accepted() {
    let settings = SshSettings::default();
    let target = endpoint("2001:db8::10", 22, login(Some("admin"), None));

    let plan = build(&launch(target, vec![], &settings)).unwrap();

    assert_eq!(
        run_args(&plan).last().map(String::as_str),
        Some("2001:db8::10")
    );
}
