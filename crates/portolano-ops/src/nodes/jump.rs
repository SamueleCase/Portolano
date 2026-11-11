//! A jump host as given by the user (`add ssh --jump PATH|ADDR`).

use portolano_connectors::ssh::JumpHost;
use portolano_core::Document;

use crate::{OpsError, Result};

/// Protocol a jump host must use.
const SSH: &str = "ssh";

/// The path of an SSH connection of the document, or an address
/// `[user@]host[:port]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JumpInput(pub String);

impl JumpInput {
    /// The jump host to store (DEC-022.3): a value that leads to a node is a
    /// reference, and the node must be an SSH connection; a value that leads
    /// nowhere is an address, unless it is written as a path (`/…` or `@…`).
    ///
    /// # Errors
    /// [`OpsError::InvalidJumpHost`] for a node that is not an SSH
    /// connection; [`portolano_core::Error::NotFound`] for a missing path;
    /// [`portolano_core::Error::Ambiguous`] and the other path errors.
    pub fn resolve(self, doc: &Document) -> Result<JumpHost> {
        let JumpInput(value) = self;
        match doc.resolve_path(&value) {
            Ok(id) => {
                let is_ssh = doc
                    .find(id)
                    .and_then(|node| node.as_connection())
                    .is_some_and(|connection| connection.protocol == SSH);
                if is_ssh {
                    Ok(JumpHost::Ref { id })
                } else {
                    Err(OpsError::InvalidJumpHost { path: value })
                }
            }
            Err(portolano_core::Error::NotFound { .. })
                if !value.starts_with('/') && !value.starts_with('@') =>
            {
                Ok(JumpHost::Address { address: value })
            }
            Err(error) => Err(error.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::testutil::*;

    fn resolve(doc: &Document, value: &str) -> Result<JumpHost> {
        JumpInput(value.to_owned()).resolve(doc)
    }

    #[test]
    fn path_of_an_ssh_connection_is_a_reference() {
        let doc = document(vec![connection("bastion", "ssh")]);
        assert_eq!(
            resolve(&doc, "bastion").unwrap(),
            JumpHost::Ref {
                id: id_of(&doc, "/bastion")
            }
        );
    }

    #[test]
    fn value_that_leads_nowhere_is_an_address() {
        let doc = document(vec![]);
        assert_eq!(
            resolve(&doc, "ops@jump.example.com:2200").unwrap(),
            JumpHost::Address {
                address: "ops@jump.example.com:2200".to_owned()
            }
        );
    }

    #[test]
    fn missing_path_written_as_a_path_is_an_error() {
        let doc = document(vec![]);
        for value in ["/bastion", "@abcdef12"] {
            assert!(matches!(
                resolve(&doc, value),
                Err(OpsError::Core(portolano_core::Error::NotFound { .. }))
            ));
        }
    }

    #[test]
    fn node_that_is_not_an_ssh_connection_is_refused() {
        let doc = document(vec![folder("Rete", vec![]), connection("dc01", "rdp")]);
        for value in ["/Rete", "/dc01"] {
            assert!(matches!(
                resolve(&doc, value),
                Err(OpsError::InvalidJumpHost { .. })
            ));
        }
    }

    #[test]
    fn ambiguous_path_is_an_error() {
        let doc = document(vec![
            connection("bastion", "ssh"),
            connection("Bastion", "ssh"),
        ]);
        assert!(matches!(
            resolve(&doc, "BASTION"),
            Err(OpsError::Core(portolano_core::Error::Ambiguous { .. }))
        ));
    }
}
