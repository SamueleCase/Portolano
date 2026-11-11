//! Output formatting: tree, tables, details and JSON (SPEC §10.2).

mod escape;
mod report;
mod show;
mod tree;

use anyhow::Result;
use serde::Serialize;
use uuid::Uuid;

pub use escape::visible;
pub use report::print_import;
pub use show::print_show;
pub use tree::{Header, render_tree};

/// Hex digits of the abbreviated ids shown by `tree --ids`; at least the 6
/// that `@<prefix>` paths need.
const SHORT_ID_DIGITS: usize = 8;

/// The first hex digits of `id`, usable as `@<prefix>`.
pub fn short_id(id: Uuid) -> String {
    id.simple().to_string()[..SHORT_ID_DIGITS].to_owned()
}

/// Prints `value` as pretty JSON on stdout. `serde_json` escapes only the
/// C0 controls: DEL, the C1 controls and the bidi controls are escaped here
/// (DEC-026.4).
///
/// # Errors
/// Serialization errors.
pub fn print_json(value: &impl Serialize) -> Result<()> {
    let json = serde_json::to_string_pretty(value)?;
    println!("{}", escape::json_visible(&json));
    Ok(())
}

/// Prints rows as a table with a header, columns separated by two spaces;
/// the cells are made [`visible`].
pub fn print_table(header: &[&str], rows: &[Vec<String>]) {
    let rows: Vec<Vec<String>> = rows
        .iter()
        .map(|row| row.iter().map(|cell| visible(cell).into_owned()).collect())
        .collect();
    let mut widths: Vec<usize> = header.iter().map(|title| display_width(title)).collect();
    for row in &rows {
        for (width, cell) in widths.iter_mut().zip(row) {
            *width = (*width).max(display_width(cell));
        }
    }
    let header: Vec<String> = header.iter().map(|title| (*title).to_owned()).collect();
    for row in std::iter::once(&header).chain(&rows) {
        let cells: Vec<String> = row
            .iter()
            .zip(&widths)
            .map(|(cell, width)| pad(cell, *width))
            .collect();
        println!("{}", cells.join("  ").trim_end());
    }
}

/// `text` followed by spaces up to `width` columns.
pub fn pad(text: &str, width: usize) -> String {
    let padding = width.saturating_sub(display_width(text));
    format!("{text}{}", " ".repeat(padding))
}

/// Columns taken by `text` on a terminal: wide characters (CJK, emoji)
/// take two. An approximation that is enough to align the tree icons.
pub fn display_width(text: &str) -> usize {
    text.chars().map(char_width).sum()
}

fn char_width(c: char) -> usize {
    match u32::from(c) {
        0x1100..=0x115F
        | 0x2E80..=0xA4CF
        | 0xAC00..=0xD7A3
        | 0xF900..=0xFAFF
        | 0xFE30..=0xFE4F
        | 0xFF00..=0xFF60
        | 0xFFE0..=0xFFE6
        | 0x1F300..=0x1FAFF
        | 0x20000..=0x3FFFD => 2,
        _ => 1,
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn short_id_is_a_prefix_of_the_id() {
        let id = Uuid::parse_str("6f1c2d3e-0000-4000-8000-000000000001").unwrap();
        assert_eq!(short_id(id), "6f1c2d3e");
    }

    #[test]
    fn emoji_take_two_columns() {
        assert_eq!(display_width("📁 Clienti"), 10);
        assert_eq!(pad("🔑", 4), "🔑  ");
    }
}
