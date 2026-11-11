//! Creating and opening the document file (SPEC §6.2, §10.2 `init`).

use std::io;
use std::path::Path;

use portolano_core::{Document, Store};

use crate::{OpsError, Result};

/// Creates a new empty document named `name` at `path`.
///
/// With `force` an existing file is replaced; the store keeps its previous
/// content in `<file>.bak`, and the vault is not touched, so the secrets of
/// the replaced document stay in it (DEC-022.10).
///
/// # Errors
/// [`portolano_core::Error::AlreadyExists`] without `force`, and the errors
/// of [`Store::create`], [`Store::open`] and [`Store::save`].
pub fn init_document(path: &Path, name: &str, force: bool) -> Result<(Store, Document)> {
    if force && path.try_exists().map_err(io_error(path))? {
        let store = Store::open(path)?;
        let mut doc = Document::new(name);
        store.save(&mut doc)?;
        return Ok((store, doc));
    }
    Ok(Store::create(path, name)?)
}

/// Opens and loads the document at `path`.
///
/// # Errors
/// [`OpsError::DocumentNotFound`] if the file does not exist, and the errors
/// of [`Store::open`] and [`Store::load`].
pub fn open_document(path: &Path) -> Result<(Store, Document)> {
    let store = match Store::open(path) {
        Ok(store) => store,
        Err(portolano_core::Error::Io { source, .. })
            if source.kind() == io::ErrorKind::NotFound =>
        {
            return Err(OpsError::DocumentNotFound {
                path: path.to_owned(),
            });
        }
        Err(error) => return Err(error.into()),
    };
    let doc = store.load()?;
    Ok((store, doc))
}

fn io_error(path: &Path) -> impl FnOnce(io::Error) -> OpsError + '_ {
    move |source| {
        OpsError::Core(portolano_core::Error::Io {
            path: path.to_owned(),
            source,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn init_creates_an_empty_document_with_the_given_name() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.json");
        init_document(&path, "Infra", false).unwrap();
        let (_, doc) = open_document(&path).unwrap();
        assert_eq!(doc.name, "Infra");
        assert!(doc.nodes.is_empty());
    }

    #[test]
    fn init_refuses_an_existing_file_without_force() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.json");
        init_document(&path, "Infra", false).unwrap();
        let result = init_document(&path, "Altro", false);
        assert!(matches!(
            result,
            Err(OpsError::Core(portolano_core::Error::AlreadyExists { .. }))
        ));
    }

    #[test]
    fn init_with_force_replaces_the_document_and_keeps_a_backup() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.json");
        init_document(&path, "Infra", false).unwrap();
        init_document(&path, "Nuovo", true).unwrap();
        let (_, doc) = open_document(&path).unwrap();
        assert_eq!(doc.name, "Nuovo");
        let backup = fs::read_to_string(dir.path().join("doc.json.bak")).unwrap();
        assert!(backup.contains("\"Infra\""));
    }

    #[test]
    fn open_of_a_missing_file_is_document_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let result = open_document(&dir.path().join("missing.json"));
        assert!(matches!(result, Err(OpsError::DocumentNotFound { .. })));
    }
}
