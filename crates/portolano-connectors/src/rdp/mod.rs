//! The RDP protocol, through FreeRDP or Windows App (SPEC §7.3).

// Stub (WP0): the parameters are used once WP4 implements the bodies.
#![expect(unused_variables)]

mod freerdp;
mod settings;
mod windows_app;

use portolano_core::{Document, Issue, Node};
use serde_json::Value;

pub use settings::{
    CertPolicy, RdpClient, RdpClientChoice, RdpConfig, RdpGateway, RdpSettings, Resolution,
};

use crate::ConnectError;
use crate::plan::LaunchPlan;
use crate::protocol::{PlanContext, Protocol};

/// The `rdp` protocol.
#[derive(Debug, Clone, Copy, Default)]
pub struct RdpProtocol;

impl Protocol for RdpProtocol {
    fn id(&self) -> &'static str {
        todo!()
    }

    fn display_name(&self) -> &'static str {
        todo!()
    }

    fn default_port(&self) -> u16 {
        todo!()
    }

    fn validate_settings(&self, settings: &serde_json::Map<String, Value>) -> Result<(), String> {
        todo!()
    }

    fn validate_config(&self, section: &toml::Value) -> Result<(), String> {
        todo!()
    }

    fn validate(&self, doc: &Document, node: &Node) -> Vec<Issue> {
        todo!()
    }

    fn summary(&self, doc: &Document, node: &Node) -> String {
        todo!()
    }

    fn plan(&self, ctx: &PlanContext) -> Result<LaunchPlan, ConnectError> {
        todo!()
    }
}
