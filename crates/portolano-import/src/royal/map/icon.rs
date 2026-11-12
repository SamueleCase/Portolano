//! Icons: from the Royal icon library or embedded images (SPEC §8.3.5,
//! `docs/royal-format.md` §6).

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use portolano_core::Icon;
use portolano_core::icons::{MAX_ICON_BYTES, detect_mime};
use sha2::{Digest, Sha256};

use super::context::ObjectContext;
use crate::PendingIcon;

const CUSTOM_IMAGE: &str = "CustomImage";

/// The icon of the object. An embedded image wins over a library icon; the
/// image is returned in `icons` once, however many nodes use it.
pub(super) fn icon(ctx: &mut ObjectContext<'_>, icons: &mut Vec<PendingIcon>) -> Option<Icon> {
    match ctx.fields.take_with(CUSTOM_IMAGE, decode_image) {
        Some(image) => return Some(register(image, icons)),
        None => {
            if let Some(encoded) = ctx.fields.peek(CUSTOM_IMAGE) {
                let reason = if decoded_len(encoded) > MAX_ICON_BYTES {
                    "icona personalizzata oltre 4 MiB: conservata in extra"
                } else {
                    "icona personalizzata in un formato non riconosciuto: conservata in extra"
                };
                ctx.add_reason(reason);
            }
        }
    }
    ctx.fields.take("CustomImageName").map(|name| Icon::Royal {
        name: name.to_owned(),
    })
}

struct Image {
    bytes: Vec<u8>,
    mime: &'static str,
}

/// The image of `encoded`, if it is of an accepted type and not larger than
/// the icon store accepts (`MAX_ICON_BYTES`): a larger one would make the
/// whole import fail when it is stored.
fn decode_image(encoded: &str) -> Option<Image> {
    if decoded_len(encoded) > MAX_ICON_BYTES {
        return None;
    }
    let compact: String = encoded.chars().filter(|c| !c.is_whitespace()).collect();
    let bytes = STANDARD.decode(compact).ok()?;
    let mime = detect_mime(&bytes)?;
    Some(Image { bytes, mime })
}

/// The size of the bytes that the base64 text `encoded` holds, without
/// decoding it.
fn decoded_len(encoded: &str) -> u64 {
    let symbols = encoded.chars().filter(|c| !c.is_whitespace()).count();
    let padding = encoded
        .trim_end()
        .chars()
        .rev()
        .take_while(|&c| c == '=')
        .count();
    u64::try_from((symbols / 4 * 3).saturating_sub(padding)).unwrap_or(u64::MAX)
}

fn register(image: Image, icons: &mut Vec<PendingIcon>) -> Icon {
    let sha256 = hex::encode(Sha256::digest(&image.bytes));
    if !icons.iter().any(|icon| icon.placeholder_sha256 == sha256) {
        icons.push(PendingIcon {
            placeholder_sha256: sha256.clone(),
            bytes: image.bytes,
            mime: image.mime.to_owned(),
        });
    }
    Icon::Custom {
        sha256,
        mime: image.mime.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn images_the_icon_store_would_refuse_are_not_decoded() {
        let mut late_svg = b"<?xml version=\"1.0\"?>".to_vec();
        late_svg.extend(vec![b' '; 5000]);
        late_svg.extend(b"<svg/>");
        for bytes in [&b"%PDF1.7"[..], b"<svgx/>", &late_svg] {
            assert!(decode_image(&STANDARD.encode(bytes)).is_none(), "{bytes:?}");
        }
    }

    #[test]
    fn image_of_an_accepted_type_is_decoded_with_its_type() {
        let image = decode_image(&STANDARD.encode(b"%PDF-1.7")).unwrap();

        assert_eq!(image.mime, "application/pdf");
    }

    #[test]
    fn decoded_length_is_computed_from_the_base64_text() {
        for bytes in [&b""[..], b"a", b"ab", b"abc", b"GIF89a-pixels"] {
            assert_eq!(
                decoded_len(&STANDARD.encode(bytes)),
                u64::try_from(bytes.len()).unwrap()
            );
        }
    }

    #[test]
    fn image_larger_than_the_icon_store_accepts_is_not_decoded() {
        let mut bytes = b"GIF89a".to_vec();
        bytes.resize(usize::try_from(MAX_ICON_BYTES).unwrap() + 1, 0);

        assert!(decode_image(&STANDARD.encode(&bytes)).is_none());
    }

    #[test]
    fn decode_image_ignores_line_breaks_in_base64() {
        let encoded = STANDARD.encode(b"GIF89a-pixels");
        let wrapped = format!("{}\n{}", &encoded[..8], &encoded[8..]);

        let image = decode_image(&wrapped).unwrap();

        assert_eq!(image.bytes, b"GIF89a-pixels");
        assert_eq!(image.mime, "image/gif");
    }
}
