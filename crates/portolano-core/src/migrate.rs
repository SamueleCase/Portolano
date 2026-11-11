//! Schema migrations of the document JSON (SPEC §6.3).

use serde_json::Value;

use crate::error::{Error, Result};

/// A migration from schema version `n` to `n + 1`. It also updates
/// `schema_version` in the returned value.
pub type Migration = fn(Value) -> Result<Value>;

/// The migrations of the document format, as `(from, migration)`.
/// Version 1 is the first one, so the table is empty.
pub const MIGRATIONS: &[(u32, Migration)] = &[];

/// Applies, in order, the migrations of `table` that bring `value` from
/// schema version `from` to `to`. After each step `schema_version` is set to
/// the new version, so a migration that forgets to do it cannot leave the
/// value inconsistent.
///
/// # Errors
/// [`Error::MissingMigration`](crate::Error::MissingMigration) if a step is
/// missing from the table, [`Error::Migration`](crate::Error::Migration) if
/// `to` is older than `from` or the value is not a JSON object, or the error
/// returned by a migration.
pub fn migrate(value: Value, from: u32, to: u32, table: &[(u32, Migration)]) -> Result<Value> {
    if to < from {
        return Err(Error::Migration {
            from,
            reason: format!("la versione di destinazione {to} è precedente: non si torna indietro"),
        });
    }
    (from..to).try_fold(value, |value, version| {
        let migration = table
            .iter()
            .find_map(|&(start, migration)| (start == version).then_some(migration))
            .ok_or(Error::MissingMigration { from: version })?;
        set_schema_version(migration(value)?, version)
    })
}

/// Sets `schema_version` to `from + 1` on the result of the migration that
/// started at `from`.
fn set_schema_version(mut value: Value, from: u32) -> Result<Value> {
    let object = value.as_object_mut().ok_or_else(|| Error::Migration {
        from,
        reason: "il risultato non è un oggetto JSON".to_owned(),
    })?;
    object.insert("schema_version".to_owned(), Value::from(from + 1));
    Ok(value)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    /// v1 → v2: the field `host` is renamed `address`.
    fn rename_host(mut value: Value) -> Result<Value> {
        if let Some(object) = value.as_object_mut()
            && let Some(host) = object.remove("host")
        {
            object.insert("address".to_owned(), host);
        }
        Ok(value)
    }

    /// v2 → v3: `port` is added next to `address`; it needs the v2 shape.
    fn add_port(mut value: Value) -> Result<Value> {
        let object = value.as_object_mut().ok_or(Error::Migration {
            from: 2,
            reason: "non è un oggetto".to_owned(),
        })?;
        if !object.contains_key("address") {
            return Err(Error::Migration {
                from: 2,
                reason: "manca address".to_owned(),
            });
        }
        object.insert("port".to_owned(), json!(22));
        Ok(value)
    }

    fn failing(_: Value) -> Result<Value> {
        Err(Error::Migration {
            from: 1,
            reason: "guasto simulato".to_owned(),
        })
    }

    fn returns_array(_: Value) -> Result<Value> {
        Ok(json!([]))
    }

    // Listed out of order on purpose: steps are looked up by version.
    const SYNTHETIC: &[(u32, Migration)] = &[(2, add_port), (1, rename_host)];

    fn v1() -> Value {
        json!({ "schema_version": 1, "host": "web01.example.com" })
    }

    #[test]
    fn migrations_are_applied_in_version_order() {
        let migrated = migrate(v1(), 1, 3, SYNTHETIC).unwrap();

        assert_eq!(
            migrated,
            json!({ "schema_version": 3, "address": "web01.example.com", "port": 22 })
        );
    }

    #[test]
    fn a_single_step_stops_at_the_target_version() {
        let migrated = migrate(v1(), 1, 2, SYNTHETIC).unwrap();

        assert_eq!(
            migrated,
            json!({ "schema_version": 2, "address": "web01.example.com" })
        );
    }

    #[test]
    fn same_version_returns_the_value_unchanged() {
        let migrated = migrate(v1(), 1, 1, &[]).unwrap();

        assert_eq!(migrated, v1());
    }

    #[test]
    fn a_missing_step_is_reported_with_its_version() {
        let table: &[(u32, Migration)] = &[(1, rename_host)];

        let error = migrate(v1(), 1, 3, table).unwrap_err();

        assert!(matches!(error, Error::MissingMigration { from: 2 }));
    }

    #[test]
    fn a_failing_migration_stops_the_chain() {
        let table: &[(u32, Migration)] = &[(1, failing), (2, add_port)];

        let error = migrate(v1(), 1, 3, table).unwrap_err();

        assert!(matches!(error, Error::Migration { from: 1, .. }));
    }

    #[test]
    fn a_migration_returning_a_non_object_is_rejected() {
        let table: &[(u32, Migration)] = &[(1, returns_array)];

        let error = migrate(v1(), 1, 2, table).unwrap_err();

        assert!(matches!(error, Error::Migration { from: 1, .. }));
    }

    #[test]
    fn migrating_to_an_older_version_is_rejected() {
        let error = migrate(v1(), 3, 1, SYNTHETIC).unwrap_err();

        assert!(matches!(error, Error::Migration { from: 3, .. }));
    }
}
