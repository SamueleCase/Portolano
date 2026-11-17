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
    #[cfg_attr(feature = "specta", specta(type = BTreeMap<String, BTreeMap<String, u32>>))]
    pub unmapped_fields: BTreeMap<String, BTreeMap<String, usize>>,
    /// Document-level warnings.
    pub warnings: Vec<String>,
    /// Source objects per kind of node they become, or would become for the
    /// skipped ones: the "Per tipo" of the app (DEC-039.1).
    pub kinds: KindCounts,
    /// Objects in the trash of the source, the trash itself excluded, whether
    /// the trash was imported or not.
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub trash_items: usize,
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
            kinds: KindCounts::default(),
            trash_items: 0,
        }
    }

    /// Counts the outcome of one source object, which becomes a node of
    /// `kind`; only items that were not fully imported are kept in
    /// [`ImportReport::items`] (SPEC §8.2).
    pub(crate) fn record(&mut self, item: ReportItem, kind: NodeTarget<'_>) {
        self.kinds.count(kind);
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

/// The kind of node a source object becomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NodeTarget<'a> {
    Folder,
    /// A connection of the protocol with this id, such as `"ssh"`.
    Connection(&'a str),
    Credential,
    /// A node kept verbatim.
    Unsupported,
}

/// Source objects per kind of node.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct KindCounts {
    /// Folders.
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub folders: usize,
    /// Connections, by protocol id.
    #[cfg_attr(feature = "specta", specta(type = BTreeMap<String, u32>))]
    pub connections: BTreeMap<String, usize>,
    /// Credentials.
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub credentials: usize,
    /// Nodes kept as unsupported.
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub unsupported: usize,
}

impl KindCounts {
    fn count(&mut self, kind: NodeTarget<'_>) {
        match kind {
            NodeTarget::Folder => self.folders += 1,
            NodeTarget::Connection(protocol) => {
                *self.connections.entry(protocol.to_owned()).or_default() += 1;
            }
            NodeTarget::Credential => self.credentials += 1,
            NodeTarget::Unsupported => self.unsupported += 1,
        }
    }
}

/// Counts of one source type.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct TypeCounts {
    /// Objects of this type in the source.
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub source: usize,
    /// Imported completely.
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub imported: usize,
    /// Imported with missing data.
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub partial: usize,
    /// Kept as unsupported nodes.
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub unsupported: usize,
    /// Not imported (for example in the trash).
    #[cfg_attr(feature = "specta", specta(type = u32))]
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

    const SSH: NodeTarget<'static> = NodeTarget::Connection("ssh");

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

        report.record(item("RoyalSSHConnection", ItemStatus::Imported), SSH);
        report.record(item("RoyalSSHConnection", ItemStatus::Partial), SSH);
        report.record(
            item("RoyalSSHConnection", ItemStatus::Unsupported),
            NodeTarget::Unsupported,
        );
        report.record(item("RoyalFolder", ItemStatus::Skipped), NodeTarget::Folder);

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

        report.record(
            item("RoyalFolder", ItemStatus::Imported),
            NodeTarget::Folder,
        );
        report.record(item("RoyalFolder", ItemStatus::Partial), NodeTarget::Folder);

        let statuses: Vec<ItemStatus> = report.items.iter().map(|i| i.status).collect();
        assert_eq!(statuses, vec![ItemStatus::Partial]);
    }

    #[test]
    fn record_counts_every_object_per_kind_of_node() {
        let mut report = ImportReport::new("royal", "test.rtsz", Utc::now());

        report.record(item("RoyalSSHConnection", ItemStatus::Imported), SSH);
        report.record(
            item("RoyalSSHConnection", ItemStatus::Unsupported),
            NodeTarget::Unsupported,
        );
        report.record(item("RoyalTrash", ItemStatus::Skipped), NodeTarget::Folder);
        report.record(
            item("RoyalCredential", ItemStatus::Partial),
            NodeTarget::Credential,
        );

        let expected = KindCounts {
            folders: 1,
            connections: BTreeMap::from([("ssh".to_owned(), 1)]),
            credentials: 1,
            unsupported: 1,
        };
        assert_eq!(report.kinds, expected);
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
