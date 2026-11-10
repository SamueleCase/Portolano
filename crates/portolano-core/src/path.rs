//! Path syntax and resolution (SPEC §4.2, DEC-002).
//!
//! Segments are separated by `/`; a leading `/` is optional and `/` alone is
//! the root. In names, `\/` is a literal slash and `\\` a backslash.
//! `@<uuid>` or `@<hex prefix of at least 6 digits>` addresses a node by id,
//! anywhere in a path.

// Stub (WP0): the parameters are used once WP1 implements the bodies.
#![expect(unused_variables)]

use uuid::Uuid;

use crate::error::Result;
use crate::model::Document;

impl Document {
    /// The path of a node, with escapes applied; `None` if the node does not
    /// exist.
    pub fn path_of(&self, id: Uuid) -> Option<String> {
        todo!()
    }

    /// Resolves a path to a node id. Names are compared exactly first, then
    /// case-insensitively if nothing matched.
    ///
    /// # Errors
    /// [`Error::NotFound`](crate::Error::NotFound) if nothing matches,
    /// [`Error::Ambiguous`](crate::Error::Ambiguous) if several nodes match,
    /// [`Error::InvalidPath`](crate::Error::InvalidPath) for bad syntax.
    pub fn resolve_path(&self, path: &str) -> Result<Uuid> {
        todo!()
    }

    /// Splits a path into its parent and its last segment, for the `add`
    /// commands. The parent (`None` = root) must exist and be a folder; the
    /// document is not modified.
    ///
    /// # Errors
    /// As [`Document::resolve_path`] for the parent, plus
    /// [`Error::NotAFolder`](crate::Error::NotAFolder) if the parent is not a
    /// folder and [`Error::InvalidPath`](crate::Error::InvalidPath) if the
    /// last segment is missing.
    pub fn resolve_parent_path(&self, path: &str) -> Result<(Option<Uuid>, String)> {
        todo!()
    }

    /// Like [`Document::resolve_parent_path`], but creates the missing
    /// intermediate folders (`add … -p`).
    ///
    /// # Errors
    /// As [`Document::resolve_parent_path`], except that missing folders are
    /// created instead of reported.
    pub fn ensure_parent_path(&mut self, path: &str) -> Result<(Option<Uuid>, String)> {
        todo!()
    }
}
