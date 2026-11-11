//! Fields common to every object: description, notes, icon, color,
//! favorite, custom fields and templates (SPEC §8.3.5).

use portolano_core::{CustomProperty, Node, PropertyValue};

use super::context::{ObjectContext, SecretCollector};
use super::{icon, notes};
use crate::royal::xml::RawField;
use crate::{ImportError, PendingIcon, SecretField};

/// Tag of the nodes that were Royal templates.
const TEMPLATE_TAG: &str = "royal-template";
/// Royal has ten numbered custom fields.
const CUSTOM_FIELDS: u8 = 10;
/// [ipo] Name prefix of the protected (encrypted) custom fields.
const PROTECTED_FIELD_PREFIX: &str = "CustomProtectedField";

/// Sets the common fields of `node`.
///
/// # Errors
/// The error of the prompter, asked for the protected custom fields.
pub(super) fn apply(
    ctx: &mut ObjectContext<'_>,
    node: &mut Node,
    secrets: &mut SecretCollector<'_>,
    icons: &mut Vec<PendingIcon>,
) -> Result<(), ImportError> {
    node.description = ctx.fields.take("Description").map(str::to_owned);
    node.favorite = ctx.fields.take_flag("Favorite");
    node.color = ctx.fields.take_with("Color", |value| {
        is_hex_color(value).then(|| value.to_owned())
    });
    node.icon = icon::icon(ctx, icons);
    node.notes = take_notes(ctx);
    node.custom_properties = custom_properties(ctx, secrets)?;
    if ctx.fields.take_flag("IsConnectionTemplate") {
        node.tags.push(TEMPLATE_TAG.to_owned());
        ctx.add_reason("i modelli non sono supportati");
    }
    Ok(())
}

/// HTML notes become plain text; the original stays in `extra`.
fn take_notes(ctx: &mut ObjectContext<'_>) -> Option<String> {
    let field = ctx.fields.take_raw("Notes")?;
    let original = field.as_str().trim();
    if original.is_empty() {
        return None;
    }
    if matches!(field, RawField::Markup(_)) || notes::looks_like_html(original) {
        ctx.keep_original("Notes", original);
        Some(notes::html_to_text(original))
    } else {
        Some(original.to_owned())
    }
}

fn custom_properties(
    ctx: &mut ObjectContext<'_>,
    secrets: &mut SecretCollector<'_>,
) -> Result<Vec<CustomProperty>, ImportError> {
    let mut properties = Vec::new();
    for number in 1..=CUSTOM_FIELDS {
        let name = format!("CustomField{number}");
        if let Some(value) = ctx.fields.take(&name) {
            properties.push(CustomProperty {
                name,
                value: PropertyValue::Text {
                    value: value.to_owned(),
                },
            });
        }
    }
    for number in 1..=CUSTOM_FIELDS {
        let name = format!("{PROTECTED_FIELD_PREFIX}{number}");
        let field = SecretField::CustomProperty(name.clone());
        if let Some(secret) = secrets.request(ctx, &name, field)? {
            properties.push(CustomProperty {
                name,
                value: PropertyValue::Secret { secret },
            });
        }
    }
    Ok(properties)
}

/// `#RRGGBB`, the only color form of the model (SPEC §3).
fn is_hex_color(value: &str) -> bool {
    value
        .strip_prefix('#')
        .is_some_and(|hex| hex.len() == 6 && hex.chars().all(|c| c.is_ascii_hexdigit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_colors_need_six_digits_after_the_hash() {
        assert!(is_hex_color("#1A2b3C"));
        assert!(!is_hex_color("1A2B3C"));
        assert!(!is_hex_color("#FF1A2B3C"));
        assert!(!is_hex_color("#GG0000"));
    }
}
