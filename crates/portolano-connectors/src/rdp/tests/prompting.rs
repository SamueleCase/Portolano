//! FreeRDP in the foreground when it has to ask for a password (DEC-025).

use pretty_assertions::assert_eq;

use super::*;

fn waits(plan: &LaunchPlan) -> bool {
    matches!(&plan.steps[0], LaunchStep::Run { wait: true, .. })
}

fn foreground_warnings(plan: &LaunchPlan) -> Vec<&String> {
    plan.warnings
        .iter()
        .filter(|warning| warning.contains("primo piano"))
        .collect()
}

fn gateway<'a>(credential: ResolvedCredential) -> Option<GatewayLogin<'a>> {
    Some(GatewayLogin {
        host: "gw.example.com",
        port: None,
        credential,
    })
}

#[test]
fn destination_with_a_password_stays_detached() {
    let settings = RdpSettings::default();
    let rdp = target(
        &settings,
        login(Some("administrator"), Some(&SecretRef::new())),
    );

    let plan = detached_freerdp_plan(&rdp);

    assert!(!waits(&plan));
    assert!(foreground_warnings(&plan).is_empty());
}

#[test]
fn destination_without_a_password_runs_in_the_foreground() {
    let settings = RdpSettings::default();
    let rdp = target(&settings, login(Some("administrator"), None));

    let plan = detached_freerdp_plan(&rdp);

    assert!(waits(&plan));
    assert!(
        run_args(&plan)
            .iter()
            .all(|arg| !arg.starts_with("/log-level:"))
    );
    let warnings = foreground_warnings(&plan);
    assert_eq!(warnings.len(), 1, "{:?}", plan.warnings);
    assert!(
        warnings[0].contains("password della destinazione"),
        "{warnings:?}"
    );
}

#[test]
fn password_without_a_user_runs_in_the_foreground() {
    let settings = RdpSettings::default();
    let rdp = target(&settings, login(None, Some(&SecretRef::new())));

    let plan = detached_freerdp_plan(&rdp);

    assert!(waits(&plan));
}

#[test]
fn gateway_with_another_login_runs_in_the_foreground() {
    let password = SecretRef::new();
    let settings = RdpSettings::default();
    let mut rdp = target(&settings, login(Some("administrator"), Some(&password)));
    rdp.gateway = gateway(login(Some("gwuser"), Some(&SecretRef::new())));

    let plan = detached_freerdp_plan(&rdp);

    assert!(waits(&plan));
    let warnings = foreground_warnings(&plan);
    assert_eq!(warnings.len(), 1, "{:?}", plan.warnings);
    assert!(warnings[0].contains("password del gateway"), "{warnings:?}");
    // The destination password still goes to stdin.
    assert!(matches!(
        &plan.steps[0],
        LaunchStep::Run { stdin_secret: Some(secret), .. } if *secret == password
    ));
}

#[test]
fn gateway_asked_at_launch_runs_in_the_foreground() {
    let settings = RdpSettings::default();
    let mut rdp = target(
        &settings,
        login(Some("administrator"), Some(&SecretRef::new())),
    );
    rdp.gateway = gateway(ResolvedCredential {
        origin: portolano_core::CredentialOrigin::Prompt,
        ..login(None, None)
    });

    let plan = detached_freerdp_plan(&rdp);

    assert!(waits(&plan));
}

#[test]
fn gateway_with_the_destination_login_stays_detached() {
    let password = SecretRef::new();
    let settings = RdpSettings::default();
    let mut rdp = target(&settings, login(Some("administrator"), Some(&password)));
    rdp.gateway = gateway(login(Some("administrator"), Some(&password)));

    let plan = detached_freerdp_plan(&rdp);

    assert!(!waits(&plan));
    assert!(foreground_warnings(&plan).is_empty());
}

#[test]
fn both_prompts_are_named_in_one_warning() {
    let settings = RdpSettings::default();
    let mut rdp = target(&settings, login(Some("administrator"), None));
    rdp.gateway = gateway(login(Some("gwuser"), None));

    let plan = detached_freerdp_plan(&rdp);

    let warnings = foreground_warnings(&plan);
    assert_eq!(warnings.len(), 1, "{:?}", plan.warnings);
    assert!(
        warnings[0].contains("della destinazione e quella del gateway"),
        "{warnings:?}"
    );
}

#[test]
fn requested_wait_needs_no_foreground_warning() {
    let settings = RdpSettings::default();
    let rdp = target(&settings, login(Some("administrator"), None));

    let plan = freerdp_plan(&rdp);

    assert!(waits(&plan));
    assert!(foreground_warnings(&plan).is_empty());
}
