//! Importer of Royal TS/TSX documents (`.rtsz`, `.rtsx`), SPEC §8.3 and
//! `docs/royal-format.md`.
//!
//! The import runs in four steps: recognize the container ([`sniff`]), parse
//! the objects ([`xml`]), rebuild the tree ([`tree`]) and map every object
//! to a node ([`map`]). Encrypted Royal secrets are never decrypted nor kept
//! (D3): the user may type them through the [`SecretPrompter`].

mod defaults;
mod fields;
mod map;
mod sniff;
mod tree;
mod types;
mod xml;

use std::path::Path;

use crate::{ImportError, ImportOptions, ImportOutcome, Importer, SecretPrompter};

/// Id of the importer, used by `import --format`.
const IMPORTER_ID: &str = "royal";
/// Name of the objects whose `Name` is empty or missing.
const UNNAMED: &str = "(senza nome)";
const EXTENSIONS: [&str; 2] = ["rtsz", "rtsx"];

/// The `royal` importer.
#[derive(Debug, Clone, Copy, Default)]
pub struct RoyalImporter;

impl Importer for RoyalImporter {
    fn id(&self) -> &'static str {
        IMPORTER_ID
    }

    fn display_name(&self) -> &'static str {
        "Royal TS/TSX"
    }

    /// A `.rtsz` or `.rtsx` file in a known container, or any XML file with
    /// the Royal root element in its first bytes.
    fn detect(&self, path: &Path, head: &[u8]) -> bool {
        let royal_extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                EXTENSIONS
                    .iter()
                    .any(|known| extension.eq_ignore_ascii_case(known))
            });
        match sniff::container(head) {
            Some(sniff::Container::Xml) => royal_extension || sniff::mentions_root_element(head),
            Some(_) => royal_extension,
            None => false,
        }
    }

    fn import(
        &self,
        input: &[u8],
        opts: &ImportOptions,
        prompter: &mut dyn SecretPrompter,
    ) -> Result<ImportOutcome, ImportError> {
        let text = sniff::decode(input)?;
        let objects = xml::parse_document(&text)?;
        let tree = tree::build(&objects, opts.include_trash)?;
        map::map_document(&objects, tree, opts, prompter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROYAL_XML: &[u8] = b"\xEF\xBB\xBF<?xml version=\"1.0\"?><RTSZDocument>";

    #[test]
    fn detect_accepts_royal_extensions_in_any_case() {
        assert!(RoyalImporter.detect(Path::new("doc.rtsz"), b"<x/>"));
        assert!(RoyalImporter.detect(Path::new("doc.RTSX"), b"\x1F\x8B\x08"));
        assert!(RoyalImporter.detect(Path::new("doc.rtsz"), b"PK\x03\x04"));
    }

    #[test]
    fn detect_accepts_royal_xml_with_another_extension() {
        assert!(RoyalImporter.detect(Path::new("doc.royal.xml"), ROYAL_XML));
    }

    #[test]
    fn detect_rejects_other_files() {
        assert!(!RoyalImporter.detect(Path::new("page.xml"), b"<html></html>"));
        assert!(!RoyalImporter.detect(Path::new("archive.gz"), b"\x1F\x8B\x08"));
        assert!(!RoyalImporter.detect(Path::new("doc.rtsz"), b"%PDF-1.7"));
    }
}
