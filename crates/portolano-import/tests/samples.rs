//! Count test on the real Royal documents (SPEC §8.3.7).
//!
//! Ignored by default and enabled by `PORTOLANO_SAMPLES_DIR`; a relative
//! value is resolved against the workspace root, where the Definition of
//! Done runs it. The documents are real: the test prints counts only, never
//! names, hosts, users or file names (SPEC §1.4).

// Test code: clippy.toml allows these in `#[test]` functions only, while the
// helpers of an integration test crate must fail loudly too.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::Read;
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use portolano_core::{Node, NodeKind};
use portolano_import::royal::RoyalImporter;
use portolano_import::{ImportOptions, Importer, NonInteractive};
use quick_xml::Reader;
use quick_xml::events::Event;

const SAMPLES_DIR_VAR: &str = "PORTOLANO_SAMPLES_DIR";
const FOLDER: &str = "RoyalFolder";
const SSH: &str = "RoyalSSHConnection";
const RDS: &str = "RoyalRDSConnection";
const CREDENTIAL: &str = "RoyalCredential";
const TRASH: &str = "RoyalTrash";

#[test]
#[ignore = "needs the real documents: set PORTOLANO_SAMPLES_DIR"]
fn real_samples_match_the_source_counts() {
    let Some(dir) = samples_dir() else {
        println!("{SAMPLES_DIR_VAR} non impostata: test saltato");
        return;
    };
    let files = sample_files(&dir);
    assert!(
        !files.is_empty(),
        "nessun documento .rtsz o .rtsx nei campioni"
    );

    for (index, path) in files.iter().enumerate() {
        let number = index + 1;
        let bytes = std::fs::read(path).unwrap();
        let source = SourceCounts::from_objects(&source_objects(&xml_text(&bytes)));
        let opts = ImportOptions {
            source_file_name: format!("campione-{number}.rtsz"),
            include_trash: false,
        };
        let outcome = RoyalImporter
            .import(&bytes, &opts, &mut NonInteractive)
            .unwrap();
        let mut imported = BTreeMap::new();
        count_nodes(&outcome.root, &mut imported);
        let imported = |kind: &str| imported.get(kind).copied().unwrap_or(0);

        let report = &outcome.report.counts;
        let statuses: usize = report
            .values()
            .map(|c| c.imported + c.partial + c.unsupported + c.skipped)
            .sum();
        let partial: usize = report.values().map(|c| c.partial).sum();
        let unsupported: usize = report.values().map(|c| c.unsupported).sum();
        let skipped: usize = report.values().map(|c| c.skipped).sum();
        println!(
            "documento {number}: oggetti {}, cartelle {}/{}, connessioni {}/{}, credenziali {}/{}, \
             stati {statuses} (parziali {partial}, non supportati {unsupported}, saltati {skipped})",
            source.total,
            imported("folder"),
            source.folders,
            imported("connection"),
            source.connections(),
            imported("credential"),
            source.credentials,
        );

        assert_eq!(
            imported("folder"),
            source.folders,
            "cartelle, documento {number}"
        );
        assert_eq!(
            imported("connection"),
            source.connections(),
            "connessioni, documento {number}"
        );
        assert_eq!(
            imported("credential"),
            source.credentials,
            "credenziali, documento {number}"
        );
        assert_eq!(
            statuses, source.total,
            "somma degli stati, documento {number}"
        );
        for (type_name, count) in &source.by_type {
            let reported = report.get(type_name).map_or(0, |c| c.source);
            assert_eq!(reported, *count, "oggetti di un tipo, documento {number}");
        }
    }
}

/// The directory of the samples, if the test is enabled.
fn samples_dir() -> Option<PathBuf> {
    let value = std::env::var_os(SAMPLES_DIR_VAR)?;
    let dir = PathBuf::from(value);
    if dir.is_absolute() {
        return Some(dir);
    }
    // Cargo runs tests from the crate directory; the DoD passes a path
    // relative to the workspace root.
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    Some(workspace_root.join(dir))
}

fn sample_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .expect("directory dei campioni non leggibile")
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| {
                    extension.eq_ignore_ascii_case("rtsz") || extension.eq_ignore_ascii_case("rtsx")
                })
        })
        .collect();
    files.sort();
    files
}

/// The XML text, read independently of the importer.
fn xml_text(bytes: &[u8]) -> String {
    let mut plain = bytes.to_vec();
    if bytes.starts_with(b"\x1F\x8B") {
        plain.clear();
        GzDecoder::new(bytes).read_to_end(&mut plain).unwrap();
    }
    assert!(
        plain.starts_with(b"\xEF\xBB\xBF") || plain.starts_with(b"<"),
        "formato del campione non gestito dal test"
    );
    let text = String::from_utf8(plain).unwrap();
    text.trim_start_matches('\u{feff}').to_owned()
}

/// The fields of a top-level object that the counts need.
#[derive(Default)]
struct SourceObject {
    type_name: String,
    fields: HashMap<String, String>,
}

impl SourceObject {
    fn field(&self, name: &str) -> &str {
        self.fields.get(name).map_or("", |value| value.trim())
    }

    fn flag(&self, name: &str) -> bool {
        self.field(name).eq_ignore_ascii_case("true")
    }
}

fn source_objects(xml: &str) -> Vec<SourceObject> {
    let mut reader = Reader::from_str(xml);
    let mut objects = Vec::new();
    let mut depth = 0_usize;
    let mut current = SourceObject::default();
    let mut field = String::new();
    loop {
        match reader.read_event().unwrap() {
            Event::Start(element) => {
                depth += 1;
                let name = element.name().as_ref().to_owned();
                match depth {
                    2 => {
                        current = SourceObject {
                            type_name: name,
                            ..SourceObject::default()
                        }
                    }
                    3 => field = name,
                    _ => {}
                }
            }
            Event::Empty(element) if depth == 1 => objects.push(SourceObject {
                type_name: element.name().as_ref().to_owned(),
                ..SourceObject::default()
            }),
            Event::Text(text) if depth == 3 => {
                current
                    .fields
                    .entry(field.clone())
                    .or_default()
                    .push_str(&text);
            }
            Event::End(_) => {
                if depth == 2 {
                    objects.push(std::mem::take(&mut current));
                }
                depth -= 1;
            }
            Event::Eof => return objects,
            _ => {}
        }
    }
}

/// Counts per type of the objects outside the trash, and the total.
struct SourceCounts {
    total: usize,
    by_type: BTreeMap<String, usize>,
    folders: usize,
    ssh: usize,
    rds: usize,
    telnet_or_serial: usize,
    credentials: usize,
}

impl SourceCounts {
    fn from_objects(objects: &[SourceObject]) -> Self {
        let mut by_type = BTreeMap::new();
        for object in objects {
            *by_type.entry(object.type_name.clone()).or_default() += 1;
        }
        let parents: HashMap<&str, &str> = objects
            .iter()
            .map(|object| (object.field("ID"), object.field("ParentID")))
            .collect();
        let trash_ids: HashSet<&str> = objects
            .iter()
            .filter(|object| object.type_name == TRASH)
            .map(|object| object.field("ID"))
            .collect();
        let outside_trash: Vec<&SourceObject> = objects
            .iter()
            .filter(|object| !in_trash(object.field("ID"), &parents, &trash_ids))
            .collect();
        let count = |type_name: &str| {
            outside_trash
                .iter()
                .filter(|object| object.type_name == type_name)
                .count()
        };
        let telnet_or_serial = outside_trash
            .iter()
            .filter(|object| object.type_name == SSH)
            .filter(|object| {
                object.flag("IsTelnetConnection") || object.flag("IsSerialPortConnection")
            })
            .count();
        Self {
            total: objects.len(),
            by_type,
            folders: count(FOLDER),
            ssh: count(SSH),
            rds: count(RDS),
            telnet_or_serial,
            credentials: count(CREDENTIAL),
        }
    }

    fn connections(&self) -> usize {
        self.ssh + self.rds - self.telnet_or_serial
    }
}

/// Whether the object is the trash or one of its descendants.
fn in_trash(id: &str, parents: &HashMap<&str, &str>, trash_ids: &HashSet<&str>) -> bool {
    let mut current = id;
    let mut seen = HashSet::new();
    while seen.insert(current) {
        if trash_ids.contains(current) {
            return true;
        }
        match parents.get(current) {
            Some(parent) if !parent.is_empty() => current = parent,
            _ => return false,
        }
    }
    false
}

/// Imported folders, connections and credentials, by kind. Only nodes from
/// the Royal types of SPEC §8.3.7 count: the root (the `RoyalDocument`) and
/// Secure Gateways do not.
fn count_nodes(node: &Node, counts: &mut BTreeMap<&'static str, usize>) {
    let source_type = node
        .source
        .as_ref()
        .map_or("", |source| source.source_type.as_str());
    let kind = match (&node.kind, source_type) {
        (NodeKind::Folder(_), FOLDER) => Some("folder"),
        (NodeKind::Connection(_), SSH | RDS) => Some("connection"),
        (NodeKind::Credential(_), CREDENTIAL) => Some("credential"),
        _ => None,
    };
    if let Some(kind) = kind {
        *counts.entry(kind).or_default() += 1;
    }
    if let Some(folder) = node.as_folder() {
        for child in &folder.children {
            count_nodes(child, counts);
        }
    }
}
