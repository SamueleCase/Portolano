//! Access to the fields of a raw object, recording which ones the mapping
//! used: the others end up in `extra` and in the report (SPEC §8.3.5).

use std::collections::BTreeSet;
use std::str::FromStr;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
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

    /// The raw field, marking it as used. A field holding a secret is never
    /// returned and stays unused, so that it is dropped and reported
    /// (D3); secrets are read with [`Fields::take_secret`].
    pub(super) fn take_raw(&mut self, name: &str) -> Option<&'a RawField> {
        if self.object.is_secret(name) {
            return None;
        }
        let (key, value) = self.object.fields.get_key_value(name)?;
        self.used.insert(key.as_str());
        Some(value)
    }

    /// Whether the field holds an encrypted secret, marking it as used: its
    /// value is never returned, it cannot be decrypted nor kept (D3).
    pub(super) fn take_secret(&mut self, name: &str) -> bool {
        let Some((key, _)) = self.object.fields.get_key_value(name) else {
            return false;
        };
        self.used.insert(key.as_str());
        self.object.is_secret(name)
    }

    /// Whether the field holds an encrypted secret.
    pub(super) fn is_secret(&self, name: &str) -> bool {
        self.object.is_secret(name)
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
    // A plain setting is at most an `i64` (20 characters) with a little
    // whitespace: longer text counts as secret without trimming it, so the
    // check is O(1) even when the parser asks it for every level of deeply
    // nested elements over the same long text. Erring here drops a field,
    // the safe direction.
    is_secret_name(name)
        && (value.len() > MAX_PLAIN_SETTING_LENGTH || !is_plain_setting(value.trim()))
}

/// Longest text that may still be a plain setting, whitespace included.
const MAX_PLAIN_SETTING_LENGTH: usize = 64;

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

/// Fields that hold base64 or long free text by design: the notes and the
/// images. They are never taken for ciphertext by their value.
const FREE_FORM_FIELDS: [&str; 3] = ["Notes", "CustomImage", "CustomImageList"];

/// Royal encrypts with a 16-byte block cipher.
const CIPHER_BLOCK_BYTES: usize = 16;

/// Shortest ciphertext recognized by its value: two blocks, such as an IV
/// and one block of data.
const MIN_CIPHERTEXT_BYTES: usize = 2 * CIPHER_BLOCK_BYTES;

/// Whether the value of `field` looks like Royal ciphertext even though the
/// field name does not say so (DEC-024.4): the encrypted fields of Royal are
/// only partly known, and one with an unexpected name must not end up in
/// `extra`. The value must be canonical base64 that decodes to at least
/// [`MIN_CIPHERTEXT_BYTES`], a multiple of [`CIPHER_BLOCK_BYTES`]. Notes
/// and images are never ciphertext; nor is a value made of hex digits only,
/// such as a hash or a key fingerprint, which base64 would accept too.
pub(super) fn looks_like_ciphertext(field: &str, value: &str) -> bool {
    // A cheap filter first: the parser asks for every field of the document.
    const MIN_ENCODED_LENGTH: usize = MIN_CIPHERTEXT_BYTES.div_ceil(3) * 4;
    let value = value.trim();
    if value.len() < MIN_ENCODED_LENGTH
        || !value.len().is_multiple_of(4)
        || FREE_FORM_FIELDS.contains(&field)
        || value.chars().all(|c| c.is_ascii_hexdigit())
    {
        return false;
    }
    STANDARD.decode(value).is_ok_and(|bytes| {
        bytes.len() >= MIN_CIPHERTEXT_BYTES && bytes.len().is_multiple_of(CIPHER_BLOCK_BYTES)
    })
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
    fn long_text_under_a_secret_name_is_secret_even_if_blank() {
        assert!(is_secret_value("Password", &" ".repeat(65)));
        assert!(!is_secret_value("Password", &format!("{:>64}", "True")));
    }

    #[test]
    fn ordinary_fields_are_not_secret() {
        assert!(!is_secret_value("UserName", "admin"));
        assert!(!is_secret_value("URI", "web01.example.com"));
    }

    /// Base64 of `len` bytes that are neither text nor hex.
    fn ciphertext(len: usize) -> String {
        let bytes: Vec<u8> = (0..len).map(|i| (i * 37 % 251) as u8 | 0x80).collect();
        STANDARD.encode(bytes)
    }

    #[test]
    fn block_aligned_base64_looks_like_ciphertext_under_any_name() {
        for len in [32, 48, 64, 1024] {
            assert!(
                looks_like_ciphertext("SomeUnknownField", &ciphertext(len)),
                "{len}"
            );
        }
        assert!(looks_like_ciphertext(
            "Description",
            &format!("  {}\n", ciphertext(32))
        ));
    }

    #[test]
    fn short_or_unaligned_base64_is_not_ciphertext() {
        for len in [16, 31, 33, 40, 47] {
            assert!(
                !looks_like_ciphertext("SomeUnknownField", &ciphertext(len)),
                "{len}"
            );
        }
    }

    #[test]
    fn notes_and_images_are_never_ciphertext() {
        for field in FREE_FORM_FIELDS {
            assert!(!looks_like_ciphertext(field, &ciphertext(64)), "{field}");
        }
    }

    #[test]
    fn hex_hashes_and_plain_text_are_not_ciphertext() {
        // 64 hex digits decode as base64 to 48 bytes: a SHA-256 in hex.
        let sha256 = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";
        assert!(!looks_like_ciphertext("Thumbprint", sha256));
        assert!(!looks_like_ciphertext("URI", "web01.example.com"));
        assert!(!looks_like_ciphertext(
            "ID",
            "b0000000-0000-4000-8000-000000000001"
        ));
        assert!(!looks_like_ciphertext(
            "Description",
            "Server web di produzione del cliente ACME, sede di Milano"
        ));
    }
}
