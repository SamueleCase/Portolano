//! Registry of the available importers.

// Stub (WP0): the parameters are used once WP5 implements the bodies.
#![expect(unused_variables)]

use std::path::Path;

use crate::Importer;

/// The importers known to Portolano.
// Stub (WP0): the field is read once WP5 implements the methods.
#[expect(dead_code)]
pub struct Registry {
    importers: Vec<Box<dyn Importer>>,
}

impl Registry {
    /// A registry with the built-in importers: `royal`.
    pub fn with_defaults() -> Registry {
        todo!()
    }

    /// The importer with the given id (`import --format`).
    pub fn get(&self, id: &str) -> Option<&dyn Importer> {
        todo!()
    }

    /// The ids of the registered importers, in registration order.
    pub fn ids(&self) -> Vec<&'static str> {
        todo!()
    }

    /// The first importer that recognizes the file.
    pub fn detect(&self, path: &Path, head: &[u8]) -> Option<&dyn Importer> {
        todo!()
    }
}
