//! Registry of the available protocols.

use crate::protocol::Protocol;
use crate::rdp::RdpProtocol;
use crate::ssh::SshProtocol;

/// The protocols known to Portolano, looked up by id.
pub struct Registry {
    protocols: Vec<Box<dyn Protocol>>,
}

impl Registry {
    /// A registry with the built-in protocols: `ssh` and `rdp`.
    pub fn with_defaults() -> Registry {
        Registry {
            protocols: vec![Box::new(SshProtocol), Box::new(RdpProtocol)],
        }
    }

    /// The protocol with the given id.
    pub fn get(&self, id: &str) -> Option<&dyn Protocol> {
        self.protocols
            .iter()
            .find(|protocol| protocol.id() == id)
            .map(AsRef::as_ref)
    }

    /// The ids of the registered protocols, in registration order.
    pub fn ids(&self) -> Vec<&'static str> {
        self.protocols
            .iter()
            .map(|protocol| protocol.id())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn defaults_register_ssh_then_rdp() {
        let registry = Registry::with_defaults();

        assert_eq!(registry.ids(), vec!["ssh", "rdp"]);
    }

    #[test]
    fn get_finds_a_protocol_by_id() {
        let registry = Registry::with_defaults();

        let rdp = registry.get("rdp").map(|protocol| protocol.default_port());

        assert_eq!(rdp, Some(3389));
    }

    #[test]
    fn get_returns_none_for_an_unknown_id() {
        let registry = Registry::with_defaults();

        assert!(registry.get("vnc").is_none());
    }
}
