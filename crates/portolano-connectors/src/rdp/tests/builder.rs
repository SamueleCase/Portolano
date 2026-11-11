//! Plans built from resolved data: no document is needed.

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
            wait: false,
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
    assert!(
        args.iter()
            .all(|arg| !arg.contains("p:") || arg.starts_with("/gateway:g:"))
    );
    assert!(args.iter().all(|arg| !arg.starts_with("/p:")));
    assert_eq!(plan.warnings.len(), 1);
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

#[test]
fn windows_app_writes_an_rdp_file_without_password_and_opens_it() {
    let settings = RdpSettings {
        resolution: Resolution::Fixed {
            width: 1280,
            height: 800,
        },
        color_depth: Some(32),
        admin: true,
        ..RdpSettings::default()
    };
    let mut rdp = target(
        &settings,
        domain_login("administrator", "ACME", Some(&SecretRef::new())),
    );
    rdp.gateway = Some(GatewayLogin {
        host: "gw.example.com",
        port: Some(443),
        credential: login(None, None),
    });
    let app = PathBuf::from("/Applications/Windows App.app");

    let plan = windows_app::plan(&app, &rdp, false);

    assert_eq!(
        plan.steps,
        vec![
            LaunchStep::WriteTempFile {
                name: "dc01.rdp".into(),
                contents: "full address:s:dc01.example.com:3389\n\
                           username:s:ACME\\administrator\n\
                           gatewayhostname:s:gw.example.com:443\n\
                           gatewayusagemethod:i:1\n\
                           screen mode id:i:1\n\
                           desktopwidth:i:1280\n\
                           desktopheight:i:800\n\
                           session bpp:i:32\n\
                           redirectclipboard:i:1\n\
                           use multimon:i:0\n\
                           administrative session:i:1\n"
                    .into(),
                mode: 0o600,
            },
            LaunchStep::Run {
                program: PathBuf::from("/usr/bin/open"),
                args: strings(&["-a", "/Applications/Windows App.app", "{temp:dc01.rdp}"]),
                env: vec![],
                stdin_secret: None,
                askpass: None,
                wait: true,
            },
        ]
    );
    assert_eq!(plan.warnings.len(), 1);
}

#[test]
fn windows_app_fullscreen_uses_screen_mode_2() {
    let settings = RdpSettings {
        resolution: Resolution::Fullscreen,
        ..RdpSettings::default()
    };
    let app = PathBuf::from("/Applications/Windows App.app");

    let plan = windows_app::plan(&app, &target(&settings, login(None, None)), false);

    let LaunchStep::WriteTempFile { contents, .. } = &plan.steps[0] else {
        panic!("expected a temporary file");
    };
    assert!(contents.contains("screen mode id:i:2\n"));
}

#[test]
fn auto_without_any_client_suggests_installing_freerdp() {
    let mut env = environment();
    env.freerdp = None;
    env.windows_app = None;

    let result = choose_client(None, RdpClientChoice::Auto, &env);

    assert!(
        matches!(result, Err(ConnectError::ClientNotFound { ref hint, .. }) if hint.contains("brew install freerdp")),
        "{result:?}"
    );
}

#[test]
fn auto_falls_back_to_windows_app() {
    let mut env = environment();
    env.freerdp = None;

    let chosen = choose_client(None, RdpClientChoice::Auto, &env).unwrap();

    assert_eq!(
        chosen,
        ChosenClient::WindowsApp {
            app: PathBuf::from("/Applications/Windows App.app"),
            fallback: true,
        }
    );
}

#[test]
fn requested_client_overrides_the_configuration() {
    let chosen = choose_client(
        Some(RdpClient::FreeRdp),
        RdpClientChoice::WindowsApp,
        &environment(),
    )
    .unwrap();

    assert_eq!(
        chosen,
        ChosenClient::FreeRdp(PathBuf::from("/opt/homebrew/bin/sdl-freerdp"))
    );
}

#[test]
fn forced_windows_app_that_is_missing_is_an_error() {
    let mut env = environment();
    env.windows_app = None;

    let result = choose_client(None, RdpClientChoice::WindowsApp, &env);

    assert!(
        matches!(result, Err(ConnectError::ClientNotFound { .. })),
        "{result:?}"
    );
}
