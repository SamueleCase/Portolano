//! Mapping of Royal objects to Portolano nodes (SPEC §8.3.3–5).
//!
//! The tree is walked in display order, so the user is asked for secrets in
//! the order the nodes appear. Every source object, skipped ones included,
//! is counted exactly once in the report.

mod connection;
mod context;
mod credential;
mod icon;
mod notes;
mod presentation;
mod references;

use std::path::Path;

use chrono::{DateTime, Utc};
use portolano_core::{Folder, Node, NodeKind, SourceInfo, Unsupported};
use serde_json::Value;

use self::context::{ObjectContext, SecretCollector};
use self::references::References;
use super::fields::{Fields, is_secret_value, parse_date};
use super::tree::{Tree, TreeNode};
use super::xml::RawObject;
use super::{IMPORTER_ID, UNNAMED, types};
use crate::{
    ImportError, ImportOptions, ImportOutcome, ImportReport, ItemStatus, PendingIcon, ReportItem,
    SecretPrompter,
};

/// Name of the folder that holds the trash with `include_trash`.
const TRASH_FOLDER_NAME: &str = "Cestino (Royal)";
const SOURCE_SYSTEM: &str = "royal";
/// Prefix of the `extra` keys of Royal fields (SPEC §3).
const EXTRA_PREFIX: &str = "royal.";

/// Maps the whole document; the `RoyalDocument` becomes the root folder.
///
/// # Errors
/// The error of the prompter, such as [`ImportError::Cancelled`].
pub(super) fn map_document(
    objects: &[RawObject],
    tree: Tree,
    opts: &ImportOptions,
    prompter: &mut dyn SecretPrompter,
) -> Result<ImportOutcome, ImportError> {
    let imported_at = Utc::now();
    let references = References::collect(objects, &tree, opts.include_trash);
    let mut report = ImportReport::new(IMPORTER_ID, &opts.source_file_name, imported_at);
    report.warnings = tree.warnings;
    let mut mapper = Mapper {
        objects,
        ids: &tree.ids,
        references,
        opts,
        imported_at,
        secrets: SecretCollector::new(prompter),
        icons: Vec::new(),
        report,
    };
    let root = mapper.map_subtree(&tree.root, "")?;
    mapper.report.document_name = root.name.clone();
    Ok(ImportOutcome {
        root,
        secrets: mapper.secrets.into_collected(),
        icons: mapper.icons,
        report: mapper.report,
    })
}

/// What an object becomes.
enum Class {
    Supported(Kind),
    /// Kept verbatim, for the given reason.
    Unsupported(String),
}

/// The node kinds Royal objects map to.
enum Kind {
    Folder,
    Ssh,
    Rdp,
    Credential,
}

struct Mapper<'a, 'p> {
    objects: &'a [RawObject],
    ids: &'a [uuid::Uuid],
    references: References,
    opts: &'a ImportOptions,
    imported_at: DateTime<Utc>,
    secrets: SecretCollector<'p>,
    icons: Vec<PendingIcon>,
    report: ImportReport,
}

impl Mapper<'_, '_> {
    fn map_subtree(&mut self, tree: &TreeNode, parent_path: &str) -> Result<Node, ImportError> {
        let (mut node, path) = self.map_object(tree.object, parent_path)?;
        // The tree only gives children to containers, which all map to folders.
        if let Some(folder) = node.as_folder_mut() {
            for child in &tree.children {
                if self.is_skipped_trash(child) {
                    self.skip_subtree(child, &path);
                } else {
                    folder.children.push(self.map_subtree(child, &path)?);
                }
            }
        }
        Ok(node)
    }

    fn is_skipped_trash(&self, tree: &TreeNode) -> bool {
        references::is_skipped_trash(&self.objects[tree.object], self.opts.include_trash)
    }

    /// Records the trash and everything in it as skipped.
    fn skip_subtree(&mut self, tree: &TreeNode, parent_path: &str) {
        let object = &self.objects[tree.object];
        let path = join_path(parent_path, object.value("Name").unwrap_or(UNNAMED));
        let reason = if object.type_name == types::TRASH {
            "cestino di Royal non importato (usa l'opzione per includerlo)"
        } else {
            "nel cestino di Royal"
        };
        self.record(
            tree.object,
            &path,
            ItemStatus::Skipped,
            vec![reason.to_owned()],
        );
        for child in &tree.children {
            self.skip_subtree(child, &path);
        }
    }

    fn map_object(
        &mut self,
        index: usize,
        parent_path: &str,
    ) -> Result<(Node, String), ImportError> {
        let objects = self.objects;
        let object = &objects[index];
        let mut fields = Fields::new(object);
        for structural in ["ID", "ParentID", "PositionNr"] {
            fields.take_raw(structural);
        }
        let royal_name = fields.take("Name").unwrap_or(UNNAMED);
        let name = if object.type_name == types::TRASH {
            TRASH_FOLDER_NAME
        } else {
            royal_name
        };
        let path = join_path(parent_path, name);
        let mut ctx = ObjectContext::new(fields, path.clone());
        let created = ctx.fields.take_with("Created", parse_date);
        let modified = ctx.fields.take_with("Modified", parse_date);
        // The constructor fills the common fields; the kind is set below.
        let mut node = Node::folder(name);
        node.id = self.ids[index];
        node.created = created.unwrap_or(self.imported_at);
        node.modified = modified.unwrap_or(self.imported_at);
        node.source = Some(self.source_info(index));

        let status = match classify(object, &ctx.fields) {
            Class::Unsupported(reason) => {
                node.description = ctx.fields.take("Description").map(str::to_owned);
                node.kind = unsupported(object, reason, &mut ctx);
                ItemStatus::Unsupported
            }
            Class::Supported(kind) => {
                node.kind = self.map_kind(kind, &mut ctx)?;
                presentation::apply(&mut ctx, &mut node, &mut self.secrets, &mut self.icons)?;
                if object.type_name == types::DOCUMENT {
                    self.take_document_fields(&mut ctx);
                }
                self.keep_unused_fields(&mut ctx, &object.type_name);
                node.extra = std::mem::take(&mut ctx.extra);
                if ctx.reasons.is_empty() {
                    ItemStatus::Imported
                } else {
                    ItemStatus::Partial
                }
            }
        };
        self.record(index, &path, status, ctx.reasons);
        Ok((node, path))
    }

    fn map_kind(
        &mut self,
        kind: Kind,
        ctx: &mut ObjectContext<'_>,
    ) -> Result<NodeKind, ImportError> {
        let references = &self.references;
        let secrets = &mut self.secrets;
        Ok(match kind {
            Kind::Folder => NodeKind::Folder(Folder {
                children: Vec::new(),
                credential: credential::spec(ctx, references, secrets)?,
            }),
            Kind::Ssh => NodeKind::Connection(connection::ssh(ctx, references, secrets)?),
            Kind::Rdp => NodeKind::Connection(connection::rdp(ctx, references, secrets)?),
            Kind::Credential => NodeKind::Credential(credential::credential(ctx, secrets)?),
        })
    }

    /// The document is protected by a password whose hash is useless to
    /// Portolano and sensitive: it is dropped (SPEC §8.3.3).
    fn take_document_fields(&mut self, ctx: &mut ObjectContext<'_>) {
        if ctx.fields.take("PasswordHash").is_some() {
            self.report
                .warnings
                .push("documento protetto da password".to_owned());
        }
    }

    /// Moves the fields without a mapping to `extra` and counts them;
    /// encrypted ones are dropped instead (D3).
    fn keep_unused_fields(&mut self, ctx: &mut ObjectContext<'_>, type_name: &str) {
        let unused: Vec<_> = ctx.fields.unused().collect();
        for (name, field) in unused {
            if is_secret_value(name, field.as_str()) {
                ctx.add_reason(format!("campo cifrato `{name}` non importato"));
            } else {
                ctx.extra.insert(
                    format!("{EXTRA_PREFIX}{name}"),
                    Value::String(field.as_str().to_owned()),
                );
                self.report.count_unmapped(type_name, name);
            }
        }
    }

    fn source_info(&self, index: usize) -> SourceInfo {
        let file = Path::new(&self.opts.source_file_name)
            .file_name()
            .and_then(|name| name.to_str())
            .map(str::to_owned);
        SourceInfo {
            system: SOURCE_SYSTEM.to_owned(),
            source_type: self.objects[index].type_name.clone(),
            source_id: Some(self.ids[index].to_string()),
            file,
            imported_at: self.imported_at,
        }
    }

    fn record(&mut self, index: usize, path: &str, status: ItemStatus, reasons: Vec<String>) {
        self.report.record(ReportItem {
            source_type: self.objects[index].type_name.clone(),
            source_id: self.ids[index].to_string(),
            path: path.to_owned(),
            status,
            reasons,
        });
    }
}

fn classify(object: &RawObject, fields: &Fields<'_>) -> Class {
    match object.type_name.as_str() {
        types::DOCUMENT | types::FOLDER | types::TRASH => Class::Supported(Kind::Folder),
        types::SSH_CONNECTION if fields.peek_flag("IsTelnetConnection") => {
            Class::Unsupported("connessione Telnet non supportata".to_owned())
        }
        types::SSH_CONNECTION if fields.peek_flag("IsSerialPortConnection") => {
            Class::Unsupported("connessione seriale non supportata".to_owned())
        }
        types::SSH_CONNECTION | types::SECURE_GATEWAY => Class::Supported(Kind::Ssh),
        types::RDS_CONNECTION => Class::Supported(Kind::Rdp),
        types::CREDENTIAL => Class::Supported(Kind::Credential),
        types::RDS_GATEWAY => Class::Unsupported(
            "gateway RDP riutilizzabile: host e porta sono copiati nelle connessioni che lo usano"
                .to_owned(),
        ),
        other => Class::Unsupported(format!("tipo Royal non supportato: {other}")),
    }
}

/// An unsupported object keeps its XML, without the encrypted fields (D3).
fn unsupported(object: &RawObject, reason: String, ctx: &mut ObjectContext<'_>) -> NodeKind {
    ctx.add_reason(reason);
    if object.has_secrets() {
        ctx.add_reason("campi cifrati rimossi dall'XML conservato");
    }
    NodeKind::Unsupported(Unsupported {
        source_type: object.type_name.clone(),
        raw: object.redacted_xml(),
    })
}

/// Appends a name to a path with the escapes of SPEC §4.2.
fn join_path(parent: &str, name: &str) -> String {
    let segment = name.replace('\\', "\\\\").replace('/', "\\/");
    if parent.is_empty() {
        segment
    } else {
        format!("{parent}/{segment}")
    }
}
