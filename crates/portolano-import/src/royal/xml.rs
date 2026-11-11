//! Parsing of the Royal XML into raw objects (SPEC §8.3.2).
//!
//! Every child of `<RTSZDocument>` is an object; every child of an object is
//! a field (`docs/royal-format.md` §3). No field is interpreted here.

use std::collections::BTreeMap;
use std::ops::Range;

use quick_xml::Reader;
use quick_xml::escape::resolve_predefined_entity;
use quick_xml::events::{BytesRef, BytesStart, Event};

use super::fields::is_secret_value;
use crate::ImportError;

const ROOT_ELEMENT: &str = "RTSZDocument";

/// The value of a field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum RawField {
    /// Text content, unescaped.
    Text(String),
    /// The inner XML, verbatim, of a field with child elements.
    Markup(String),
}

impl RawField {
    pub(super) fn as_str(&self) -> &str {
        match self {
            Self::Text(text) | Self::Markup(text) => text,
        }
    }
}

/// One object of the document, before mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawObject {
    /// Element name, such as `RoyalSSHConnection`.
    pub(super) type_name: String,
    /// Fields by name; for a repeated field the first occurrence wins.
    pub(super) fields: BTreeMap<String, RawField>,
    /// The original XML fragment of the object.
    pub(super) raw_xml: String,
    /// Byte ranges in `raw_xml` of the fields holding encrypted secrets.
    secret_spans: Vec<Range<usize>>,
}

impl RawObject {
    /// `raw_xml` without the elements that hold encrypted secrets: the
    /// ciphertext must not be kept anywhere (D3), not even in the XML of an
    /// unsupported node.
    pub(super) fn redacted_xml(&self) -> String {
        let mut redacted = String::with_capacity(self.raw_xml.len());
        let mut cursor = 0;
        for span in &self.secret_spans {
            redacted.push_str(&self.raw_xml[cursor..span.start]);
            cursor = span.end;
        }
        redacted.push_str(&self.raw_xml[cursor..]);
        redacted
    }

    /// Whether some fields hold encrypted secrets.
    pub(super) fn has_secrets(&self) -> bool {
        !self.secret_spans.is_empty()
    }
}

/// The objects of a Royal document, in file order.
///
/// # Errors
/// [`ImportError::Xml`] if the XML is malformed,
/// [`ImportError::InvalidInput`] if the root element is not `RTSZDocument`.
pub(super) fn parse_document(xml: &str) -> Result<Vec<RawObject>, ImportError> {
    let mut reader = Reader::from_str(xml);
    loop {
        match read(&mut reader)? {
            Event::Start(root) => {
                expect_root(&root)?;
                break;
            }
            Event::Empty(root) => {
                expect_root(&root)?;
                return Ok(Vec::new());
            }
            Event::Eof => {
                return Err(ImportError::InvalidInput {
                    reason: "il documento non contiene elementi XML".to_owned(),
                });
            }
            _ => {}
        }
    }

    let mut objects = Vec::new();
    loop {
        let start = position(&reader)?;
        match read(&mut reader)? {
            Event::Start(element) => {
                let type_name = element_name(&element);
                objects.push(parse_object(&mut reader, xml, type_name, start)?);
            }
            Event::Empty(element) => objects.push(RawObject {
                type_name: element_name(&element),
                fields: BTreeMap::new(),
                raw_xml: xml[start..position(&reader)?].to_owned(),
                secret_spans: Vec::new(),
            }),
            Event::End(_) => return Ok(objects),
            Event::Eof => return Err(unexpected_end()),
            _ => {}
        }
    }
}

/// Reads the fields of an object whose start tag began at byte `start`.
fn parse_object(
    reader: &mut Reader<&[u8]>,
    xml: &str,
    type_name: String,
    start: usize,
) -> Result<RawObject, ImportError> {
    let mut fields = BTreeMap::new();
    let mut secret_spans = Vec::new();
    loop {
        let field_start = position(reader)?;
        let (name, value) = match read(reader)? {
            Event::Start(element) => (element_name(&element), read_field_value(reader, xml)?),
            Event::Empty(element) => (element_name(&element), RawField::Text(String::new())),
            Event::End(_) => {
                return Ok(RawObject {
                    type_name,
                    fields,
                    raw_xml: xml[start..position(reader)?].to_owned(),
                    secret_spans,
                });
            }
            Event::Eof => return Err(unexpected_end()),
            _ => continue,
        };
        if is_secret_value(&name, value.as_str()) {
            secret_spans.push(field_start - start..position(reader)? - start);
        }
        fields.entry(name).or_insert(value);
    }
}

/// Reads the content of a field up to its end tag.
fn read_field_value(reader: &mut Reader<&[u8]>, xml: &str) -> Result<RawField, ImportError> {
    let inner_start = position(reader)?;
    let mut text = String::new();
    let mut depth = 0_usize;
    let mut has_children = false;
    loop {
        let before = position(reader)?;
        match read(reader)? {
            Event::Text(content) => text.push_str(&content.xml10_content()),
            Event::CData(content) => text.push_str(&content),
            Event::GeneralRef(reference) => push_reference(&mut text, &reference),
            Event::Start(_) => {
                depth += 1;
                has_children = true;
            }
            Event::Empty(_) => has_children = true,
            Event::End(_) if depth == 0 => {
                return Ok(if has_children {
                    RawField::Markup(xml[inner_start..before].to_owned())
                } else {
                    RawField::Text(text)
                });
            }
            Event::End(_) => depth -= 1,
            Event::Eof => return Err(unexpected_end()),
            _ => {}
        }
    }
}

/// Appends a character or predefined entity reference; an unknown entity is
/// kept as written, since Royal documents declare no DTD.
fn push_reference(text: &mut String, reference: &BytesRef<'_>) {
    if let Ok(Some(character)) = reference.resolve_char_ref() {
        text.push(character);
    } else if let Some(resolved) = resolve_predefined_entity(reference) {
        text.push_str(resolved);
    } else {
        text.push('&');
        text.push_str(reference);
        text.push(';');
    }
}

fn expect_root(element: &BytesStart<'_>) -> Result<(), ImportError> {
    let name = element_name(element);
    if name == ROOT_ELEMENT {
        Ok(())
    } else {
        Err(ImportError::InvalidInput {
            reason: format!("elemento radice `{name}` inatteso: atteso `{ROOT_ELEMENT}`"),
        })
    }
}

fn element_name(element: &BytesStart<'_>) -> String {
    element.name().as_ref().to_owned()
}

fn read<'a>(reader: &mut Reader<&'a [u8]>) -> Result<Event<'a>, ImportError> {
    reader.read_event().map_err(|error| {
        // Only the kind of error and the offset: the details of quick-xml may
        // quote the content, which can hold encrypted secrets.
        let kind = match error {
            quick_xml::Error::Syntax(_) => "sintassi non valida",
            quick_xml::Error::IllFormed(_) => "elementi non bilanciati",
            quick_xml::Error::Escape(_) => "riferimento a entità non valido",
            quick_xml::Error::Encoding(_) => "codifica non valida",
            _ => "lettura non riuscita",
        };
        ImportError::Xml {
            reason: format!("{kind} al byte {}", reader.error_position()),
        }
    })
}

fn position(reader: &Reader<&[u8]>) -> Result<usize, ImportError> {
    usize::try_from(reader.buffer_position()).map_err(|_| ImportError::InvalidInput {
        reason: "documento troppo grande".to_owned(),
    })
}

fn unexpected_end() -> ImportError {
    ImportError::Xml {
        reason: format!("fine del file prima della chiusura di `{ROOT_ELEMENT}`"),
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn text(value: &str) -> RawField {
        RawField::Text(value.to_owned())
    }

    #[test]
    fn objects_keep_file_order_type_and_fields() {
        let xml = "<?xml version=\"1.0\"?>\n<RTSZDocument>\n  \
                   <RoyalFolder><ID>a</ID><Name>Uno</Name></RoyalFolder>\n  \
                   <RoyalCredential><ID>b</ID><UserName /></RoyalCredential>\n\
                   </RTSZDocument>";

        let objects = parse_document(xml).unwrap();

        let types: Vec<&str> = objects.iter().map(|o| o.type_name.as_str()).collect();
        assert_eq!(types, vec!["RoyalFolder", "RoyalCredential"]);
        assert_eq!(objects[0].fields["Name"], text("Uno"));
        assert_eq!(objects[1].fields["UserName"], text(""));
    }

    #[test]
    fn raw_xml_is_the_original_fragment() {
        let fragment = "<RoyalVNCConnection><ID>a</ID><URI>vnc.example.com</URI></RoyalVNCConnection>";
        let xml = format!("<RTSZDocument>\n  {fragment}\n</RTSZDocument>");

        let objects = parse_document(&xml).unwrap();

        assert_eq!(objects[0].raw_xml, fragment);
    }

    #[test]
    fn text_is_unescaped_including_cdata_and_character_references() {
        let xml = "<RTSZDocument><RoyalFolder>\
                   <Name>A &amp; B &#x41;&#66;</Name><Notes><![CDATA[<b>x</b>]]></Notes>\
                   </RoyalFolder></RTSZDocument>";

        let objects = parse_document(xml).unwrap();

        assert_eq!(objects[0].fields["Name"], text("A & B AB"));
        assert_eq!(objects[0].fields["Notes"], text("<b>x</b>"));
    }

    #[test]
    fn a_field_with_children_keeps_its_inner_xml() {
        let xml = "<RTSZDocument><RoyalFolder>\
                   <Notes><p>uno <b>due</b></p></Notes>\
                   </RoyalFolder></RTSZDocument>";

        let objects = parse_document(xml).unwrap();

        assert_eq!(
            objects[0].fields["Notes"],
            RawField::Markup("<p>uno <b>due</b></p>".to_owned())
        );
    }

    #[test]
    fn the_first_occurrence_of_a_repeated_field_wins() {
        let xml = "<RTSZDocument><RoyalFolder><Name>uno</Name><Name>due</Name></RoyalFolder></RTSZDocument>";

        let objects = parse_document(xml).unwrap();

        assert_eq!(objects[0].fields["Name"], text("uno"));
    }

    #[test]
    fn redacted_xml_drops_the_encrypted_fields_only() {
        let xml = "<RTSZDocument><RoyalVNCConnection><ID>a</ID>\
                   <CredentialPassword>Y2lwaGVydGV4dA==</CredentialPassword>\
                   <CredentialPasswordFromParent>False</CredentialPasswordFromParent>\
                   </RoyalVNCConnection></RTSZDocument>";

        let objects = parse_document(xml).unwrap();

        assert!(objects[0].has_secrets());
        assert_eq!(
            objects[0].redacted_xml(),
            "<RoyalVNCConnection><ID>a</ID>\
             <CredentialPasswordFromParent>False</CredentialPasswordFromParent>\
             </RoyalVNCConnection>"
        );
    }

    #[test]
    fn an_empty_root_has_no_objects() {
        assert_eq!(parse_document("<RTSZDocument/>").unwrap(), Vec::new());
    }

    #[test]
    fn another_root_element_is_rejected() {
        let error = parse_document("<html><body/></html>").unwrap_err();

        assert!(matches!(error, ImportError::InvalidInput { .. }));
    }

    #[test]
    fn malformed_xml_is_rejected_without_quoting_the_content() {
        let error = parse_document("<RTSZDocument><RoyalFolder><Name>secret-ish</Folder>")
            .unwrap_err();

        assert!(matches!(error, ImportError::Xml { .. }));
        assert!(!error.to_string().contains("secret-ish"));
    }

    #[test]
    fn a_truncated_document_is_rejected() {
        let error = parse_document("<RTSZDocument><RoyalFolder><ID>a</ID>").unwrap_err();

        assert!(matches!(error, ImportError::Xml { .. }));
    }
}
