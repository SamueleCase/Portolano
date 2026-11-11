//! Schema migrations of the document JSON (SPEC §6.3).

// Stub (WP0): the parameters are used once WP2 implements the body.
#![expect(unused_variables)]

use serde_json::Value;

use crate::error::Result;

/// A migration from schema version `n` to `n + 1`. It also updates
/// `schema_version` in the returned value.
pub type Migration = fn(Value) -> Result<Value>;

/// The migrations of the document format, as `(from, migration)`.
/// Version 1 is the first one, so the table is empty.
pub const MIGRATIONS: &[(u32, Migration)] = &[];

/// Applies, in order, the migrations of `table` that bring `value` from
/// schema version `from` to `to`.
///
/// # Errors
/// [`Error::MissingMigration`](crate::Error::MissingMigration) if a step is
/// missing from the table, or the error returned by a migration.
pub fn migrate(value: Value, from: u32, to: u32, table: &[(u32, Migration)]) -> Result<Value> {
    todo!()
}
