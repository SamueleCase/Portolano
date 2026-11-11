//! Behaviour of the Royal importer through its public API.

// Test code: clippy.toml allows these in `#[test]` functions only, while the
// shared helpers of an integration test crate must fail loudly too.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common_fields;
mod connections;
mod credentials;
mod encrypted_fields;
mod fixtures;
mod testutil;
mod tree;
