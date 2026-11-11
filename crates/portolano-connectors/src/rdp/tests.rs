//! Tests of the RDP protocol. The `builder` tests work on resolved data; the
//! `protocol` tests go through whole documents and the core resolution.

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
        name: "dc01",
        host: "dc01.example.com",
        port: 3389,
        credential,
        settings,
        gateway: None,
    }
}

fn freerdp_plan(target: &RdpTarget) -> LaunchPlan {
    freerdp::plan(
        PathBuf::from("/opt/homebrew/bin/sdl-freerdp"),
        target,
        CertPolicy::Tofu,
        false,
    )
}

mod builder;
mod protocol;
