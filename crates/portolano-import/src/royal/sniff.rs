//! Recognition of the container: plain XML, gzip or zip (SPEC §8.3.1).
//!
//! The extension is not trusted: historically `.rtsx` was plain XML and
//! `.rtsz` compressed, but Royal TSX 6 writes plain XML with both
//! (`docs/royal-format.md` §2). The magic bytes decide.

use std::io::{Cursor, Read};

use flate2::read::GzDecoder;
use zip::ZipArchive;

use crate::ImportError;

const UTF8_BOM: &[u8] = b"\xEF\xBB\xBF";
const GZIP_MAGIC: &[u8] = b"\x1F\x8B";
const ZIP_MAGIC: &[u8] = b"PK";
const ROOT_TAG: &[u8] = b"<RTSZDocument";

/// The container of a Royal document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Container {
    /// Plain XML, with or without a UTF-8 BOM.
    Xml,
    /// Gzip-compressed XML.
    Gzip,
    /// A zip archive holding the XML.
    Zip,
}

/// The container announced by the first bytes, if any.
pub(super) fn container(head: &[u8]) -> Option<Container> {
    if head.starts_with(UTF8_BOM) || head.starts_with(b"<") {
        Some(Container::Xml)
    } else if head.starts_with(GZIP_MAGIC) {
        Some(Container::Gzip)
    } else if head.starts_with(ZIP_MAGIC) {
        Some(Container::Zip)
    } else {
        None
    }
}

/// Whether plain-XML bytes contain the root element of a Royal document.
pub(super) fn mentions_root_element(head: &[u8]) -> bool {
    head.windows(ROOT_TAG.len())
        .any(|window| window == ROOT_TAG)
}

/// The XML text of a Royal document, decompressed if needed and without BOM.
///
/// # Errors
/// [`ImportError::UnrecognizedFormat`] if neither the input nor the
/// decompressed content is XML, [`ImportError::Decompress`] if decompression
/// fails, [`ImportError::InvalidInput`] if a zip has no XML entry or the text
/// is not UTF-8.
pub(super) fn decode(input: &[u8]) -> Result<String, ImportError> {
    let bytes = match container(input).ok_or(ImportError::UnrecognizedFormat)? {
        Container::Xml => input.to_vec(),
        Container::Gzip => gunzip(input)?,
        Container::Zip => unzip(input)?,
    };
    if container(&bytes) != Some(Container::Xml) {
        return Err(ImportError::UnrecognizedFormat);
    }
    let without_bom = bytes.strip_prefix(UTF8_BOM).unwrap_or(&bytes);
    String::from_utf8(without_bom.to_vec()).map_err(|_| ImportError::InvalidInput {
        reason: "il documento non è testo UTF-8".to_owned(),
    })
}

fn gunzip(input: &[u8]) -> Result<Vec<u8>, ImportError> {
    let mut bytes = Vec::new();
    GzDecoder::new(input)
        .read_to_end(&mut bytes)
        .map_err(|error| decompress_error(&error))?;
    Ok(bytes)
}

/// Extracts the first `.xml` or `.rtsx` entry of the archive.
fn unzip(input: &[u8]) -> Result<Vec<u8>, ImportError> {
    let mut archive = ZipArchive::new(Cursor::new(input)).map_err(|e| decompress_error(&e))?;
    let entry_name = archive
        .file_names()
        .filter_map(Result::ok)
        .find(|name| is_xml_entry(name))
        .map(|name| name.into_owned())
        .ok_or_else(|| ImportError::InvalidInput {
            reason: "l'archivio zip non contiene un file .xml o .rtsx".to_owned(),
        })?;
    let mut entry = archive
        .by_name(&entry_name)
        .map_err(|e| decompress_error(&e))?;
    let mut bytes = Vec::new();
    entry
        .read_to_end(&mut bytes)
        .map_err(|error| decompress_error(&error))?;
    Ok(bytes)
}

fn is_xml_entry(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.ends_with(".xml") || lower.ends_with(".rtsx")
}

// The messages of flate2 and zip describe the stream, never its content.
fn decompress_error(error: &dyn std::error::Error) -> ImportError {
    ImportError::Decompress {
        reason: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use flate2::Compression;
    use flate2::write::GzEncoder;
    use pretty_assertions::assert_eq;
    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    use super::*;

    const XML: &str = "<RTSZDocument></RTSZDocument>";

    fn gzip(bytes: &[u8]) -> Vec<u8> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(bytes).unwrap();
        encoder.finish().unwrap()
    }

    fn zip_with(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        for (name, bytes) in entries {
            writer
                .start_file(*name, SimpleFileOptions::default())
                .unwrap();
            writer.write_all(bytes).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    #[test]
    fn container_recognizes_plain_xml() {
        assert_eq!(container(XML.as_bytes()), Some(Container::Xml));
    }

    #[test]
    fn container_recognizes_xml_with_bom() {
        let bytes = [UTF8_BOM, XML.as_bytes()].concat();

        assert_eq!(container(&bytes), Some(Container::Xml));
    }

    #[test]
    fn container_recognizes_gzip_and_zip() {
        assert_eq!(container(&gzip(XML.as_bytes())), Some(Container::Gzip));
        assert_eq!(
            container(&zip_with(&[("doc.xml", XML.as_bytes())])),
            Some(Container::Zip)
        );
    }

    #[test]
    fn container_rejects_unknown_bytes() {
        assert_eq!(container(b"%PDF-1.7"), None);
        assert_eq!(container(b""), None);
    }

    #[test]
    fn decode_strips_the_bom() {
        let bytes = [UTF8_BOM, XML.as_bytes()].concat();

        assert_eq!(decode(&bytes).unwrap(), XML);
    }

    #[test]
    fn decode_decompresses_gzip() {
        let bytes = gzip([UTF8_BOM, XML.as_bytes()].concat().as_slice());

        assert_eq!(decode(&bytes).unwrap(), XML);
    }

    #[test]
    fn decode_extracts_the_first_xml_entry_of_a_zip() {
        let bytes = zip_with(&[
            ("readme.txt", b"not xml".as_slice()),
            ("doc.rtsx", XML.as_bytes()),
        ]);

        assert_eq!(decode(&bytes).unwrap(), XML);
    }

    #[test]
    fn decode_rejects_a_zip_without_xml() {
        let bytes = zip_with(&[("readme.txt", b"not xml".as_slice())]);

        assert!(matches!(
            decode(&bytes),
            Err(ImportError::InvalidInput { .. })
        ));
    }

    #[test]
    fn decode_rejects_an_unknown_format() {
        assert!(matches!(
            decode(b"%PDF-1.7"),
            Err(ImportError::UnrecognizedFormat)
        ));
    }

    #[test]
    fn decode_rejects_compressed_content_that_is_not_xml() {
        assert!(matches!(
            decode(&gzip(b"plain text")),
            Err(ImportError::UnrecognizedFormat)
        ));
    }

    #[test]
    fn decode_reports_a_corrupt_gzip_stream() {
        let mut bytes = gzip(XML.as_bytes());
        bytes.truncate(12);

        assert!(matches!(
            decode(&bytes),
            Err(ImportError::Decompress { .. })
        ));
    }

    #[test]
    fn mentions_root_element_finds_the_royal_root() {
        assert!(mentions_root_element(b"\xEF\xBB\xBF<RTSZDocument>"));
        assert!(!mentions_root_element(b"<html></html>"));
    }
}
