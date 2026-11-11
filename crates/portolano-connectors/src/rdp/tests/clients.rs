//! Windows App plans and the choice of the RDP client, from resolved data.

use pretty_assertions::assert_eq;

use super::*;

#[test]
fn windows_app_rejects_a_line_break_in_the_host() {
    let settings = RdpSettings::default();
    let mut rdp = target(&settings, login(Some("administrator"), None));
    rdp.host = "dc01.example.com\nusername:s:intruder";
    let app = PathBuf::from("/Applications/Windows App.app");

    let result = windows_app::plan(&app, &rdp, false);

    assert!(
        matches!(result, Err(ConnectError::InvalidValue { ref field, .. }) if field == "host"),
        "{result:?}"
    );
}

#[test]
fn windows_app_rejects_a_carriage_return_in_the_domain() {
    let settings = RdpSettings::default();
    let rdp = target(&settings, domain_login("administrator", "ACME\r", None));
    let app = PathBuf::from("/Applications/Windows App.app");

    let result = windows_app::plan(&app, &rdp, false);

    assert!(
        matches!(result, Err(ConnectError::InvalidValue { ref field, .. }) if field == "domain"),
        "{result:?}"
    );
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

    let plan = windows_app::plan(&app, &rdp, false).unwrap();

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

    let plan = windows_app::plan(&app, &target(&settings, login(None, None)), false).unwrap();

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
