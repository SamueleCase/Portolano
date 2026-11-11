use std::collections::HashSet;

use portolano_core::Secret;
use portolano_core::vault::MemoryVault;
use portolano_import::{ImportError, NonInteractive, SecretRequest};
use pretty_assertions::assert_eq;

pub(super) use super::*;
pub(super) use crate::testutil::*;
pub(super) use crate::tree_util::document_secret_refs;

pub(super) const MINIMAL: &[u8] =
    include_bytes!("../../../../portolano-import/tests/fixtures/minimal.royal.xml");
pub(super) const WORKSPACE: &[u8] =
    include_bytes!("../../../../portolano-import/tests/fixtures/workspace.royal.xml");
pub(super) const FULL: &[u8] =
    include_bytes!("../../../../portolano-import/tests/fixtures/full-synthetic.royal.xml");

/// Answers every request with the sentinel.
pub(super) struct Typist;

impl SecretPrompter for Typist {
    fn ask(&mut self, _req: &SecretRequest) -> Result<Option<Secret>, ImportError> {
        Ok(Some(secret(SENTINEL)))
    }
}

/// Aborts at the first request.
struct Quitter;

impl SecretPrompter for Quitter {
    fn ask(&mut self, _req: &SecretRequest) -> Result<Option<Secret>, ImportError> {
        Err(ImportError::Cancelled)
    }
}

pub(super) fn request(input: &[u8], on_conflict: ConflictPolicy) -> ImportRequest<'_> {
    ImportRequest {
        input,
        file_name: "fixture.rtsx".to_owned(),
        format: None,
        into: None,
        on_conflict,
        include_trash: false,
        dry_run: false,
    }
}

pub(super) fn run(
    stored: &mut Stored,
    vault: &dyn Vault,
    request: ImportRequest<'_>,
    prompter: &mut dyn SecretPrompter,
) -> Result<ImportResult> {
    import(
        &stored.store,
        &mut stored.doc,
        vault,
        &Registry::with_defaults(),
        request,
        prompter,
    )
}

pub(super) fn ids(doc: &Document) -> HashSet<Uuid> {
    doc.walk().map(|(_, node)| node.id).collect()
}

mod conflicts;

#[test]
fn import_inserts_the_imported_folder_at_the_root_and_saves() {
    let mut stored = stored(vec![]);
    let result = run(
        &mut stored,
        &MemoryVault::default(),
        request(MINIMAL, ConflictPolicy::Fail),
        &mut NonInteractive,
    )
    .unwrap();
    let saved = stored.reloaded();
    assert_eq!(saved.nodes.len(), 1);
    assert_eq!(saved.nodes[0].id, result.root);
    assert_eq!(ids(&saved).len(), result.inserted);
    assert_eq!((result.updated, result.secrets_written), (0, 0));
}

#[test]
fn typed_secrets_go_to_the_vault_only() {
    let mut stored = stored(vec![]);
    let vault = MemoryVault::default();
    let result = run(
        &mut stored,
        &vault,
        request(WORKSPACE, ConflictPolicy::Fail),
        &mut Typist,
    )
    .unwrap();
    assert!(result.secrets_written > 0);
    let refs = document_secret_refs(&stored.doc);
    assert_eq!(refs.len(), result.secrets_written);
    for reference in &refs {
        assert_eq!(vault.get(reference).unwrap().unwrap().expose(), SENTINEL);
    }
    assert!(!stored.file_text().contains(SENTINEL));
}

#[test]
fn failed_save_removes_the_typed_secrets() {
    let mut stored = stored(vec![]);
    let vault = RecordingVault::default();
    stored.modify_externally();
    let result = run(
        &mut stored,
        &vault,
        request(WORKSPACE, ConflictPolicy::Fail),
        &mut Typist,
    );
    assert!(result.is_err());
    assert!(stored.doc.nodes.is_empty());
    let (written, deleted) = vault.history();
    assert!(!written.is_empty());
    assert_eq!(written, deleted);
}

#[test]
fn cancelled_prompt_writes_nothing() {
    let mut stored = stored(vec![]);
    let before = stored.file_text();
    let result = run(
        &mut stored,
        &MemoryVault::default(),
        request(WORKSPACE, ConflictPolicy::Fail),
        &mut Quitter,
    );
    assert!(matches!(
        result,
        Err(OpsError::Import(ImportError::Cancelled))
    ));
    assert_eq!(stored.file_text(), before);
}

#[test]
fn dry_run_writes_nothing() {
    let mut stored = stored(vec![]);
    let vault = RecordingVault::default();
    let before = stored.file_text();
    let mut dry = request(FULL, ConflictPolicy::Fail);
    dry.dry_run = true;
    let result = run(&mut stored, &vault, dry, &mut Typist).unwrap();
    assert!(result.dry_run);
    assert!(result.inserted > 0);
    assert!(result.secrets_written > 0 && result.icons_stored > 0);
    assert_eq!(stored.file_text(), before);
    assert!(stored.doc.nodes.is_empty());
    assert_eq!(vault.history(), (Vec::new(), Vec::new()));
    assert!(!stored.dir.path().join("doc.icons").exists());
}

#[test]
fn custom_icons_are_stored() {
    let mut stored = stored(vec![]);
    let result = run(
        &mut stored,
        &MemoryVault::default(),
        request(FULL, ConflictPolicy::Fail),
        &mut NonInteractive,
    )
    .unwrap();
    assert!(result.icons_stored > 0);
    let icons = stored.store.icons();
    let custom: Vec<_> = stored
        .doc
        .walk()
        .filter_map(|(_, node)| node.icon.clone())
        .filter(|icon| matches!(icon, Icon::Custom { .. }))
        .collect();
    assert!(!custom.is_empty());
    assert!(
        custom
            .iter()
            .all(|icon| icons.path_for(icon).unwrap().exists())
    );
}

#[test]
fn import_goes_into_the_given_folder() {
    let mut stored = stored(vec![folder("Import", vec![])]);
    let mut into = request(MINIMAL, ConflictPolicy::Fail);
    into.into = Some(id_of(&stored.doc, "/Import"));
    let result = run(
        &mut stored,
        &MemoryVault::default(),
        into,
        &mut NonInteractive,
    )
    .unwrap();
    assert_eq!(
        stored.doc.parent_of(result.root),
        Some(Some(id_of(&stored.doc, "/Import")))
    );
}

#[test]
fn destination_must_be_a_folder() {
    let mut stored = stored(vec![connection("web01", "ssh")]);
    let mut into = request(MINIMAL, ConflictPolicy::Fail);
    into.into = Some(id_of(&stored.doc, "/web01"));
    let result = run(
        &mut stored,
        &MemoryVault::default(),
        into,
        &mut NonInteractive,
    );
    assert!(matches!(
        result,
        Err(OpsError::Core(portolano_core::Error::NotAFolder { .. }))
    ));
}

#[test]
fn import_too_deep_for_the_destination_is_explained() {
    let mut deepest = folder("livello", vec![]);
    for _ in 1..portolano_core::MAX_DEPTH {
        deepest = folder("livello", vec![deepest]);
    }
    let mut stored = stored(vec![deepest]);
    let bottom = stored.doc.walk().last().unwrap().1.id;
    let mut into = request(MINIMAL, ConflictPolicy::Fail);
    into.into = Some(bottom);
    let result = run(
        &mut stored,
        &MemoryVault::default(),
        into,
        &mut NonInteractive,
    );
    assert!(matches!(
        result,
        Err(OpsError::ImportTooDeep {
            max: portolano_core::MAX_DEPTH,
            ..
        })
    ));
}

#[test]
fn unknown_format_is_refused() {
    let mut stored = stored(vec![]);
    let mut forced = request(MINIMAL, ConflictPolicy::Fail);
    forced.format = Some("mremoteng".to_owned());
    let result = run(
        &mut stored,
        &MemoryVault::default(),
        forced,
        &mut NonInteractive,
    );
    assert!(matches!(result, Err(OpsError::UnknownImportFormat { .. })));
}

#[test]
fn unrecognized_file_is_refused() {
    let mut stored = stored(vec![]);
    let mut text = request(b"just some notes", ConflictPolicy::Fail);
    text.file_name = "notes.txt".to_owned();
    let result = run(
        &mut stored,
        &MemoryVault::default(),
        text,
        &mut NonInteractive,
    );
    assert!(matches!(
        result,
        Err(OpsError::UnrecognizedImportFile { .. })
    ));
}
