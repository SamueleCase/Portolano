//! Access to the fields of a raw object, recording which ones the mapping
//! used: the others end up in `extra` and in the report (SPEC §8.3.5).

use std::collections::BTreeSet;
use std::str::FromStr;

use chrono::{DateTime, NaiveDateTime, Utc};

use super::xml::{RawField, RawObject};

/// Royal timestamps: `MM/dd/yyyy HH:mm:ss.fff`, without time zone.
const DATE_FORMAT: &str = "%m/%d/%Y %H:%M:%S%.f";

/// The fields of one object, with the set of fields already used.
pub(super) struct Fields<'a> {
    object: &'a RawObject,
    used: BTreeSet<&'a str>,
}

impl<'a> Fields<'a> {
    pub(super) fn new(object: &'a RawObject) -> Self {
        Self {
            object,
            used: BTreeSet::new(),
        }
    }

    /// The raw field, marking it as used.
    pub(super) fn take_raw(&mut self, name: &str) -> Option<&'a RawField> {
        let (key, value) = self.object.fields.get_key_value(name)?;
        self.used.insert(key.as_str());
        Some(value)
    }

    /// The trimmed value, `None` if absent or blank; the field is marked as
    /// used either way.
    pub(super) fn take(&mut self, name: &str) -> Option<&'a str> {
        self.take_raw(name)
            .map(|field| field.as_str().trim())
            .filter(|value| !value.is_empty())
    }

    /// Whether the field is `True`; Royal omits flags that are `False`.
    pub(super) fn take_flag(&mut self, name: &str) -> bool {
        self.take(name).is_some_and(is_true)
    }

    /// The value converted by `convert`. A blank field is marked as used; a
    /// value that `convert` rejects is left unused, so it is preserved in
    /// `extra` instead of being lost.
    pub(super) fn take_with<T>(
        &mut self,
        name: &str,
        convert: impl FnOnce(&'a str) -> Option<T>,
    ) -> Option<T> {
        let Some(value) = self.peek(name) else {
            self.take_raw(name);
            return None;
        };
        let converted = convert(value)?;
        self.take_raw(name);
        Some(converted)
    }

    /// The value parsed with [`FromStr`], with the rules of [`Fields::take_with`].
    pub(super) fn take_parsed<T: FromStr>(&mut self, name: &str) -> Option<T> {
        self.take_with(name, |value| value.parse().ok())
    }

    /// The trimmed, non-blank value, without marking it as used.
    pub(super) fn peek(&self, name: &str) -> Option<&'a str> {
        self.object.value(name)
    }

    /// Whether the field is `True`, without marking it as used.
    pub(super) fn peek_flag(&self, name: &str) -> bool {
        self.peek(name).is_some_and(is_true)
    }

    /// The fields not used by the mapping, in name order.
    pub(super) fn unused(&self) -> impl Iterator<Item = (&'a str, &'a RawField)> + '_ {
        self.object
            .fields
            .iter()
            .filter(|(name, _)| !self.used.contains(name.as_str()))
            .map(|(name, field)| (name.as_str(), field))
    }
}

/// Royal booleans are `True` / `False`.
pub(super) fn is_true(value: &str) -> bool {
    value.eq_ignore_ascii_case("true")
}

/// A Royal timestamp, read as UTC (SPEC §8.3.5).
pub(super) fn parse_date(value: &str) -> Option<DateTime<Utc>> {
    NaiveDateTime::parse_from_str(value, DATE_FORMAT)
        .ok()
        .map(|naive| naive.and_utc())
}

/// Whether a field holds an encrypted secret, which must never be kept
/// anywhere (D3).
///
/// The names of some secret fields are hypotheses (`docs/royal-format.md`
/// §5), so any field whose name mentions a password, passphrase, secret or
/// embedded key counts, unless its value is blank, a boolean or a number:
/// those are settings such as `CredentialPasswordFromParent`, never
/// ciphertext.
pub(super) fn is_secret_value(name: &str, value: &str) -> bool {
    is_secret_name(name) && !is_plain_setting(value.trim())
}

fn is_secret_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    let embedded_key = lower.contains("privatekey")
        && !matches!(lower.as_str(), "privatekeypath" | "privatekeymode");
    ["password", "passphrase", "secret"]
        .iter()
        .any(|word| lower.contains(word))
        || embedded_key
        || lower.starts_with("customprotectedfield")
}

fn is_plain_setting(value: &str) -> bool {
    value.is_empty()
        || value.eq_ignore_ascii_case("true")
        || value.eq_ignore_ascii_case("false")
        || value.parse::<i64>().is_ok()
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn parse_date_reads_the_royal_format_as_utc() {
        let parsed = parse_date("11/10/2026 09:05:07.123").unwrap();

        let expected = Utc.with_ymd_and_hms(2026, 11, 10, 9, 5, 7).unwrap()
            + chrono::Duration::milliseconds(123);
        assert_eq!(parsed, expected);
    }

    #[test]
    fn parse_date_accepts_a_missing_fraction() {
        let parsed = parse_date("01/02/2026 00:00:00").unwrap();

        assert_eq!(parsed, Utc.with_ymd_and_hms(2026, 1, 2, 0, 0, 0).unwrap());
    }

    #[test]
    fn parse_date_rejects_other_formats() {
        assert_eq!(parse_date("2026-11-10T09:05:07Z"), None);
    }

    #[test]
    fn encrypted_fields_are_secret() {
        let ciphertext = "c2VjcmV0LWJ1dC1ub3QtcmVhbA==";

        assert!(is_secret_value("Password", ciphertext));
        assert!(is_secret_value("CredentialPassword", ciphertext));
        assert!(is_secret_value("PasswordHash", ciphertext));
        assert!(is_secret_value("Passphrase", ciphertext));
        assert!(is_secret_value("PrivateKeyContent", ciphertext));
        assert!(is_secret_value("CustomProtectedField3", ciphertext));
    }

    #[test]
    fn settings_with_secret_names_are_not_secret() {
        assert!(!is_secret_value("CredentialPasswordFromParent", "True"));
        assert!(!is_secret_value("PasswordMode", "2"));
        assert!(!is_secret_value("Password", "  "));
        assert!(!is_secret_value("PrivateKeyPath", "~/.ssh/id_ed25519"));
        assert!(!is_secret_value("PrivateKeyMode", "1"));
    }

    #[test]
    fn ordinary_fields_are_not_secret() {
        assert!(!is_secret_value("UserName", "admin"));
        assert!(!is_secret_value("URI", "web01.example.com"));
    }
}
