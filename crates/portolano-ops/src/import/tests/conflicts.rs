use portolano_core::vault::MemoryVault;
use portolano_import::NonInteractive;
use pretty_assertions::assert_eq;

use super::*;

#[test]
fn second_import_with_fail_lists_the_conflicts() {
    let mut stored = stored(vec![]);
    let vault = MemoryVault::default();
    let first = run(
        &mut stored,
        &vault,
        request(MINIMAL, ConflictPolicy::Fail),
        &mut NonInteractive,
    )
    .unwrap();
    let result = run(
        &mut stored,
        &vault,
        request(MINIMAL, ConflictPolicy::Fail),
        &mut NonInteractive,
    );
    assert!(
        matches!(result, Err(OpsError::ImportConflicts { conflicts }) if conflicts.len() == first.inserted)
    );
}

#[test]
fn new_ids_imports_a_second_copy_with_internal_references_remapped() {
    let mut stored = stored(vec![]);
    let vault = MemoryVault::default();
    let first = run(
        &mut stored,
        &vault,
        request(WORKSPACE, ConflictPolicy::Fail),
        &mut NonInteractive,
    )
    .unwrap();
    let before = ids(&stored.doc);
    let second = run(
        &mut stored,
        &vault,
        request(WORKSPACE, ConflictPolicy::NewIds),
        &mut NonInteractive,
    )
    .unwrap();
    let copy = stored.doc.find(second.root).unwrap();
    assert!(subtree_ids(copy).is_disjoint(&before));
    assert_eq!(second.inserted, first.inserted);
    let copy_ids = subtree_ids(copy);
    for (_, node) in stored
        .doc
        .walk()
        .filter(|(_, node)| copy_ids.contains(&node.id))
    {
        if let Some(portolano_core::Connection {
            credential: portolano_core::CredentialSpec::ById { id },
            ..
        }) = node.as_connection()
        {
            assert!(
                copy_ids.contains(id) || !before.contains(id),
                "reference left on the first copy"
            );
        }
    }
    let dangling = stored
        .doc
        .validate()
        .into_iter()
        .filter(|issue| issue.code == "dangling_ref")
        .count();
    let original_dangling = {
        let mut single = stored.doc.clone();
        single.remove(second.root).unwrap();
        single
            .validate()
            .into_iter()
            .filter(|issue| issue.code == "dangling_ref")
            .count()
    };
    assert_eq!(dangling, original_dangling * 2);
}

#[test]
fn new_ids_keep_the_source_ids() {
    let mut stored = stored(vec![]);
    let result = run(
        &mut stored,
        &MemoryVault::default(),
        request(MINIMAL, ConflictPolicy::NewIds),
        &mut NonInteractive,
    )
    .unwrap();
    let root = stored.doc.find(result.root).unwrap();
    let source_id = root
        .source
        .as_ref()
        .and_then(|source| source.source_id.clone())
        .unwrap();
    assert_ne!(source_id, result.root.to_string());
}

#[test]
fn update_replaces_nodes_in_place_and_keeps_their_position() {
    let mut stored = stored(vec![folder("Archivio", vec![])]);
    let vault = MemoryVault::default();
    let first = run(
        &mut stored,
        &vault,
        request(MINIMAL, ConflictPolicy::Fail),
        &mut NonInteractive,
    )
    .unwrap();
    let archive = id_of(&stored.doc, "/Archivio");
    crate::moves::move_node(
        &stored.store,
        &mut stored.doc,
        first.root,
        Some(archive),
        crate::moves::Placement::End,
    )
    .unwrap();
    let original_name = stored.doc.find(first.root).unwrap().name.clone();
    stored.doc.find_mut(first.root).unwrap().name = "rinominata".to_owned();
    let mut renamed = stored.doc.clone();
    stored.store.save(&mut renamed).unwrap();
    stored.doc = renamed;
    let result = run(
        &mut stored,
        &vault,
        request(MINIMAL, ConflictPolicy::Update),
        &mut NonInteractive,
    )
    .unwrap();
    assert_eq!((result.inserted, result.updated), (0, first.inserted));
    assert_eq!(stored.doc.parent_of(first.root), Some(Some(archive)));
    assert_eq!(stored.doc.find(first.root).unwrap().name, original_name);
    assert_eq!(result.warnings.len(), 1);
}

#[test]
fn update_keeps_secrets_the_import_does_not_bring() {
    let mut stored = stored(vec![]);
    let vault = MemoryVault::default();
    run(
        &mut stored,
        &vault,
        request(WORKSPACE, ConflictPolicy::Fail),
        &mut Typist,
    )
    .unwrap();
    let typed = document_secret_refs(&stored.doc);
    let result = run(
        &mut stored,
        &vault,
        request(WORKSPACE, ConflictPolicy::Update),
        &mut NonInteractive,
    )
    .unwrap();
    assert!(result.cleanup_failures.is_empty());
    assert_eq!(document_secret_refs(&stored.doc), typed);
    assert!(
        typed
            .iter()
            .all(|reference| vault.contains(reference).unwrap())
    );
}

#[test]
fn update_with_new_secrets_deletes_the_replaced_ones() {
    let mut stored = stored(vec![]);
    let vault = MemoryVault::default();
    run(
        &mut stored,
        &vault,
        request(WORKSPACE, ConflictPolicy::Fail),
        &mut Typist,
    )
    .unwrap();
    let old = document_secret_refs(&stored.doc);
    run(
        &mut stored,
        &vault,
        request(WORKSPACE, ConflictPolicy::Update),
        &mut Typist,
    )
    .unwrap();
    assert!(old.is_disjoint(&document_secret_refs(&stored.doc)));
    assert!(
        old.iter()
            .all(|reference| !vault.contains(reference).unwrap())
    );
}
