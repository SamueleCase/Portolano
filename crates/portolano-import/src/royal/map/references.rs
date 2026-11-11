//! The objects other objects can point to, collected before the mapping so
//! that a reference can be checked wherever it appears in the tree.

use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::royal::tree::{Tree, TreeNode};
use crate::royal::types;
use crate::royal::xml::RawObject;

/// Host and port of a reusable Remote Desktop Gateway.
pub(super) struct GatewayAddress {
    pub(super) host: String,
    pub(super) port: Option<u16>,
}

/// Targets of references, among the objects that will be imported.
pub(super) struct References {
    /// Every object of the file, imported or not.
    pub(super) in_file: HashSet<Uuid>,
    pub(super) credentials: HashSet<Uuid>,
    pub(super) secure_gateways: HashSet<Uuid>,
    pub(super) rds_gateways: HashMap<Uuid, GatewayAddress>,
}

impl References {
    pub(super) fn collect(objects: &[RawObject], tree: &Tree, include_trash: bool) -> Self {
        let mut references = Self {
            in_file: tree.ids.iter().copied().collect(),
            credentials: HashSet::new(),
            secure_gateways: HashSet::new(),
            rds_gateways: HashMap::new(),
        };
        references.add_subtree(objects, &tree.ids, &tree.root, include_trash);
        references
    }

    fn add_subtree(
        &mut self,
        objects: &[RawObject],
        ids: &[Uuid],
        node: &TreeNode,
        include_trash: bool,
    ) {
        let object = &objects[node.object];
        if is_skipped_trash(object, include_trash) {
            return;
        }
        let id = ids[node.object];
        match object.type_name.as_str() {
            types::CREDENTIAL => {
                self.credentials.insert(id);
            }
            types::SECURE_GATEWAY => {
                self.secure_gateways.insert(id);
            }
            // [ipo] The gateway object is assumed to hold its address like a
            // connection does, in `URI` and `Port`.
            types::RDS_GATEWAY => {
                if let Some(host) = object.value("URI") {
                    let port = object.value("Port").and_then(|port| port.parse().ok());
                    let address = GatewayAddress {
                        host: host.to_owned(),
                        port,
                    };
                    self.rds_gateways.insert(id, address);
                }
            }
            _ => {}
        }
        for child in &node.children {
            self.add_subtree(objects, ids, child, include_trash);
        }
    }
}

/// Whether the object is the trash and the trash is not imported.
pub(super) fn is_skipped_trash(object: &RawObject, include_trash: bool) -> bool {
    object.type_name == types::TRASH && !include_trash
}
