//! References between nodes and structural validation (SPEC §4.4).

// Stub (WP0): the parameters are used once WP1 implements the bodies.
#![expect(unused_variables)]

use uuid::Uuid;

use crate::error::Issue;
use crate::model::Document;

impl Document {
    /// The nodes that refer to `target`: those containing `{"$ref": target}`
    /// anywhere (children excluded) and, if `target` is a credential, those
    /// with a `ByName` spec equal to its name.
    pub fn references_to(&self, target: Uuid) -> Vec<Uuid> {
        todo!()
    }

    /// Structural problems of the document, with the codes `duplicate_id`,
    /// `dangling_ref`, `credential_ref_not_credential`, `by_name_unresolved`,
    /// `by_name_ambiguous` and `empty_host`. Protocol and vault checks are
    /// added by the caller.
    pub fn validate(&self) -> Vec<Issue> {
        todo!()
    }
}
