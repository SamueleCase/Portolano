//! The data the commands exchange with the frontend: nothing they return is
//! a secret (SPEC-UI §1.1); secrets typed in the interface arrive as
//! [`SecretText`] and become secrets at once (SPEC-UI §1.2).

mod details;
mod icon;
mod import;
mod inputs;
mod json;
mod secret;
mod settings;
mod status;
mod tree;

pub use details::{NodeDetailsView, SecretView, node_details};
pub use icon::IconView;
pub use import::{ImportAnswerInput, ImportOptionsInput, ImportPreviewView};
pub use inputs::{
    DeleteOptionsInput, NewNodeInput, NodeEditInput, PlacementInput, SecretSlotInput,
};
pub use secret::SecretText;
pub use settings::{ConfigEntry, VaultStatus, config_entries};
pub use status::{AppStatus, ChosenFile, DocumentInfo, DocumentLocation, VaultKind, file_name};
pub use tree::{TreeView, tree_view};
