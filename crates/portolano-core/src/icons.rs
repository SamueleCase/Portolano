//! Store of custom icon images (SPEC §6.4).
//!
//! Images live in `<dir of the document>/<stem of the document>.icons/` as
//! `<sha256>.<ext>`. Accepted types: png, jpeg, gif, tiff, svg+xml, pdf; the
//! type is recognized from the magic bytes and the extension derived from it.

// Stub (WP0): the parameters are used once WP2 implements the bodies.
#![expect(unused_variables)]

use std::path::PathBuf;

use crate::error::Result;
use crate::model::{Document, Icon};

/// The icon directory of one document. Obtained from
/// [`Store::icons`](crate::store::Store::icons).
#[derive(Debug, Clone)]
// Stub (WP0): the field is read once WP2 implements the methods.
#[expect(dead_code)]
pub struct IconStore {
    dir: PathBuf,
}

impl IconStore {
    /// Stores an image, deduplicated by hash, and returns the matching
    /// [`Icon::Custom`].
    ///
    /// # Errors
    /// [`Error::UnsupportedIcon`](crate::Error::UnsupportedIcon) for a type
    /// that is not accepted, [`Error::Io`](crate::Error::Io).
    pub fn add(&self, bytes: &[u8], mime: &str) -> Result<Icon> {
        todo!()
    }

    /// The file of a custom icon; `None` for other icon kinds.
    pub fn path_for(&self, icon: &Icon) -> Option<PathBuf> {
        todo!()
    }

    /// Removes the images no longer used by `doc` and returns how many were
    /// removed.
    ///
    /// # Errors
    /// [`Error::Io`](crate::Error::Io).
    pub fn gc(&self, doc: &Document) -> Result<usize> {
        todo!()
    }
}
