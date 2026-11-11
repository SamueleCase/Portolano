//! Builders of synthetic Royal documents and helpers shared by the tests.
//! Hosts and users are fake (SPEC §1.3).

use std::io::Write;
use std::path::PathBuf;

use flate2::Compression;
use flate2::write::GzEncoder;
use portolano_core::{Connection, Node, NodeKind, Secret};
use portolano_import::royal::RoyalImporter;
use portolano_import::{
    ImportError, ImportOptions, ImportOutcome, Importer, ItemStatus, NonInteractive, ReportItem,
    SecretPrompter, SecretRequest,
};
use uuid::Uuid;

/// Name of the `RoyalDocument` built by [`document`].
pub const DOCUMENT_NAME: &str = "Doc";
/// A random-looking base64 string of 80 bytes, like a Royal ciphertext.
pub const FAKE_CIPHERTEXT: &str = "c3ludGhldGljLWNpcGhlcnRleHQtZm9yLXRlc3RzLW9ubHktbm90LWRlcml2ZWQtZnJvbS1hbnktcGFzc3dvcmQtMTIzNDU2Nzg5MA==";

/// The id of object number `n`; [`document`] uses 1.
pub fn id(n: u32) -> String {
    format!("d0000000-0000-4000-8000-{n:012x}")
}

pub fn uuid(n: u32) -> Uuid {
    Uuid::parse_str(&id(n)).unwrap()
}

/// A Royal object of `type_name` with `ID` [`id`]`(n)`, under `parent`.
pub fn object(type_name: &str, n: u32, parent: u32, fields: &[(&str, &str)]) -> String {
    let mut xml = format!(
        "<{type_name}><ID>{}</ID><ParentID>{}</ParentID>",
        id(n),
        id(parent)
    );
    for (name, value) in fields {
        xml.push_str(&format!("<{name}>{value}</{name}>"));
    }
    xml.push_str(&format!("</{type_name}>"));
    xml
}

/// A document whose `RoyalDocument` has id 1, followed by `objects`.
pub fn document(objects: &[String]) -> String {
    format!(
        "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<RTSZDocument>\
         <RoyalDocument><ID>{}</ID><Name>{DOCUMENT_NAME}</Name></RoyalDocument>{}</RTSZDocument>",
        id(1),
        objects.concat()
    )
}

pub fn options() -> ImportOptions {
    ImportOptions {
        source_file_name: "test.rtsz".to_owned(),
        include_trash: false,
    }
}

/// Imports with [`NonInteractive`].
pub fn import(xml: &str) -> ImportOutcome {
    import_bytes(xml.as_bytes(), &options(), &mut NonInteractive).unwrap()
}

pub fn import_bytes(
    input: &[u8],
    opts: &ImportOptions,
    prompter: &mut dyn SecretPrompter,
) -> Result<ImportOutcome, ImportError> {
    RoyalImporter.import(input, opts, prompter)
}

/// Bytes of a fixture in `tests/fixtures`.
pub fn fixture(name: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    std::fs::read(path).unwrap()
}

pub fn gzip(bytes: &[u8]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(bytes).unwrap();
    encoder.finish().unwrap()
}

/// The node with `id` in the tree under `root`.
pub fn find(root: &Node, id: Uuid) -> &Node {
    try_find(root, id).unwrap_or_else(|| panic!("node {id} not found"))
}

pub fn try_find(node: &Node, id: Uuid) -> Option<&Node> {
    if node.id == id {
        return Some(node);
    }
    node.as_folder()?
        .children
        .iter()
        .find_map(|child| try_find(child, id))
}

pub fn connection(root: &Node, n: u32) -> &Connection {
    find(root, uuid(n)).as_connection().unwrap()
}

pub fn child_names(node: &Node) -> Vec<&str> {
    node.as_folder()
        .unwrap()
        .children
        .iter()
        .map(|child| child.name.as_str())
        .collect()
}

/// The report item of object `n`; only items not fully imported have one.
pub fn item(outcome: &ImportOutcome, n: u32) -> Option<&ReportItem> {
    outcome
        .report
        .items
        .iter()
        .find(|item| item.source_id == id(n))
}

pub fn status(outcome: &ImportOutcome, n: u32) -> ItemStatus {
    item(outcome, n).map_or(ItemStatus::Imported, |item| item.status)
}

pub fn reasons(outcome: &ImportOutcome, n: u32) -> Vec<String> {
    item(outcome, n).map_or_else(Vec::new, |item| item.reasons.clone())
}

pub fn is_unsupported(node: &Node) -> bool {
    matches!(node.kind, NodeKind::Unsupported(_))
}

/// A prompter that answers every request with the same value, or skips,
/// and remembers the requests.
pub struct ScriptedPrompter {
    answer: Option<&'static str>,
    pub requests: Vec<SecretRequest>,
}

impl ScriptedPrompter {
    pub fn answering(answer: &'static str) -> Self {
        Self {
            answer: Some(answer),
            requests: Vec::new(),
        }
    }

    pub fn skipping() -> Self {
        Self {
            answer: None,
            requests: Vec::new(),
        }
    }
}

impl SecretPrompter for ScriptedPrompter {
    fn ask(&mut self, req: &SecretRequest) -> Result<Option<Secret>, ImportError> {
        self.requests.push(req.clone());
        Ok(self.answer.map(|answer| Secret::new(answer.to_owned())))
    }
}

/// Imports `xml` answering every secret request with `answer`.
pub fn import_answering(xml: &str, answer: &'static str) -> (ImportOutcome, ScriptedPrompter) {
    let mut prompter = ScriptedPrompter::answering(answer);
    let outcome = import_bytes(xml.as_bytes(), &options(), &mut prompter).unwrap();
    (outcome, prompter)
}
