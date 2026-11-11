//! Document persistence: exclusive lock, versioned load and atomic save
//! (SPEC §6.2).

// Stub (WP0): the parameters are used once WP2 implements the bodies.
#![expect(unused_variables)]

use std::path::PathBuf;

use crate::error::Result;
use crate::icons::IconStore;
use crate::model::Document;

/// An open document file, locked exclusively (`<file>.lock`) for as long as
/// the value lives.
#[derive(Debug)]
// Stub (WP0): the field is read once WP2 implements the methods.
#[expect(dead_code)]
pub struct Store {
    path: PathBuf,
}

impl Store {
    /// Opens an existing document file and acquires its lock.
    ///
    /// # Errors
    /// [`Error::Locked`](crate::Error::Locked) if another process holds the
    /// lock, [`Error::Io`](crate::Error::Io) on filesystem errors.
    pub fn open(path: impl Into<PathBuf>) -> Result<Store> {
        todo!()
    }

    /// Creates a new document file containing an empty document named
    /// `name`, acquires its lock and returns both.
    ///
    /// # Errors
    /// [`Error::AlreadyExists`](crate::Error::AlreadyExists) if the file
    /// exists, plus the errors of [`Store::open`] and [`Store::save`].
    pub fn create(path: impl Into<PathBuf>, name: &str) -> Result<(Store, Document)> {
        todo!()
    }

    /// Reads the document, migrating it from an older schema version if
    /// needed.
    ///
    /// # Errors
    /// [`Error::NotADocument`](crate::Error::NotADocument),
    /// [`Error::UnsupportedSchema`](crate::Error::UnsupportedSchema),
    /// migration errors, [`Error::InvalidDocument`](crate::Error::InvalidDocument),
    /// [`Error::DuplicateId`](crate::Error::DuplicateId),
    /// [`Error::Io`](crate::Error::Io).
    pub fn load(&self) -> Result<Document> {
        todo!()
    }

    /// Writes the document atomically (temporary file, `fsync`, rename) after
    /// copying the previous file to `<file>.bak`, and updates
    /// `doc.modified`. The output is deterministic: 2-space pretty JSON with
    /// a final newline.
    ///
    /// # Errors
    /// [`Error::Serialize`](crate::Error::Serialize),
    /// [`Error::Io`](crate::Error::Io).
    pub fn save(&self, doc: &mut Document) -> Result<()> {
        todo!()
    }

    /// The icon store next to this document (`<dir>/<stem>.icons/`).
    pub fn icons(&self) -> IconStore {
        todo!()
    }
}
