//! The data the commands return, shaped for the frontend: nothing in them is
//! a secret or a vault reference (SPEC-UI §1.1).

mod status;

pub use status::{AppStatus, DocumentLocation, VaultKind};
