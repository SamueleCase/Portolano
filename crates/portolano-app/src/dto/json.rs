//! The TypeScript type of the JSON values the commands pass as they are:
//! the settings of the protocols and the unmapped fields of an import.

use std::collections::BTreeMap;

use serde::Serialize;

/// A JSON value, for the bindings only: fields of type `serde_json::Value`
/// are exported with this type (`specta(type = …)`). Specta refuses the
/// integers of `serde_json::Number`, which may not fit a JavaScript number;
/// the numbers of settings (ports, sizes, timeouts) always do.
#[derive(Serialize, specta::Type)]
#[serde(untagged)]
#[expect(
    dead_code,
    reason = "never built: it only describes `serde_json::Value` to the TypeScript exporter"
)]
pub enum JsonValue {
    /// `null`.
    Null(()),
    /// `true` or `false`.
    Bool(bool),
    /// A number.
    Number(f64),
    /// A string.
    String(String),
    /// An array.
    Array(Vec<JsonValue>),
    /// An object.
    Object(BTreeMap<String, JsonValue>),
}

/// The settings of a protocol, or the unmapped fields of an import.
pub type JsonObject = BTreeMap<String, JsonValue>;
