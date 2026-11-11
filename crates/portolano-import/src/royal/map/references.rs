//! The objects other objects can point to, collected before the mapping so
//! that a reference can be checked wherever it appears in the tree.

use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::royal::tree::Tree;
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
    /// Every object of the tree is imported unless it is in a skipped trash,
    /// so a flat scan is enough.
    pub(super) fn collect(objects: &[RawObject], tree: &Tree, include_trash: bool) -> Self {
        let mut references = Self {
            in_file: tree.ids.iter().copied().collect(),
            credentials: HashSet::new(),
            secure_gateways: HashSet::new(),
            rds_gateways: HashMap::new(),
        };
        for (index, object) in objects.iter().enumerate() {
            if !tree.is_skipped_trash(index, include_trash) {
                references.add(object, tree.ids[index]);
            }
        }
        references
    }

    fn add(&mut self, object: &RawObject, id: Uuid) {
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
    }
}
