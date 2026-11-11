//! Integration tests of the `portolano` binary (SPEC §11.1 WP6): temporary
//! directories, the file vault with the test master password, and probe
//! scripts instead of real clients.

// Test code: clippy.toml allows these in `#[test]` functions only, while the
// shared helpers of an integration test crate must fail loudly too.
#![cfg(unix)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod config;
mod connect;
mod delete;
mod document;
mod edit;
mod import;
mod nodes;
mod secrets;
mod testutil;
mod usage;
mod validate;
