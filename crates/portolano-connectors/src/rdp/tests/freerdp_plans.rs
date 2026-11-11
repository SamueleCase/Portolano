//! FreeRDP plans built from resolved data: no document is needed.

use pretty_assertions::assert_eq;

use super::*;

#[test]
fn freerdp_fit_passes_the_password_on_stdin_only() {
    let password = SecretRef::new();
    let settings = RdpSettings::default();

    let plan = freerdp_plan(&target(
        &settings,
        domain_login("administrator", "ACME", Some(&password)),
    ));

    assert_eq!(
        plan.steps,
        vec![LaunchStep::Run {
            program: PathBuf::from("/opt/homebrew/bin/sdl-freerdp"),
            args: strings(&[
                "/v:dc01.example.com:3389",
                "/u:administrator",
                "/d:ACME",
                "/cert:tofu",
                "/dynamic-resolution",
                "+clipboard",
                "/from-stdin:force",
            ]),
            env: vec![],
            stdin_secret: Some(password),
            askpass: None,
            wait: true,
        }]
    );
}

#[test]
fn freerdp_fixed_size_and_display_options() {
    let settings = RdpSettings {
        resolution: Resolution::Fixed {
            width: 1920,
            height: 1080,
        },
        scale: Some(140),
        color_depth: Some(24),
        multimon: true,
        admin: true,
        clipboard: false,
        ..RdpSettings::default()
    };

    let plan = freerdp_plan(&target(&settings, login(None, None)));

    assert_eq!(
        run_args(&plan),
        strings(&[
            "/v:dc01.example.com:3389",
            "/cert:tofu",
            "/size:1920x1080",
            "/scale:140",
            "/bpp:24",
            "/multimon",
            "/admin",
            "-clipboard",
        ])
    );
}

#[test]
fn freerdp_fullscreen_uses_f() {
    let settings = RdpSettings {
        resolution: Resolution::Fullscreen,
        ..RdpSettings::default()
    };

    let plan = freerdp_plan(&target(&settings, login(None, None)));

    assert!(run_args(&plan).contains(&"/f".to_owned()));
}

#[test]
fn freerdp_gateway_with_the_same_login_reuses_the_credentials() {
    let password = SecretRef::new();
    let settings = RdpSettings::default();
    let mut rdp = target(
        &settings,
        domain_login("administrator", "ACME", Some(&password)),
    );
    rdp.gateway = Some(GatewayLogin {
        host: "gw.example.com",
        port: Some(443),
        credential: domain_login("administrator", "ACME", Some(&password)),
    });

    let plan = freerdp_plan(&rdp);

    let args = run_args(&plan);
    assert!(
        args.contains(&"/gateway:g:gw.example.com:443".to_owned()),
        "{args:?}"
    );
    assert!(plan.warnings.is_empty());
}

#[test]
fn freerdp_gateway_with_another_login_passes_user_and_domain_but_never_the_password() {
    let settings = RdpSettings::default();
    let mut rdp = target(
        &settings,
        domain_login("administrator", "ACME", Some(&SecretRef::new())),
    );
    rdp.gateway = Some(GatewayLogin {
        host: "gw.example.com",
        port: None,
        credential: domain_login("gwuser", "EDGE", Some(&SecretRef::new())),
    });

    let plan = freerdp_plan(&rdp);

    let args = run_args(&plan);
    assert!(
        args.contains(&"/gateway:g:gw.example.com,u:gwuser,d:EDGE".to_owned()),
        "{args:?}"
    );
    let gateway = args
        .iter()
        .find(|arg| arg.starts_with("/gateway:"))
        .unwrap();
    assert!(!gateway.contains(",p:"), "{gateway}");
    assert!(args.iter().all(|arg| !arg.starts_with("/p:")));
    assert_eq!(plan.warnings.len(), 1);
}

#[test]
fn freerdp_gateway_user_without_password_still_warns() {
    let settings = RdpSettings::default();
    let mut rdp = target(&settings, login(Some("administrator"), None));
    rdp.gateway = Some(GatewayLogin {
        host: "gw.example.com",
        port: None,
        credential: login(Some("gwuser"), None),
    });

    let plan = freerdp_plan(&rdp);

    assert!(run_args(&plan).contains(&"/gateway:g:gw.example.com,u:gwuser".to_owned()));
    assert_eq!(plan.warnings.len(), 1);
}

#[test]
fn freerdp_gateway_with_prompt_credential_warns_and_passes_no_user() {
    let settings = RdpSettings::default();
    let mut rdp = target(&settings, login(Some("administrator"), None));
    rdp.gateway = Some(GatewayLogin {
        host: "gw.example.com",
        port: None,
        credential: ResolvedCredential {
            origin: portolano_core::CredentialOrigin::Prompt,
            ..login(None, None)
        },
    });

    let plan = freerdp_plan(&rdp);

    assert!(run_args(&plan).contains(&"/gateway:g:gw.example.com".to_owned()));
    assert_eq!(plan.warnings.len(), 1);
}

#[test]
fn freerdp_gateway_user_with_a_comma_is_rejected() {
    let settings = RdpSettings::default();
    let mut rdp = target(&settings, login(Some("administrator"), None));
    rdp.gateway = Some(GatewayLogin {
        host: "gw.example.com",
        port: None,
        credential: login(Some("gwuser,p:injected"), None),
    });

    let result = freerdp_result(&rdp);

    assert!(
        matches!(result, Err(ConnectError::InvalidValue { ref field, .. }) if field == "gateway.username"),
        "{result:?}"
    );
}

#[test]
fn freerdp_gateway_host_with_a_control_character_is_rejected() {
    let settings = RdpSettings::default();
    let mut rdp = target(&settings, login(Some("administrator"), None));
    rdp.gateway = Some(GatewayLogin {
        host: "gw.example.com\t",
        port: None,
        credential: login(None, None),
    });

    let result = freerdp_result(&rdp);

    assert!(
        matches!(result, Err(ConnectError::InvalidValue { ref field, .. }) if field == "gateway.host"),
        "{result:?}"
    );
}

fn assert_invalid_field(result: &Result<LaunchPlan, ConnectError>, expected: &str) {
    assert!(
        matches!(
            result,
            Err(ConnectError::InvalidValue { field, reason, .. })
                if field == expected && reason.contains("carattere di controllo")
        ),
        "{result:?}"
    );
}

#[test]
fn freerdp_host_with_a_control_character_is_rejected() {
    let settings = RdpSettings::default();
    let mut rdp = target(&settings, login(Some("administrator"), None));
    rdp.host = "dc01\u{1b}x.example.com";

    let result = freerdp_result(&rdp);

    assert_invalid_field(&result, "host");
}

#[test]
fn freerdp_user_with_a_control_character_is_rejected() {
    let settings = RdpSettings::default();

    let result = freerdp_result(&target(&settings, login(Some("admin\u{1b}x"), None)));

    assert_invalid_field(&result, "username");
}

#[test]
fn freerdp_domain_with_a_control_character_is_rejected() {
    let settings = RdpSettings::default();

    let result = freerdp_result(&target(
        &settings,
        domain_login("administrator", "ACME\n", None),
    ));

    assert_invalid_field(&result, "domain");
}

#[test]
fn freerdp_password_without_user_is_not_sent() {
    let settings = RdpSettings::default();

    let plan = freerdp_plan(&target(&settings, login(None, Some(&SecretRef::new()))));

    assert!(!run_args(&plan).contains(&"/from-stdin:force".to_owned()));
    assert!(matches!(
        &plan.steps[0],
        LaunchStep::Run {
            stdin_secret: None,
            ..
        }
    ));
    assert_eq!(plan.warnings.len(), 1);
}

#[test]
fn detached_freerdp_says_where_its_log_goes() {
    let settings = RdpSettings::default();
    let rdp = target(
        &settings,
        login(Some("administrator"), Some(&SecretRef::new())),
    );

    let plan = detached_freerdp_plan(&rdp);

    assert!(matches!(
        &plan.steps[0],
        LaunchStep::Run { wait: false, .. }
    ));
    assert_eq!(plan.warnings.len(), 1);
    assert!(plan.warnings[0].contains("log"), "{:?}", plan.warnings);
}

#[test]
fn detached_freerdp_logs_only_warnings_and_errors() {
    let settings = RdpSettings::default();
    let rdp = target(
        &settings,
        login(Some("administrator"), Some(&SecretRef::new())),
    );

    let plan = detached_freerdp_plan(&rdp);

    assert!(run_args(&plan).contains(&"/log-level:WARN".to_owned()));
}

#[test]
fn waited_freerdp_keeps_its_own_log_level() {
    let settings = RdpSettings::default();

    let plan = freerdp_plan(&target(&settings, login(Some("administrator"), None)));

    assert!(
        run_args(&plan)
            .iter()
            .all(|arg| !arg.starts_with("/log-level:"))
    );
}

#[test]
fn freerdp_without_password_does_not_read_stdin() {
    let settings = RdpSettings::default();

    let plan = freerdp_plan(&target(&settings, login(Some("administrator"), None)));

    assert!(!run_args(&plan).contains(&"/from-stdin:force".to_owned()));
    assert!(matches!(
        &plan.steps[0],
        LaunchStep::Run {
            stdin_secret: None,
            ..
        }
    ));
}
