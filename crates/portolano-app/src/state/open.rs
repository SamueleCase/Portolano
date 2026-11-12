//! The document open in the app.

use std::path::{Path, PathBuf};

use portolano_core::{Document, Store};

/// A document file with its store and its content: loading, changing and
/// saving it happen under the one lock of [`super::AppState`].
pub struct OpenDocument {
    path: PathBuf,
    store: Store,
    document: Document,
}

impl OpenDocument {
    /// The document `document` read from `path` through `store`.
    pub fn new(path: PathBuf, store: Store, document: Document) -> Self {
        Self {
            path,
            store,
            document,
        }
    }

    /// The document file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The store of the file.
    pub fn store(&self) -> &Store {
        &self.store
    }

    /// The content.
    pub fn document(&self) -> &Document {
        &self.document
    }

    /// The store and the content, to change it with an operation of
    /// `portolano-ops`, which saves it.
    pub fn parts(&mut self) -> (&Store, &mut Document) {
        (&self.store, &mut self.document)
    }

    /// Replaces the content with the one read from the file again.
    pub fn reload(&mut self) -> portolano_core::Result<()> {
        self.document = self.store.load()?;
        Ok(())
    }
}
