//! Registry of the available protocols.

// Stub (WP0): the parameters are used once WP4 implements the bodies.
#![expect(unused_variables)]

use crate::protocol::Protocol;

/// The protocols known to Portolano, looked up by id.
// Stub (WP0): the field is read once WP4 implements the methods.
#[expect(dead_code)]
pub struct Registry {
    protocols: Vec<Box<dyn Protocol>>,
}

impl Registry {
    /// A registry with the built-in protocols: `ssh` and `rdp`.
    pub fn with_defaults() -> Registry {
        todo!()
    }

    /// The protocol with the given id.
    pub fn get(&self, id: &str) -> Option<&dyn Protocol> {
        todo!()
    }

    /// The ids of the registered protocols, in registration order.
    pub fn ids(&self) -> Vec<&'static str> {
        todo!()
    }
}
