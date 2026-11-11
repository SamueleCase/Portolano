//! Decoding of protocol settings and protocol configuration sections into
//! their typed structs, with error messages in Italian (SPEC §7.1, DEC-003).
//!
//! serde reports errors in English and `serde_json` does not say which field
//! failed, so the field is pinned here and the common serde messages are
//! translated.

use portolano_core::AppConfig;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

use crate::ConnectError;

/// Decodes the `settings` of a connection into `T`, whose fields must all
/// have defaults (`#[serde(default)]`).
///
/// # Errors
/// A description of the first invalid field, in Italian.
pub(crate) fn decode_settings<T: DeserializeOwned>(
    settings: &Map<String, Value>,
) -> Result<T, String> {
    // Decoding one field at a time pins the error to the field that caused it.
    for (key, value) in settings {
        let single = Map::from_iter([(key.clone(), value.clone())]);
        if let Err(error) = serde_json::from_value::<T>(Value::Object(single)) {
            return Err(field_error(key, &error.to_string()));
        }
    }
    serde_json::from_value(Value::Object(settings.clone()))
        .map_err(|error| translate(&error.to_string()))
}

/// Reads the section `name` of `config` into `T`; defaults if it is missing.
///
/// # Errors
/// [`ConnectError::InvalidConfig`] if the section does not match `T`.
pub(crate) fn decode_config_section<T: DeserializeOwned + Default>(
    config: &AppConfig,
    name: &str,
) -> Result<T, ConnectError> {
    config.protocols.get(name).map_or_else(
        || Ok(T::default()),
        |section| {
            decode_section(section).map_err(|reason| ConnectError::InvalidConfig {
                section: name.to_owned(),
                reason,
            })
        },
    )
}

/// Decodes a TOML section of `config.toml` into `T`.
///
/// # Errors
/// A description of the problem, in Italian.
pub(crate) fn decode_section<T: DeserializeOwned>(section: &toml::Value) -> Result<T, String> {
    // Going through JSON reuses the per-field error reporting of
    // `decode_settings`; the sections only hold strings and tables.
    match serde_json::to_value(section) {
        Ok(Value::Object(map)) => decode_settings(&map),
        Ok(_) => Err("la sezione deve essere una tabella".to_owned()),
        Err(error) => Err(translate(&error.to_string())),
    }
}

fn field_error(key: &str, message: &str) -> String {
    let translated = translate(message);
    // The key itself is unknown, rather than a field nested in its value.
    if message.starts_with("unknown field") && backticked(message).first() == Some(&key) {
        translated
    } else {
        format!("campo '{key}': {translated}")
    }
}

/// Translates the messages of serde's derive and of `serde_json` into
/// Italian; unknown messages are kept, prefixed by a generic description.
fn translate(message: &str) -> String {
    let names = backticked(message);
    if message.starts_with("unknown field") {
        let field = names.first().copied().unwrap_or_default();
        return format!("campo sconosciuto '{field}'{}", allowed(names.get(1..)));
    }
    if message.starts_with("unknown variant") {
        let variant = names.first().copied().unwrap_or_default();
        return format!("valore sconosciuto '{variant}'{}", allowed(names.get(1..)));
    }
    if message.starts_with("missing field") {
        let field = names.first().copied().unwrap_or_default();
        return format!("campo obbligatorio mancante '{field}'");
    }
    if let Some(rest) = message.strip_prefix("invalid type: ")
        && let Some((found, expected)) = rest.split_once(", expected ")
    {
        return format!(
            "tipo non valido: trovato {}, atteso {}",
            describe(found),
            describe(expected)
        );
    }
    if let Some(rest) = message.strip_prefix("invalid value: ")
        && let Some((found, expected)) = rest.split_once(", expected ")
    {
        return format!(
            "valore non valido {}, atteso {}",
            describe(found),
            describe(expected)
        );
    }
    if message.starts_with("data did not match any variant of untagged enum") {
        return "formato non riconosciuto".to_owned();
    }
    format!("valore non valido ({message})")
}

/// The words quoted with backticks in a serde message, in order.
fn backticked(message: &str) -> Vec<&str> {
    message.split('`').skip(1).step_by(2).collect()
}

fn allowed(names: Option<&[&str]>) -> String {
    match names {
        Some(names) if !names.is_empty() => format!(" (ammessi: {})", names.join(", ")),
        _ => String::new(),
    }
}

/// Italian wording of a serde type description, such as `a boolean` or
/// ``integer `5` ``.
fn describe(text: &str) -> String {
    let value = backticked(text).first().map(|value| format!(" {value}"));
    let value = value.unwrap_or_default();
    let words = [
        ("string ", "la stringa "),
        ("integer", "il numero"),
        ("floating point", "il numero"),
        ("boolean", "il booleano"),
    ];
    for (english, italian) in words {
        if let Some(rest) = text.strip_prefix(english) {
            return if english == "string " {
                format!("{italian}{rest}")
            } else {
                format!("{italian}{value}")
            };
        }
    }
    let kind = match text {
        "a boolean" => "un booleano",
        "a string" => "una stringa",
        "a sequence" | "sequence" => "una lista",
        "a map" | "map" => "un oggetto",
        "null" | "unit value" => "null",
        "u8" | "u16" | "u32" | "u64" | "i32" | "i64" => "un numero intero",
        _ if text.starts_with("struct ") || text.contains("enum ") => "un oggetto",
        _ => return text.to_owned(),
    };
    kind.to_owned()
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn unknown_field_lists_the_allowed_fields() {
        let message = translate("unknown field `colour`, expected `host` or `port`");

        assert_eq!(message, "campo sconosciuto 'colour' (ammessi: host, port)");
    }

    #[test]
    fn invalid_type_names_the_found_and_expected_kinds() {
        let message = translate("invalid type: string \"yes\", expected a boolean");

        assert_eq!(
            message,
            "tipo non valido: trovato la stringa \"yes\", atteso un booleano"
        );
    }

    #[test]
    fn invalid_integer_value_is_translated() {
        let message = translate("invalid value: integer `70000`, expected u16");

        assert_eq!(
            message,
            "valore non valido il numero 70000, atteso un numero intero"
        );
    }

    #[test]
    fn unknown_message_is_kept() {
        let message = translate("something else");

        assert_eq!(message, "valore non valido (something else)");
    }
}
