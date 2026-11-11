//! The text form of `tree` (SPEC §10.2).

use portolano_ops::inspect::{Entry, NodeType};
use uuid::Uuid;

use super::{display_width, pad, short_id};

/// Icon of a folder; the only emoji of the output, kept in constants
/// (SPEC §11.3).
const FOLDER_ICON: &str = "📁";
/// Icon of a credential.
const CREDENTIAL_ICON: &str = "🔑";
/// Mark of an unsupported node; its trailing space aligns it with the
/// three-column marks of the protocols.
const UNSUPPORTED_MARK: &str = "[?] ";

const BRANCH: &str = "├── ";
const LAST_BRANCH: &str = "└── ";
const CONTINUATION: &str = "│   ";
const BLANK: &str = "    ";

/// Spaces between a name and its summary, at least.
const SUMMARY_GAP: usize = 2;

/// The first line of the tree: the document name, or the starting node.
pub struct Header {
    /// The text.
    pub label: String,
    /// The summary of the starting node, if any.
    pub summary: String,
}

impl Header {
    /// The header of a subtree that starts at a node.
    pub fn node(node_type: &NodeType, name: &str, id: Uuid, summary: String, ids: bool) -> Header {
        Header {
            label: label(node_type, name, id, ids),
            summary,
        }
    }
}

/// The tree as text: the header, then one line per entry with the branch
/// drawing, the label and the summary aligned in a column.
pub fn render_tree(header: Header, entries: &[Entry], ids: bool) -> String {
    let mut lines = vec![(header.label, header.summary)];
    // For each depth, whether the current node at that depth is the last
    // of its siblings: it decides between a continuation and a blank.
    let mut last_at_depth: Vec<bool> = Vec::new();
    for (index, entry) in entries.iter().enumerate() {
        let is_last = entries[index + 1..]
            .iter()
            .find(|next| next.depth <= entry.depth)
            .is_none_or(|next| next.depth < entry.depth);
        last_at_depth.truncate(entry.depth);
        let mut prefix: String = last_at_depth
            .iter()
            .map(|&last| if last { BLANK } else { CONTINUATION })
            .collect();
        prefix.push_str(if is_last { LAST_BRANCH } else { BRANCH });
        last_at_depth.push(is_last);
        let text = label(&entry.node_type, &entry.name, entry.id, ids);
        lines.push((format!("{prefix}{text}"), entry.summary.clone()));
    }
    let width = lines
        .iter()
        .filter(|(_, summary)| !summary.is_empty())
        .map(|(left, _)| display_width(left))
        .max()
        .unwrap_or(0)
        + SUMMARY_GAP;
    lines
        .into_iter()
        .map(|(left, summary)| {
            if summary.is_empty() {
                left
            } else {
                format!("{}{summary}", pad(&left, width))
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn label(node_type: &NodeType, name: &str, id: Uuid, ids: bool) -> String {
    let mark = match node_type {
        NodeType::Folder => FOLDER_ICON.to_owned(),
        NodeType::Connection { protocol } => format!("[{protocol}]"),
        NodeType::Credential => CREDENTIAL_ICON.to_owned(),
        NodeType::Unsupported { .. } => UNSUPPORTED_MARK.to_owned(),
    };
    if ids {
        format!("{mark} {name}  @{}", short_id(id))
    } else {
        format!("{mark} {name}")
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn entry(depth: usize, node_type: NodeType, name: &str, summary: &str) -> Entry {
        Entry {
            id: Uuid::nil(),
            depth,
            name: name.to_owned(),
            path: String::new(),
            node_type,
            summary: summary.to_owned(),
        }
    }

    fn ssh() -> NodeType {
        NodeType::Connection {
            protocol: "ssh".to_owned(),
        }
    }

    #[test]
    fn tree_draws_branches_and_aligns_summaries() {
        let entries = vec![
            entry(0, NodeType::Folder, "Clienti", ""),
            entry(1, NodeType::Folder, "ACME", ""),
            entry(2, ssh(), "web01", "admin@web01.example.com:22"),
            entry(
                2,
                NodeType::Connection {
                    protocol: "rdp".to_owned(),
                },
                "dc01",
                "ACME\\administrator@192.0.2.10",
            ),
            entry(1, NodeType::Credential, "Admin ACME", "ACME\\administrator"),
            entry(
                0,
                NodeType::Unsupported {
                    source_type: "RoyalVNCConnection".to_owned(),
                },
                "vnc-legacy",
                "non supportato: RoyalVNCConnection",
            ),
        ];
        let header = Header {
            label: "Portolano".to_owned(),
            summary: String::new(),
        };
        let expected = "\
Portolano
├── 📁 Clienti
│   ├── 📁 ACME
│   │   ├── [ssh] web01  admin@web01.example.com:22
│   │   └── [rdp] dc01   ACME\\administrator@192.0.2.10
│   └── 🔑 Admin ACME    ACME\\administrator
└── [?]  vnc-legacy      non supportato: RoyalVNCConnection";
        assert_eq!(render_tree(header, &entries, false), expected);
    }

    #[test]
    fn ids_follow_the_names() {
        let header = Header {
            label: "Doc".to_owned(),
            summary: String::new(),
        };
        let text = render_tree(header, &[entry(0, NodeType::Folder, "F", "")], true);
        assert_eq!(text, "Doc\n└── 📁 F  @00000000");
    }
}
