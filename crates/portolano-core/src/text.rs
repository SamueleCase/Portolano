//! Characters of a document that must not reach the terminal, or a dialog,
//! as they are (DEC-024.3, DEC-026.4). Names, hosts, paths and messages may
//! come from a shared or imported document: control characters would drive
//! the terminal (move the cursor, rewrite lines, change the window title),
//! and the bidi controls would show text in another order than it has.
//!
//! The one rule of the CLI and of the native dialogs of the app
//! (DEC-046.13); the interface has its equivalent in TypeScript.

use std::borrow::Cow;
use std::fmt::Write as _;

/// The invisible characters that are not controls: the line and paragraph
/// separators (Zl, Zp) and the format characters (Cf, Unicode 16), which
/// include the bidi controls (embeddings, overrides, isolates, marks, the
/// Arabic letter mark) and the zero-width ones (U+200B–U+200D, U+2060,
/// U+FEFF). They would hide text, break a line or show it in another order
/// than it has (DEC-047.2).
fn is_invisible_format(c: char) -> bool {
    matches!(
        c,
        '\u{2028}'
            | '\u{2029}'
            | '\u{00AD}'
            | '\u{0600}'..='\u{0605}'
            | '\u{061C}'
            | '\u{06DD}'
            | '\u{070F}'
            | '\u{0890}'..='\u{0891}'
            | '\u{08E2}'
            | '\u{180E}'
            | '\u{200B}'..='\u{200F}'
            | '\u{202A}'..='\u{202E}'
            | '\u{2060}'..='\u{2064}'
            | '\u{2066}'..='\u{206F}'
            | '\u{FEFF}'
            | '\u{FFF9}'..='\u{FFFB}'
            | '\u{110BD}'
            | '\u{110CD}'
            | '\u{13430}'..='\u{1343F}'
            | '\u{1BCA0}'..='\u{1BCA3}'
            | '\u{1D173}'..='\u{1D17A}'
            | '\u{E0001}'
            | '\u{E0020}'..='\u{E007F}'
    )
}

/// The zero-width joiner and non-joiner (U+200D, U+200C): format characters
/// that compose emoji (👨‍👩‍👧) and join or keep apart the letters of
/// some scripts. The text of a document keeps them as they are (DEC-047.5).
fn is_joiner(c: char) -> bool {
    matches!(c, '\u{200C}' | '\u{200D}')
}

/// What [`visible_strict`] escapes: every control and invisible character.
fn needs_strict_escape(c: char) -> bool {
    c.is_control() || is_invisible_format(c)
}

/// What [`visible`] escapes: the same, except the joiners.
fn needs_escape(c: char) -> bool {
    needs_strict_escape(c) && !is_joiner(c)
}

/// `text` with every control character (C0, DEL, C1), separator of lines
/// and paragraphs and format character (bidi and zero-width ones included)
/// written as an escape such as `\u{1b}`, except the joiners of
/// [`is_joiner`]. Every value of a document goes through here before it is
/// printed as text.
pub fn visible(text: &str) -> Cow<'_, str> {
    escape(text, needs_escape, write_escape)
}

/// [`visible`] without the exception of the joiners: for a text that must
/// show every character it has, such as a command to confirm before it
/// runs (DEC-047.5).
pub fn visible_strict(text: &str) -> Cow<'_, str> {
    escape(text, needs_strict_escape, write_escape)
}

fn write_escape(escaped: &mut String, c: char) {
    // Writing to a `String` cannot fail.
    let _ = write!(escaped, "{}", c.escape_unicode());
}

/// JSON text with DEL, the C1 controls and the characters of
/// [`is_invisible_format`], except the joiners, written as JSON escapes
/// (`\u007f`, a surrogate pair beyond U+FFFF): `serde_json` escapes only
/// U+0000–U+001F, and these characters can only appear inside strings, so
/// the result is the same JSON value. The line breaks of the pretty layout
/// stay as they are.
pub fn json_visible(json: &str) -> Cow<'_, str> {
    let unescaped = |c: char| {
        c == '\u{7f}'
            || ('\u{80}'..='\u{9f}').contains(&c)
            || (is_invisible_format(c) && !is_joiner(c))
    };
    escape(json, unescaped, |escaped, c| {
        for unit in c.encode_utf16(&mut [0; 2]) {
            // Writing to a `String` cannot fail.
            let _ = write!(escaped, "\\u{unit:04x}");
        }
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
    fn visible_escapes_the_separators_of_lines_and_paragraphs() {
        assert_eq!(visible("a\u{2028}b\u{2029}c"), "a\\u{2028}b\\u{2029}c");
    }

    #[test]
    fn visible_escapes_the_zero_width_and_format_characters() {
        assert_eq!(
            visible("w\u{200b}eb\u{2060}0\u{feff}1\u{ad}x\u{e0041}"),
            "w\\u{200b}eb\\u{2060}0\\u{feff}1\\u{ad}x\\u{e0041}"
        );
    }

    #[test]
    fn visible_keeps_the_joiners_of_a_composed_emoji() {
        let family = "Famiglia \u{1f468}\u{200d}\u{1f469}\u{200d}\u{1f467} e\u{200c}x";

        assert!(matches!(visible(family), Cow::Borrowed(text) if text == family));
    }

    #[test]
    fn visible_strict_escapes_the_joiners_too() {
        assert_eq!(
            visible_strict("term\u{200d} {cmd}\u{200c}\u{1b}"),
            "term\\u{200d} {cmd}\\u{200c}\\u{1b}"
        );
    }

    #[test]
    fn json_visible_keeps_the_joiners() {
        let json = serde_json::to_string(&serde_json::json!({ "n": "a\u{200d}b" })).unwrap();

        assert_eq!(json_visible(&json), json);
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

    #[test]
    fn json_visible_writes_a_surrogate_pair_beyond_the_basic_plane() {
        let value = serde_json::json!({ "tag": "a\u{e0041}\u{2028}b" });
        let json = serde_json::to_string(&value).unwrap();

        let shown = json_visible(&json);

        assert!(
            shown.contains("\\udb40\\udc41") && shown.contains("\\u2028"),
            "{shown}"
        );
        let back: serde_json::Value = serde_json::from_str(&shown).unwrap();
        assert_eq!(back, value);
    }
}
