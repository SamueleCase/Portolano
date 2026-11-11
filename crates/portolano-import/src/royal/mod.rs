//! Importer of Royal TS/TSX documents (`.rtsz`, `.rtsx`), SPEC §8.3 and
//! `docs/royal-format.md`.

// Stub (WP0): the parameters are used once WP5 implements the bodies.
#![expect(unused_variables)]

mod defaults;
mod fields;
mod map;
mod sniff;
mod xml;

use std::path::Path;

use crate::{ImportError, ImportOptions, ImportOutcome, Importer, SecretPrompter};

/// The `royal` importer.
#[derive(Debug, Clone, Copy, Default)]
pub struct RoyalImporter;

impl Importer for RoyalImporter {
    fn id(&self) -> &'static str {
        todo!()
    }

    fn display_name(&self) -> &'static str {
        todo!()
    }

    fn detect(&self, path: &Path, head: &[u8]) -> bool {
        todo!()
    }

    fn import(
        &self,
        input: &[u8],
        opts: &ImportOptions,
        prompter: &mut dyn SecretPrompter,
    ) -> Result<ImportOutcome, ImportError> {
        todo!()
    }
}
