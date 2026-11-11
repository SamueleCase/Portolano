//! Tests of the RDP protocol. The `freerdp_plans`, `prompting` and `clients`
//! tests work on resolved data; the `protocol` tests go through whole
//! documents and the core resolution.

use std::path::PathBuf;

use portolano_core::{CredentialSpec, SecretRef};
use uuid::Uuid;

use super::*;
use crate::plan::LaunchStep;
use crate::testutil::{environment, login, run_args};

fn strings(words: &[&str]) -> Vec<String> {
    words.iter().map(|word| (*word).to_owned()).collect()
}

fn domain_login(user: &str, domain: &str, password: Option<&SecretRef>) -> ResolvedCredential {
    ResolvedCredential {
        domain: Some(domain.into()),
        ..login(Some(user), password)
    }
}

fn target<'a>(settings: &'a RdpSettings, credential: ResolvedCredential) -> RdpTarget<'a> {
    RdpTarget {
        node_id: Uuid::nil(),
        name: "dc01",
        host: "dc01.example.com",
        port: 3389,
        credential,
        settings,
        gateway: None,
    }
}

/// A FreeRDP plan that waits for the client, so that its warnings are only
/// those about the connection.
fn freerdp_plan(target: &RdpTarget) -> LaunchPlan {
    freerdp_result(target).unwrap()
}

fn freerdp_result(target: &RdpTarget) -> Result<LaunchPlan, ConnectError> {
    freerdp::plan(
        PathBuf::from("/opt/homebrew/bin/sdl-freerdp"),
        target,
        CertPolicy::Tofu,
        true,
    )
}

/// A FreeRDP plan without `--wait`.
fn detached_freerdp_plan(target: &RdpTarget) -> LaunchPlan {
    freerdp::plan(
        PathBuf::from("/opt/homebrew/bin/sdl-freerdp"),
        target,
        CertPolicy::Tofu,
        false,
    )
    .unwrap()
}

mod clients;
mod freerdp_plans;
mod prompting;
mod protocol;
