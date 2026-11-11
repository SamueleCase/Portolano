//! Import report (SPEC §8.2). It holds names, paths and reasons only: never
//! secrets or ciphertext.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::Serialize;

/// Summary of an import, printed as a table and saved as JSON by
/// `import --report`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ImportReport {
    /// Id of the importer, such as `"royal"`.
    pub importer: String,
    /// Name of the source file, without its directory.
    pub source_file: String,
    /// Name of the imported document.
    pub document_name: String,
    /// Time of the import.
    pub imported_at: DateTime<Utc>,
    /// Counts per source type.
    pub counts: BTreeMap<String, TypeCounts>,
    /// Items that were not fully imported (all items with `--verbose`).
    pub items: Vec<ReportItem>,
    /// Source fields without a mapping: type → field → occurrences.
    pub unmapped_fields: BTreeMap<String, BTreeMap<String, usize>>,
    /// Document-level warnings.
    pub warnings: Vec<String>,
}

/// Counts of one source type.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct TypeCounts {
    /// Objects of this type in the source.
    pub source: usize,
    /// Imported completely.
    pub imported: usize,
    /// Imported with missing data.
    pub partial: usize,
    /// Kept as unsupported nodes.
    pub unsupported: usize,
    /// Not imported (for example in the trash).
    pub skipped: usize,
}

/// Outcome of one source object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReportItem {
    /// Type in the source, such as `"RoyalSSHConnection"`.
    pub source_type: String,
    /// Id in the source.
    pub source_id: String,
    /// Path of the object in the imported tree.
    pub path: String,
    /// How it was imported.
    pub status: ItemStatus,
    /// Why it is not [`ItemStatus::Imported`], in Italian.
    pub reasons: Vec<String>,
}

/// How a source object was imported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemStatus {
    /// Imported completely.
    Imported,
    /// Imported with missing data, such as a secret that was not provided.
    Partial,
    /// Kept as an unsupported node.
    Unsupported,
    /// Not imported.
    Skipped,
}
