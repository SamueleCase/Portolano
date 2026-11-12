//! The search of the tree (SPEC-UI §5.4): by name, host and tag, ignoring
//! case. The filters by type and favorite are of the interface, on the
//! tree it already has.

use portolano_core::Document;
use uuid::Uuid;

/// The nodes whose name, host or one of whose tags contains `query`,
/// ignoring case, in tree order. A blank query matches nothing.
pub fn search_nodes(doc: &Document, query: &str) -> Vec<Uuid> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return Vec::new();
    }
    let contains = |text: &str| text.to_lowercase().contains(&query);
    doc.walk()
        .map(|(_, node)| node)
        .filter(|node| {
            contains(&node.name)
                || node.as_connection().is_some_and(|c| contains(&c.host))
                || node.tags.iter().any(|tag| contains(tag))
        })
        .map(|node| node.id)
        .collect()
}

#[cfg(test)]
mod tests {
    use portolano_core::Node;
    use pretty_assertions::assert_eq;

    use super::*;

    fn sample() -> (Document, Vec<Uuid>) {
        let mut doc = Document::new("Test");
        let mut prod = Node::folder("Produzione");
        let web01 = Node::connection("web01", "ssh", "web01.example.com");
        let mut dc01 = Node::connection("dc01", "rdp", "192.0.2.10");
        dc01.tags.push("Windows".to_owned());
        let ids = vec![prod.id, web01.id, dc01.id];
        if let Some(folder) = prod.as_folder_mut() {
            folder.children = vec![web01, dc01];
        }
        doc.nodes.push(prod);
        (doc, ids)
    }

    #[test]
    fn name_matches_ignoring_case() {
        let (doc, ids) = sample();

        assert_eq!(search_nodes(&doc, "PRODU"), vec![ids[0]]);
    }

    #[test]
    fn host_matches() {
        let (doc, ids) = sample();

        assert_eq!(search_nodes(&doc, "192.0.2"), vec![ids[2]]);
    }

    #[test]
    fn tag_matches() {
        let (doc, ids) = sample();

        assert_eq!(search_nodes(&doc, "windows"), vec![ids[2]]);
    }

    #[test]
    fn several_matches_are_in_tree_order() {
        let (doc, ids) = sample();

        assert_eq!(search_nodes(&doc, "01"), vec![ids[1], ids[2]]);
    }

    #[test]
    fn blank_query_matches_nothing() {
        let (doc, _) = sample();

        assert!(search_nodes(&doc, "  ").is_empty());
    }
}
