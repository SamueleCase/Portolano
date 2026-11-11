//! Registry of the available importers.

use std::path::Path;

use crate::Importer;
use crate::royal::RoyalImporter;

/// The importers known to Portolano.
pub struct Registry {
    importers: Vec<Box<dyn Importer>>,
}

impl Registry {
    /// A registry with the built-in importers: `royal`.
    pub fn with_defaults() -> Registry {
        Registry {
            importers: vec![Box::new(RoyalImporter)],
        }
    }

    /// The importer with the given id (`import --format`).
    pub fn get(&self, id: &str) -> Option<&dyn Importer> {
        self.importers
            .iter()
            .map(Box::as_ref)
            .find(|importer| importer.id() == id)
    }

    /// The ids of the registered importers, in registration order.
    pub fn ids(&self) -> Vec<&'static str> {
        self.importers
            .iter()
            .map(|importer| importer.id())
            .collect()
    }

    /// The first importer that recognizes the file.
    pub fn detect(&self, path: &Path, head: &[u8]) -> Option<&dyn Importer> {
        self.importers
            .iter()
            .map(Box::as_ref)
            .find(|importer| importer.detect(path, head))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn defaults_register_royal() {
        let registry = Registry::with_defaults();

        assert_eq!(registry.ids(), vec!["royal"]);
        assert_eq!(registry.get("royal").map(|i| i.id()), Some("royal"));
        assert!(registry.get("mremoteng").is_none());
    }

    #[test]
    fn detect_returns_the_importer_that_recognizes_the_file() {
        let registry = Registry::with_defaults();

        let found = registry.detect(Path::new("doc.rtsz"), b"<RTSZDocument>");

        assert_eq!(found.map(|i| i.id()), Some("royal"));
    }

    #[test]
    fn detect_returns_none_for_an_unknown_file() {
        let registry = Registry::with_defaults();

        assert!(registry.detect(Path::new("notes.txt"), b"hello").is_none());
    }
}
