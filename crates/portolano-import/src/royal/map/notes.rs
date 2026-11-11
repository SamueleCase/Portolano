//! Conversion of HTML notes to plain text (SPEC §8.3.5).
//!
//! Notes are short, user-written documents: a small tag scanner is enough.
//! Block elements become line breaks, list items get a dash, scripts and
//! styles are dropped and entities are decoded.

/// Elements that start a new line (`br` and `li` are handled apart).
const BLOCK_ELEMENTS: &[&str] = &[
    "p",
    "div",
    "tr",
    "ul",
    "ol",
    "table",
    "blockquote",
    "pre",
    "hr",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
];
/// Elements whose content is not text.
const HIDDEN_ELEMENTS: [&str; 3] = ["script", "style", "head"];

/// Whether the text contains an HTML tag: `<` followed by a letter, `/` or
/// `!`, and a `>` later on.
pub(super) fn looks_like_html(text: &str) -> bool {
    text.match_indices('<').any(|(index, _)| {
        let rest = &text[index + 1..];
        rest.starts_with(|c: char| c.is_ascii_alphabetic() || c == '/' || c == '!')
            && rest.contains('>')
    })
}

/// The text of an HTML fragment.
pub(super) fn html_to_text(html: &str) -> String {
    let mut text = String::new();
    let mut hidden: Option<String> = None;
    let mut rest = html;
    while let Some(start) = rest.find('<') {
        if hidden.is_none() {
            push_text(&mut text, &rest[..start]);
        }
        let tag_and_after = &rest[start..];
        let (tag, after) = if let Some(comment) = tag_and_after.strip_prefix("<!--") {
            ("", comment.split_once("-->").map_or("", |(_, after)| after))
        } else if let Some((tag, after)) = tag_and_after[1..].split_once('>') {
            (tag, after)
        } else {
            // A lone `<` is text.
            push_text(&mut text, tag_and_after);
            ("", "")
        };
        handle_tag(&mut text, &mut hidden, tag);
        rest = after;
    }
    if hidden.is_none() {
        push_text(&mut text, rest);
    }
    tidy(&text)
}

fn handle_tag(text: &mut String, hidden: &mut Option<String>, tag: &str) {
    let closing = tag.starts_with('/');
    let name: String = tag
        .trim_start_matches('/')
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase();
    if let Some(open) = hidden.as_deref() {
        if closing && name == open {
            *hidden = None;
        }
        return;
    }
    if !closing && HIDDEN_ELEMENTS.contains(&name.as_str()) {
        *hidden = Some(name);
    } else if name == "br" {
        text.push('\n');
    } else if name == "li" && !closing {
        start_line(text);
        text.push_str("- ");
    } else if name == "li" || BLOCK_ELEMENTS.contains(&name.as_str()) {
        start_line(text);
    }
}

/// Starts a new line unless the text is already at the start of one, so
/// that nested and adjacent blocks do not stack empty lines.
fn start_line(text: &mut String) {
    text.truncate(text.trim_end_matches(' ').len());
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
}

/// Appends HTML text: entities decoded, whitespace collapsed as a browser
/// would.
fn push_text(text: &mut String, html: &str) {
    let decoded = decode_entities(html);
    let mut previous_space = text.ends_with([' ', '\n']);
    for c in decoded.chars() {
        if c.is_whitespace() && c != '\u{a0}' {
            if !previous_space {
                text.push(' ');
            }
            previous_space = true;
        } else {
            text.push(if c == '\u{a0}' { ' ' } else { c });
            previous_space = false;
        }
    }
}

fn decode_entities(html: &str) -> String {
    let mut decoded = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(start) = rest.find('&') {
        decoded.push_str(&rest[..start]);
        let candidate = &rest[start + 1..];
        let entity = candidate
            .find(';')
            .filter(|end| *end <= 10)
            .and_then(|end| Some((decode_entity(&candidate[..end])?, end)));
        match entity {
            Some((c, end)) => {
                decoded.push(c);
                rest = &candidate[end + 1..];
            }
            None => {
                decoded.push('&');
                rest = candidate;
            }
        }
    }
    decoded.push_str(rest);
    decoded
}

fn decode_entity(name: &str) -> Option<char> {
    match name {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        "nbsp" => Some('\u{a0}'),
        _ => {
            let number = name.strip_prefix('#')?;
            let code = match number.strip_prefix(['x', 'X']) {
                Some(hex) => u32::from_str_radix(hex, 16).ok()?,
                None => number.parse().ok()?,
            };
            char::from_u32(code)
        }
    }
}

/// Trims every line and keeps at most one empty line in a row.
fn tidy(text: &str) -> String {
    let mut lines: Vec<&str> = Vec::new();
    for line in text.lines().map(str::trim) {
        if !(line.is_empty() && lines.last().is_none_or(|last| last.is_empty())) {
            lines.push(line);
        }
    }
    while lines.last().is_some_and(|last| last.is_empty()) {
        lines.pop();
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn looks_like_html_needs_a_tag() {
        assert!(looks_like_html("<p>ciao</p>"));
        assert!(looks_like_html("riga<br/>riga"));
        assert!(!looks_like_html("a < b e c > d"));
        assert!(!looks_like_html("testo semplice"));
    }

    #[test]
    fn blocks_become_lines_and_inline_tags_disappear() {
        let html = "<html><body><p>Uno <b>due</b></p><div>tre<br>quattro</div></body></html>";

        assert_eq!(html_to_text(html), "Uno due\ntre\nquattro");
    }

    #[test]
    fn list_items_get_a_dash() {
        assert_eq!(
            html_to_text("<ul><li>uno</li><li>due</li></ul>"),
            "- uno\n- due"
        );
    }

    #[test]
    fn scripts_styles_and_comments_are_dropped() {
        let html = "<head><title>t</title><style>p{}</style></head><!-- c --><p>testo</p>";

        assert_eq!(html_to_text(html), "testo");
    }

    #[test]
    fn entities_are_decoded() {
        assert_eq!(
            html_to_text("<p>A &amp; B &lt;C&gt; &#233;&#x41;&nbsp;fine &unknown;</p>"),
            "A & B <C> éA fine &unknown;"
        );
    }

    #[test]
    fn source_whitespace_is_collapsed() {
        assert_eq!(
            html_to_text("<p>uno\n   due</p>\n\n\n<p>tre</p>"),
            "uno due\ntre"
        );
    }
}
