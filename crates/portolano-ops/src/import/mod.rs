//! Inserting an import into the document (SPEC §8.4).
//!
//! The importer returns a folder of nodes, the secrets typed by the user and
//! the custom icons; this module resolves the conflicts with the document,
//! then writes secrets, icons and document in this order, removing the
//! secrets again if the document cannot be saved.

mod answers;
mod merge;
pub(crate) mod remap;

use std::fmt;
use std::path::Path;

use portolano_core::{Document, Icon, Node, Secret, SecretRef, Store, Vault};
use portolano_import::{
    ImportOptions, ImportOutcome, ImportReport, Importer, PendingIcon, Registry, SecretPrompter,
};
use serde::Serialize;
use uuid::Uuid;

use crate::commit::{Change, CleanupFailure};
use crate::nodes::collect_unused_icons;
use crate::tree_util::{for_each_node_mut, location_path, node_path, subtree_ids};
use crate::{OpsError, Result};

pub use answers::{ImportPreview, SecretAnswer, import_requests, import_with_answers};

/// Bytes of the input given to [`Importer::detect`].
const DETECT_HEAD_BYTES: usize = 4096;

/// What to do when an id of the import already exists in the document.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ConflictPolicy {
    /// Refuse the import and list the conflicts.
    #[default]
    Fail,
    /// Replace each existing node in place (same parent and position),
    /// keeping its secrets where the import brings none.
    Update,
    /// Give every imported node a new id, remapping the references inside
    /// the import.
    NewIds,
}

/// An import to insert.
pub struct ImportRequest<'a> {
    /// The content of the file.
    pub input: &'a [u8],
    /// The file name, without directory: used to recognize the format and
    /// recorded in the nodes and the report.
    pub file_name: String,
    /// The importer id (`--format`); recognized from the file if `None`.
    pub format: Option<String>,
    /// The destination folder; `None` is the root.
    pub into: Option<Uuid>,
    /// What to do with ids already in the document.
    pub on_conflict: ConflictPolicy,
    /// Import the trash of the source too.
    pub include_trash: bool,
    /// Do everything in memory and write nothing.
    pub dry_run: bool,
}

/// Shows the length of the input, not its content: a foreign document can
/// hold encrypted secrets, which must not reach logs (D3).
impl fmt::Debug for ImportRequest<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ImportRequest")
            .field("input", &format_args!("{} byte", self.input.len()))
            .field("file_name", &self.file_name)
            .field("format", &self.format)
            .field("into", &self.into)
            .field("on_conflict", &self.on_conflict)
            .field("include_trash", &self.include_trash)
            .field("dry_run", &self.dry_run)
            .finish()
    }
}

/// The result of [`import`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ImportResult {
    /// The report of the importer.
    pub report: ImportReport,
    /// The imported folder.
    pub root: Uuid,
    /// Nodes added to the document.
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub inserted: usize,
    /// Existing nodes replaced (`ConflictPolicy::Update`).
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub updated: usize,
    /// Secrets written to the vault (to be written, with `dry_run`).
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub secrets_written: usize,
    /// Custom icons stored (to be stored, with `dry_run`).
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub icons_stored: usize,
    /// Notes about the insertion, in Italian.
    pub warnings: Vec<String>,
    /// Secrets no longer used that could not be deleted from the vault.
    pub cleanup_failures: Vec<CleanupFailure>,
    /// Whether nothing was written.
    pub dry_run: bool,
}

/// Imports `request.input` into the document.
///
/// # Errors
/// [`OpsError::UnknownImportFormat`], [`OpsError::UnrecognizedImportFile`],
/// importer and prompter errors, [`portolano_core::Error::NodeNotFound`] or
/// [`portolano_core::Error::NotAFolder`] for the destination,
/// [`OpsError::ImportConflicts`], [`OpsError::ImportKindChanged`],
/// [`OpsError::ImportTooDeep`], errors of the vault, the icon store and the
/// store. On error the document and the vault are unchanged; images already
/// stored are removed by a later cleanup of the unused icons.
pub fn import(
    store: &Store,
    doc: &mut Document,
    vault: &dyn Vault,
    importers: &Registry,
    request: ImportRequest<'_>,
    prompter: &mut dyn SecretPrompter,
) -> Result<ImportResult> {
    let (change, finish) = prepare(doc, importers, &request, prompter)?;
    commit(store, vault, doc, change, finish)
}

/// Runs the importer and places its tree in a copy of `doc`, writing
/// nothing.
fn prepare(
    doc: &Document,
    importers: &Registry,
    request: &ImportRequest<'_>,
    prompter: &mut dyn SecretPrompter,
) -> Result<(Change, Finish)> {
    let importer = select_importer(importers, request)?;
    if let Some(into) = request.into {
        doc.children(Some(into))?;
    }
    let options = ImportOptions {
        source_file_name: request.file_name.clone(),
        include_trash: request.include_trash,
    };
    let ImportOutcome {
        root,
        secrets,
        icons,
        report,
    } = importer.import(request.input, &options, prompter)?;
    let mut change = Change::from(doc);
    let mut warnings = Vec::new();
    let placed = place(
        &mut change.doc,
        request.into,
        request.on_conflict,
        root,
        &mut warnings,
    )?;
    let finish = Finish {
        report,
        root: placed.root,
        inserted: placed.inserted,
        updated: placed.updated,
        secrets,
        icons,
        warnings,
        dry_run: request.dry_run,
    };
    Ok((change, finish))
}

/// Where the imported tree went.
struct Placed {
    root: Uuid,
    inserted: usize,
    updated: usize,
}

/// Puts the imported tree in `doc` under `into`, resolving the ids that
/// already exist as `policy` says.
fn place(
    doc: &mut Document,
    into: Option<Uuid>,
    policy: ConflictPolicy,
    mut root: Node,
    warnings: &mut Vec<String>,
) -> Result<Placed> {
    let (inserted, updated) = match policy {
        ConflictPolicy::Fail => {
            refuse_conflicts(doc, &root)?;
            (0, 0)
        }
        ConflictPolicy::NewIds => {
            remap::regenerate_ids(&mut root)?;
            (0, 0)
        }
        ConflictPolicy::Update => {
            let root_id = root.id;
            let (inserted, updated) = merge::merge(doc, into, root, warnings)?;
            return Ok(Placed {
                root: root_id,
                inserted,
                updated,
            });
        }
    };
    let root_id = root.id;
    Ok(Placed {
        root: root_id,
        inserted: inserted + insert(doc, into, root)?,
        updated,
    })
}

/// The importer chosen by `--format`, or the one that recognizes the file.
fn select_importer<'r>(
    importers: &'r Registry,
    request: &ImportRequest<'_>,
) -> Result<&'r dyn Importer> {
    match &request.format {
        Some(format) => importers
            .get(format)
            .ok_or_else(|| OpsError::UnknownImportFormat {
                format: format.clone(),
                known: importers.ids().into_iter().map(str::to_owned).collect(),
            }),
        None => {
            let head = &request.input[..request.input.len().min(DETECT_HEAD_BYTES)];
            importers
                .detect(Path::new(&request.file_name), head)
                .ok_or_else(|| OpsError::UnrecognizedImportFile {
                    file: request.file_name.clone(),
                })
        }
    }
}

/// [`OpsError::ImportConflicts`] if an id of `root`'s subtree is in `doc`.
fn refuse_conflicts(doc: &Document, root: &Node) -> Result<()> {
    let imported = subtree_ids(root);
    let conflicts: Vec<(Uuid, String)> = doc
        .walk()
        .filter(|(_, node)| imported.contains(&node.id))
        .map(|(_, node)| (node.id, node_path(doc, node.id)))
        .collect();
    if conflicts.is_empty() {
        Ok(())
    } else {
        Err(OpsError::ImportConflicts { conflicts })
    }
}

/// Inserts `node` with its subtree at the end of `into`; returns the number
/// of nodes inserted.
pub(super) fn insert(doc: &mut Document, into: Option<Uuid>, node: Node) -> Result<usize> {
    let count = subtree_ids(&node).len();
    doc.insert(into, None, node).map_err(|error| match error {
        portolano_core::Error::TooDeep { max } => OpsError::ImportTooDeep {
            into: location_path(doc, into),
            max,
        },
        other => other.into(),
    })?;
    Ok(count)
}

/// Everything [`finish`] needs besides the change.
struct Finish {
    report: ImportReport,
    root: Uuid,
    inserted: usize,
    updated: usize,
    secrets: Vec<(SecretRef, Secret)>,
    icons: Vec<PendingIcon>,
    warnings: Vec<String>,
    dry_run: bool,
}

impl Finish {
    /// The result of the import before anything is written, with the
    /// secrets and the icons still to store.
    fn into_parts(self) -> (ImportResult, Vec<(SecretRef, Secret)>, Vec<PendingIcon>) {
        let result = ImportResult {
            report: self.report,
            root: self.root,
            inserted: self.inserted,
            updated: self.updated,
            secrets_written: self.secrets.len(),
            icons_stored: self.icons.len(),
            warnings: self.warnings,
            cleanup_failures: Vec::new(),
            dry_run: self.dry_run,
        };
        (result, self.secrets, self.icons)
    }
}

/// Writes secrets, icons and document, unless it is a dry run.
fn commit(
    store: &Store,
    vault: &dyn Vault,
    doc: &mut Document,
    mut change: Change,
    finish: Finish,
) -> Result<ImportResult> {
    let (mut result, secrets, icons) = finish.into_parts();
    if result.dry_run {
        return Ok(result);
    }
    change.new_secrets = secrets;
    let committed = change.commit(store, vault, doc, |doc| store_icons(store, doc, icons))?;
    // An update may have replaced custom icons of existing nodes.
    collect_unused_icons(store, doc);
    result.cleanup_failures = committed.cleanup_failures;
    Ok(result)
}

/// Stores the custom icons and points the nodes to the stored ones: the
/// store recognizes the type from the bytes, which may differ from the type
/// the importer declared.
fn store_icons(store: &Store, doc: &mut Document, icons: Vec<PendingIcon>) -> Result<()> {
    let icon_store = store.icons();
    for pending in icons {
        let stored = icon_store.add(&pending.bytes, &pending.mime)?;
        for_each_node_mut(&mut doc.nodes, &mut |node| {
            let is_placeholder = matches!(
                &node.icon,
                Some(Icon::Custom { sha256, .. }) if *sha256 == pending.placeholder_sha256
            );
            if is_placeholder {
                node.icon = Some(stored.clone());
            }
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
