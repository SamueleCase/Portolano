//! Import report (SPEC §8.2). It holds names, paths and reasons only: never
//! secrets or ciphertext.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::Serialize;

/// Summary of an import, printed as a table and saved as JSON by
/// `import --report`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
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

impl ImportReport {
    /// An empty report for an import starting at `imported_at`.
    pub(crate) fn new(importer: &str, source_file: &str, imported_at: DateTime<Utc>) -> Self {
        Self {
            importer: importer.to_owned(),
            source_file: source_file.to_owned(),
            document_name: String::new(),
            imported_at,
            counts: BTreeMap::new(),
            items: Vec::new(),
            unmapped_fields: BTreeMap::new(),
            warnings: Vec::new(),
        }
    }

    /// Counts the outcome of one source object; only items that were not
    /// fully imported are kept in [`ImportReport::items`] (SPEC §8.2).
    pub(crate) fn record(&mut self, item: ReportItem) {
        let counts = self.counts.entry(item.source_type.clone()).or_default();
        counts.source += 1;
        match item.status {
            ItemStatus::Imported => counts.imported += 1,
            ItemStatus::Partial => counts.partial += 1,
            ItemStatus::Unsupported => counts.unsupported += 1,
            ItemStatus::Skipped => counts.skipped += 1,
        }
        if item.status != ItemStatus::Imported {
            self.items.push(item);
        }
    }

    /// Counts one occurrence of a source field kept without a mapping.
    pub(crate) fn count_unmapped(&mut self, source_type: &str, field: &str) {
        *self
            .unmapped_fields
            .entry(source_type.to_owned())
            .or_default()
            .entry(field.to_owned())
            .or_default() += 1;
    }
}

/// Counts of one source type.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
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
#[cfg_attr(feature = "specta", derive(specta::Type))]
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
#[cfg_attr(feature = "specta", derive(specta::Type))]
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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn item(source_type: &str, status: ItemStatus) -> ReportItem {
        ReportItem {
            source_type: source_type.to_owned(),
            source_id: "00000000-0000-4000-8000-000000000001".to_owned(),
            path: "Royal/web01".to_owned(),
            status,
            reasons: Vec::new(),
        }
    }

    #[test]
    fn record_counts_every_status_per_source_type() {
        let mut report = ImportReport::new("royal", "test.rtsz", Utc::now());

        report.record(item("RoyalSSHConnection", ItemStatus::Imported));
        report.record(item("RoyalSSHConnection", ItemStatus::Partial));
        report.record(item("RoyalSSHConnection", ItemStatus::Unsupported));
        report.record(item("RoyalFolder", ItemStatus::Skipped));

        let ssh = TypeCounts {
            source: 3,
            imported: 1,
            partial: 1,
            unsupported: 1,
            skipped: 0,
        };
        let folder = TypeCounts {
            source: 1,
            skipped: 1,
            ..TypeCounts::default()
        };
        assert_eq!(report.counts["RoyalSSHConnection"], ssh);
        assert_eq!(report.counts["RoyalFolder"], folder);
    }

    #[test]
    fn record_keeps_only_items_not_fully_imported() {
        let mut report = ImportReport::new("royal", "test.rtsz", Utc::now());

        report.record(item("RoyalFolder", ItemStatus::Imported));
        report.record(item("RoyalFolder", ItemStatus::Partial));

        let statuses: Vec<ItemStatus> = report.items.iter().map(|i| i.status).collect();
        assert_eq!(statuses, vec![ItemStatus::Partial]);
    }

    #[test]
    fn count_unmapped_sums_occurrences_per_type_and_field() {
        let mut report = ImportReport::new("royal", "test.rtsz", Utc::now());

        report.count_unmapped("RoyalFolder", "IsExpanded");
        report.count_unmapped("RoyalFolder", "IsExpanded");
        report.count_unmapped("RoyalSSHConnection", "IsExpanded");

        assert_eq!(report.unmapped_fields["RoyalFolder"]["IsExpanded"], 2);
        assert_eq!(
            report.unmapped_fields["RoyalSSHConnection"]["IsExpanded"],
            1
        );
    }
}
