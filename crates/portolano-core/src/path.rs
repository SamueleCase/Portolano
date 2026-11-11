//! Path syntax and resolution (SPEC §4.2, DEC-002, DEC-014).
//!
//! Segments are separated by `/`; a leading `/` is optional and `/` alone is
//! the root. In names, `\/` is a literal slash and `\\` a backslash; any other
//! escape, a trailing `\`, an empty segment and a trailing `/` are invalid.
//!
//! `@<uuid>` or `@<hex prefix of at least 6 digits>` addresses a node by id,
//! anywhere in a path: as the first segment it searches the whole document,
//! after other segments it searches the descendants, at any depth, of the
//! nodes matched so far. Any other segment starting with `@` is a name, and
//! there is no escape for `@`: a node named exactly like an id reference (for
//! example `@cafe12`) can only be addressed by its id.
//!
//! Sibling names need not be unique, so a path is matched against every node
//! it can lead to and is ambiguous only if it ends on more than one. Names are
//! compared exactly first; if the whole path matches nothing, it is compared
//! again ignoring case.
//!
//! [`Document::path_of`] returns paths with a leading `/` and without the
//! document name, such as `/Clienti/ACME/web01`.

use std::collections::HashSet;

use uuid::Uuid;

use crate::error::{Error, Result};
use crate::model::{Document, Node};
use crate::tree::{ensure_within_max_depth, walk_nodes};

/// Shortest hex prefix accepted after `@`, to keep accidental matches rare.
const MIN_ID_PREFIX: usize = 6;

impl Document {
    /// The path of a node, with escapes applied; `None` if the node does not
    /// exist.
    pub fn path_of(&self, id: Uuid) -> Option<String> {
        let lineage = lineage(&self.nodes, id)?;
        Some(
            lineage
                .iter()
                .map(|node| format!("/{}", escape_path_segment(&node.name)))
                .collect(),
        )
    }

    /// Resolves a path to a node id. Names are compared exactly first, then
    /// case-insensitively if nothing matched.
    ///
    /// # Errors
    /// [`Error::NotFound`](crate::Error::NotFound) if nothing matches,
    /// [`Error::Ambiguous`](crate::Error::Ambiguous) if several nodes match,
    /// [`Error::InvalidPath`](crate::Error::InvalidPath) for bad syntax and
    /// for `/`, because the root is not a node.
    pub fn resolve_path(&self, path: &str) -> Result<Uuid> {
        self.resolve_location(path)?
            .ok_or_else(|| invalid(path, "la radice non è un nodo"))
    }

    /// Resolves a path to a place in the tree: `None` is the root (`/`),
    /// `Some(id)` a node. For commands where the root is a valid target, such
    /// as `tree /` or `move X /` (DEC-014).
    ///
    /// # Errors
    /// As [`Document::resolve_path`], except that `/` is accepted.
    pub fn resolve_location(&self, path: &str) -> Result<Option<Uuid>> {
        let segments = parse(path)?;
        Ok(self.locate(path, &segments)?.map(|node| node.id))
    }

    /// Splits a path into its parent and its last segment, for the `add`
    /// commands. The parent (`None` = root) must exist and be a folder; the
    /// document is not modified.
    ///
    /// # Errors
    /// As [`Document::resolve_path`] for the parent, plus
    /// [`Error::NotAFolder`](crate::Error::NotAFolder) if the parent is not a
    /// folder and [`Error::InvalidPath`](crate::Error::InvalidPath) if the
    /// last segment is missing or is an `@id`.
    pub fn resolve_parent_path(&self, path: &str) -> Result<(Option<Uuid>, String)> {
        let (parent, name) = split_last(path)?;
        let parent = self.locate(&render(&parent), &parent)?;
        Ok((folder_id(parent)?, name))
    }

    /// Like [`Document::resolve_parent_path`], but creates the missing
    /// intermediate folders (`add … -p`).
    ///
    /// # Errors
    /// As [`Document::resolve_parent_path`], except that missing folders are
    /// created instead of reported; a missing `@id` segment cannot be created
    /// and gives [`Error::NotFound`](crate::Error::NotFound);
    /// [`Error::TooDeep`](crate::Error::TooDeep) if the node to add under the
    /// returned parent would be deeper than
    /// [`MAX_DEPTH`](crate::model::MAX_DEPTH). Nothing is created when an
    /// error is returned.
    pub fn ensure_parent_path(&mut self, path: &str) -> Result<(Option<Uuid>, String)> {
        let (parent, name) = split_last(path)?;
        // The longest prefix that exists anchors the folders to create; the
        // empty prefix (the root) always exists, so the loop terminates.
        let mut existing = parent.len();
        let mut anchor = loop {
            let prefix = &parent[..existing];
            match self.locate(&render(prefix), prefix) {
                Ok(node) => break folder_id(node)?,
                Err(Error::NotFound { .. }) if existing > 0 => existing -= 1,
                Err(error) => return Err(error),
            }
        };
        let missing = parent[existing..]
            .iter()
            .enumerate()
            .map(|(offset, segment)| match segment {
                Segment::Name(name) => Ok(name),
                Segment::Id { .. } => Err(Error::NotFound {
                    query: render(&parent[..=existing + offset]),
                }),
            })
            .collect::<Result<Vec<_>>>()?;
        // Checked before creating anything; the node the caller adds under
        // the returned parent counts too.
        ensure_within_max_depth(self.child_level(anchor) + missing.len())?;
        for folder_name in missing {
            let folder = Node::folder(folder_name.as_str());
            let id = folder.id;
            self.insert(anchor, None, folder)?;
            anchor = Some(id);
        }
        Ok((anchor, name))
    }

    /// Builds [`Error::Ambiguous`] for `query`, listing `nodes` with their
    /// paths.
    pub(crate) fn ambiguous(&self, query: String, nodes: &[&Node]) -> Error {
        Error::Ambiguous {
            query,
            candidates: nodes
                .iter()
                .map(|node| (node.id, self.path_of(node.id).unwrap_or_default()))
                .collect(),
        }
    }

    /// The node at `segments` (`None` for no segments, the root), comparing
    /// names exactly and then, if nothing matched, ignoring case. `query`
    /// names the path in errors.
    fn locate(&self, query: &str, segments: &[Segment]) -> Result<Option<&Node>> {
        if segments.is_empty() {
            return Ok(None);
        }
        let mut matched = self.matching_nodes(segments, Case::Sensitive);
        if matched.is_empty() {
            matched = self.matching_nodes(segments, Case::Insensitive);
        }
        match matched.as_slice() {
            [] => Err(Error::NotFound {
                query: query.to_owned(),
            }),
            [node] => Ok(Some(node)),
            _ => Err(self.ambiguous(query.to_owned(), &matched)),
        }
    }

    /// Every node a non-empty path can lead to.
    fn matching_nodes(&self, segments: &[Segment], case: Case) -> Vec<&Node> {
        let mut matched: Vec<&Node> = Vec::new();
        for (index, segment) in segments.iter().enumerate() {
            let scopes: Vec<&[Node]> = if index == 0 {
                vec![&self.nodes]
            } else {
                matched
                    .iter()
                    .filter_map(|node| node.as_folder())
                    .map(|folder| folder.children.as_slice())
                    .collect()
            };
            let mut next: Vec<&Node> = match segment {
                Segment::Name(name) => scopes
                    .into_iter()
                    .flatten()
                    .filter(|node| case.same(&node.name, name))
                    .collect(),
                Segment::Id { prefix, .. } => scopes
                    .into_iter()
                    .flat_map(walk_nodes)
                    .map(|(_, node)| node)
                    .filter(|node| has_id_prefix(node.id, prefix))
                    .collect(),
            };
            // Nested scopes (an id prefix matching both a folder and one of
            // its descendants) would otherwise yield the same node twice.
            let mut seen = HashSet::new();
            next.retain(|node| seen.insert(std::ptr::from_ref(*node)));
            matched = next;
        }
        matched
    }
}

/// Whether two names are equal ignoring case, comparing their Unicode
/// lowercase forms (`char::to_lowercase`, not full case folding).
pub(crate) fn same_name_ignoring_case(left: &str, right: &str) -> bool {
    left.chars()
        .flat_map(char::to_lowercase)
        .eq(right.chars().flat_map(char::to_lowercase))
}

/// The key under which [`same_name_ignoring_case`] considers names equal:
/// two names compare equal exactly when their keys do.
pub(crate) fn lowercase_name(name: &str) -> String {
    name.chars().flat_map(char::to_lowercase).collect()
}

/// One parsed path segment.
enum Segment {
    /// A node name, unescaped.
    Name(String),
    /// An `@id` reference.
    Id {
        /// The segment as written, without the `@`.
        text: String,
        /// The id or prefix as lowercase hex digits without hyphens.
        prefix: String,
    },
}

impl Segment {
    fn from_text(text: String) -> Self {
        if let Some(reference) = text.strip_prefix('@') {
            if let Ok(id) = Uuid::try_parse(reference) {
                let prefix = id.simple().to_string();
                return Self::Id {
                    text: reference.to_owned(),
                    prefix,
                };
            }
            if reference.len() >= MIN_ID_PREFIX
                && reference.bytes().all(|byte| byte.is_ascii_hexdigit())
            {
                return Self::Id {
                    text: reference.to_owned(),
                    prefix: reference.to_ascii_lowercase(),
                };
            }
        }
        Self::Name(text)
    }
}

#[derive(Clone, Copy)]
enum Case {
    Sensitive,
    Insensitive,
}

impl Case {
    fn same(self, left: &str, right: &str) -> bool {
        match self {
            Self::Sensitive => left == right,
            Self::Insensitive => same_name_ignoring_case(left, right),
        }
    }
}

fn parse(path: &str) -> Result<Vec<Segment>> {
    if path.is_empty() {
        return Err(invalid(path, "percorso vuoto"));
    }
    let body = path.strip_prefix('/').unwrap_or(path);
    if body.is_empty() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    let mut current = String::new();
    let mut chars = body.chars();
    while let Some(character) = chars.next() {
        match character {
            '\\' => match chars.next() {
                Some(escaped @ ('/' | '\\')) => current.push(escaped),
                Some(_) => {
                    return Err(invalid(
                        path,
                        "escape sconosciuto: sono ammessi solo \\/ e \\\\",
                    ));
                }
                None => return Err(invalid(path, "'\\' finale senza carattere da proteggere")),
            },
            '/' => names.push(std::mem::take(&mut current)),
            other => current.push(other),
        }
    }
    names.push(current);
    if names.iter().any(String::is_empty) {
        return Err(invalid(path, "segmento vuoto ('/' doppio o finale)"));
    }
    Ok(names.into_iter().map(Segment::from_text).collect())
}

/// Splits a path into the segments of its parent and its last name.
fn split_last(path: &str) -> Result<(Vec<Segment>, String)> {
    let mut segments = parse(path)?;
    match segments.pop() {
        Some(Segment::Name(name)) => Ok((segments, name)),
        Some(Segment::Id { .. }) => Err(invalid(
            path,
            "l'ultimo segmento deve essere un nome, non un id",
        )),
        None => Err(invalid(path, "manca il nome dell'ultimo segmento")),
    }
}

/// The segments in path form, for error messages about part of a path.
fn render(segments: &[Segment]) -> String {
    if segments.is_empty() {
        return "/".to_owned();
    }
    segments
        .iter()
        .map(|segment| match segment {
            Segment::Name(name) => format!("/{}", escape_path_segment(name)),
            Segment::Id { text, .. } => format!("/@{text}"),
        })
        .collect()
}

/// A node name as a path segment (SPEC §4.2): a backslash becomes `\\`
/// and a slash `\/`, as in the paths returned by [`Document::path_of`].
/// Callers that build the paths of many nodes while walking the tree use it
/// instead of asking `path_of` for each node.
pub fn escape_path_segment(name: &str) -> String {
    name.replace('\\', "\\\\").replace('/', "\\/")
}

/// The id of a parent location, which must be the root or a folder.
fn folder_id(parent: Option<&Node>) -> Result<Option<Uuid>> {
    match parent {
        None => Ok(None),
        Some(node) if node.is_folder() => Ok(Some(node.id)),
        Some(node) => Err(Error::NotAFolder { id: node.id }),
    }
}

fn has_id_prefix(id: Uuid, prefix: &str) -> bool {
    let mut buffer = Uuid::encode_buffer();
    id.simple().encode_lower(&mut buffer).starts_with(prefix)
}

/// The nodes from the outermost folder down to the node `id`, both included.
fn lineage(nodes: &[Node], id: Uuid) -> Option<Vec<&Node>> {
    nodes.iter().find_map(|node| {
        if node.id == id {
            return Some(vec![node]);
        }
        let mut chain = lineage(&node.as_folder()?.children, id)?;
        chain.insert(0, node);
        Some(chain)
    })
}

fn invalid(path: &str, reason: &str) -> Error {
    Error::InvalidPath {
        path: path.to_owned(),
        reason: reason.to_owned(),
    }
}

#[cfg(test)]
mod tests;
