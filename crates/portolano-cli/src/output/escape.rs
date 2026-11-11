//! Characters of a document that must not reach the terminal as they are
//! (DEC-024.3, DEC-026.4). Names, hosts, paths and messages may come from a
//! shared or imported document: control characters would drive the terminal
//! (move the cursor, rewrite lines, change the window title), and the bidi
//! controls would show text in another order than it has.

use std::borrow::Cow;
use std::fmt::Write as _;

/// The bidi controls: embeddings and overrides (U+202A–U+202E), isolates
/// (U+2066–U+2069), the marks (U+200E, U+200F) and the Arabic letter mark
/// (U+061C).
fn is_bidi_control(c: char) -> bool {
    matches!(
        c,
        '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' | '\u{200E}' | '\u{200F}' | '\u{061C}'
    )
}

fn needs_escape(c: char) -> bool {
    c.is_control() || is_bidi_control(c)
}

/// `text` with every control character (C0, DEL, C1) and every bidi control
/// written as an escape such as `\u{1b}`. Every value of a document goes
/// through here before it is printed as text.
pub fn visible(text: &str) -> Cow<'_, str> {
    escape(text, needs_escape, |escaped, c| {
        // Writing to a `String` cannot fail.
        let _ = write!(escaped, "{}", c.escape_unicode());
    })
}

/// JSON text with DEL, the C1 controls and the bidi controls written as
/// JSON escapes (`\u007f`): `serde_json` escapes only U+0000–U+001F, and
/// these characters can only appear inside strings, so the result is the
/// same JSON value. The line breaks of the pretty layout stay as they are.
pub fn json_visible(json: &str) -> Cow<'_, str> {
    let unescaped =
        |c: char| c == '\u{7f}' || ('\u{80}'..='\u{9f}').contains(&c) || is_bidi_control(c);
    escape(json, unescaped, |escaped, c| {
        // Writing to a `String` cannot fail.
        let _ = write!(escaped, "\\u{:04x}", u32::from(c));
    })
}

fn escape(
    text: &str,
    needs: impl Fn(char) -> bool,
    write_escape: impl Fn(&mut String, char),
) -> Cow<'_, str> {
    if !text.chars().any(&needs) {
        return Cow::Borrowed(text);
    }
    let mut escaped = String::with_capacity(text.len() + 8);
    for c in text.chars() {
        if needs(c) {
            write_escape(&mut escaped, c);
        } else {
            escaped.push(c);
        }
    }
    Cow::Owned(escaped)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn visible_escapes_control_characters() {
        assert_eq!(
            visible("web\u{1b}[2J01\r\n\u{7}\u{7f}\u{9b}"),
            "web\\u{1b}[2J01\\u{d}\\u{a}\\u{7}\\u{7f}\\u{9b}"
        );
    }

    #[test]
    fn visible_escapes_bidi_controls() {
        assert_eq!(
            visible("a\u{202e}b\u{2066}c\u{200f}d\u{61c}e"),
            "a\\u{202e}b\\u{2066}c\\u{200f}d\\u{61c}e"
        );
    }

    #[test]
    fn visible_keeps_other_text_as_it_is() {
        assert!(matches!(
            visible("📁 Clienti/ACME àè"),
            Cow::Borrowed("📁 Clienti/ACME àè")
        ));
    }

    #[test]
    fn json_visible_escapes_what_serde_json_leaves_and_keeps_the_value() {
        let value = serde_json::json!({ "name": "a\u{1b}b\u{7f}c\u{9b}d\u{202e}e" });
        let json = serde_json::to_string_pretty(&value).unwrap();

        let shown = json_visible(&json);

        assert!(
            !shown.chars().any(|c| c != '\n' && needs_escape(c)),
            "{shown}"
        );
        assert!(
            shown.contains("\\u007f") && shown.contains("\\u009b") && shown.contains("\\u202e")
        );
        let back: serde_json::Value = serde_json::from_str(&shown).unwrap();
        assert_eq!(back, value);
    }
}
