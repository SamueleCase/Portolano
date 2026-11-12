//! The accepted image types (SPEC §6.4), recognized from their bytes.

/// The accepted image types (SPEC §6.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ImageType {
    Png,
    Jpeg,
    Gif,
    Tiff,
    Svg,
    Pdf,
}

/// Leading bytes of the binary types. SVG is text and has none.
const SIGNATURES: &[(&[u8], ImageType)] = &[
    (b"\x89PNG\r\n\x1a\n", ImageType::Png),
    (b"\xff\xd8\xff", ImageType::Jpeg),
    (b"GIF87a", ImageType::Gif),
    (b"GIF89a", ImageType::Gif),
    (b"II*\0", ImageType::Tiff),
    (b"MM\0*", ImageType::Tiff),
    (b"%PDF-", ImageType::Pdf),
];

/// How far into an SVG file the `<svg` element is looked for: enough for an
/// XML declaration, a DOCTYPE and the comments that editors put first.
const SVG_HEAD_LEN: usize = 4096;

impl ImageType {
    const ALL: [ImageType; 6] = [
        ImageType::Png,
        ImageType::Jpeg,
        ImageType::Gif,
        ImageType::Tiff,
        ImageType::Svg,
        ImageType::Pdf,
    ];

    /// The type of `bytes`, from their magic bytes.
    pub(super) fn detect(bytes: &[u8]) -> Option<ImageType> {
        SIGNATURES
            .iter()
            .find(|(magic, _)| bytes.starts_with(magic))
            .map(|&(_, image)| image)
            .or_else(|| looks_like_svg(bytes).then_some(ImageType::Svg))
    }

    pub(super) fn from_mime(mime: &str) -> Option<ImageType> {
        Self::ALL.into_iter().find(|image| image.mime() == mime)
    }

    pub(super) fn from_extension(extension: &str) -> Option<ImageType> {
        Self::ALL
            .into_iter()
            .find(|image| image.extension() == extension)
    }

    pub(super) fn mime(self) -> &'static str {
        match self {
            ImageType::Png => "image/png",
            ImageType::Jpeg => "image/jpeg",
            ImageType::Gif => "image/gif",
            ImageType::Tiff => "image/tiff",
            ImageType::Svg => "image/svg+xml",
            ImageType::Pdf => "application/pdf",
        }
    }

    pub(super) fn extension(self) -> &'static str {
        match self {
            ImageType::Png => "png",
            ImageType::Jpeg => "jpg",
            ImageType::Gif => "gif",
            ImageType::Tiff => "tiff",
            ImageType::Svg => "svg",
            ImageType::Pdf => "pdf",
        }
    }

    pub(super) fn file_name(self, sha256: &str) -> String {
        format!("{sha256}.{}", self.extension())
    }
}

/// SVG has no binary signature: it is XML whose root element is `<svg`.
/// The byte order mark, the XML declaration (and other processing
/// instructions), comments and the DOCTYPE may come before the root.
fn looks_like_svg(bytes: &[u8]) -> bool {
    let head = &bytes[..bytes.len().min(SVG_HEAD_LEN)];
    let mut rest = head
        .strip_prefix(b"\xef\xbb\xbf")
        .unwrap_or(head)
        .trim_ascii_start();
    while let Some(len) = prolog_item_len(rest) {
        rest = rest[len..].trim_ascii_start();
    }
    rest.strip_prefix(b"<svg").is_some_and(|after| {
        matches!(
            after.first(),
            Some(b'>' | b'/' | b' ' | b'\t' | b'\r' | b'\n')
        )
    })
}

/// The length of the XML prolog item that `rest` starts with, or `None` if
/// it starts with something else or the item is not terminated in the head.
fn prolog_item_len(rest: &[u8]) -> Option<usize> {
    if rest.starts_with(b"<?") {
        end_of(rest, b"?>")
    } else if rest.starts_with(b"<!--") {
        end_of(rest, b"-->")
    } else if rest.starts_with(b"<!DOCTYPE") {
        // An internal subset, `[ ... ]`, may contain `>` itself.
        let first = rest.iter().position(|&b| b == b'[' || b == b'>')?;
        if rest[first] == b'>' {
            return Some(first + 1);
        }
        let subset_end = first + end_of(&rest[first..], b"]")?;
        Some(subset_end + end_of(&rest[subset_end..], b">")?)
    } else {
        None
    }
}

/// The index just past the first `marker` in `haystack`.
fn end_of(haystack: &[u8], marker: &[u8]) -> Option<usize> {
    haystack
        .windows(marker.len())
        .position(|window| window == marker)
        .map(|start| start + marker.len())
}
