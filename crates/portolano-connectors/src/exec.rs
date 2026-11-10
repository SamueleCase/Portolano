//! Execution of launch plans (SPEC §7.2, §7.3).

// Stub (WP0): the parameters are used once WP4 implements the body.
#![expect(unused_variables)]

use portolano_core::{AppConfig, Vault};

use crate::ConnectError;
use crate::plan::LaunchPlan;

/// Runs the steps of `plan`, reading the referenced secrets from `vault`
/// only when needed and passing them through stdin or askpass.
///
/// Returns the exit code of the last waited program, or 0 if it detached.
///
/// # Errors
/// [`ConnectError::MissingSecret`], [`ConnectError::AgentUnavailable`],
/// [`ConnectError::Spawn`], [`ConnectError::Io`], [`ConnectError::Askpass`],
/// vault errors.
pub fn execute(
    plan: &LaunchPlan,
    vault: &dyn Vault,
    config: &AppConfig,
) -> Result<i32, ConnectError> {
    todo!()
}
