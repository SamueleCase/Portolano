//! Parsing of the Royal XML into raw objects (SPEC §8.3.2).
//!
//! Every child of `<RTSZDocument>` is an object; every child of an object is
//! a field (`docs/royal-format.md` §3). No field is interpreted here.

use std::collections::{BTreeMap, BTreeSet};
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
    /// Fields holding an encrypted secret in any occurrence: in their text,
    /// in a descendant element or in an attribute.
    secret_fields: BTreeSet<String>,
    /// Parts of `raw_xml` to replace so that no secret is kept: the field
    /// elements holding secrets, or the start tag if it has secret
    /// attributes. In document order.
    redactions: Vec<(Range<usize>, String)>,
}

impl RawObject {
    /// `raw_xml` without the parts that hold encrypted secrets: the
    /// ciphertext must not be kept anywhere (D3), not even in the XML of an
    /// unsupported node.
    pub(super) fn redacted_xml(&self) -> String {
        let mut redacted = String::with_capacity(self.raw_xml.len());
        let mut cursor = 0;
        for (span, replacement) in &self.redactions {
            redacted.push_str(&self.raw_xml[cursor..span.start]);
            redacted.push_str(replacement);
            cursor = span.end;
        }
        redacted.push_str(&self.raw_xml[cursor..]);
        redacted
    }

    /// The trimmed value of a field, `None` if absent, blank or holding a
    /// secret: a secret is never used as data.
    pub(super) fn value(&self, name: &str) -> Option<&str> {
        if self.is_secret(name) {
            return None;
        }
        self.fields
            .get(name)
            .map(|field| field.as_str().trim())
            .filter(|value| !value.is_empty())
    }

    /// Whether the field holds an encrypted secret.
    pub(super) fn is_secret(&self, name: &str) -> bool {
        self.secret_fields.contains(name)
    }

    /// Whether some parts of the XML hold encrypted secrets.
    pub(super) fn has_secrets(&self) -> bool {
        !self.redactions.is_empty()
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
                let mut object = new_object(&reader, xml, &element, start)?;
                read_fields(&mut reader, xml, &mut object, start)?;
                objects.push(object);
            }
            Event::Empty(element) => objects.push(new_object(&reader, xml, &element, start)?),
            Event::End(_) => return Ok(objects),
            Event::Eof => return Err(unexpected_end()),
            _ => {}
        }
    }
}

/// An object from its start tag, which began at byte `start` and has just
/// been read; until [`read_fields`] runs, `raw_xml` holds the tag only.
fn new_object(
    reader: &Reader<&[u8]>,
    xml: &str,
    element: &BytesStart<'_>,
    start: usize,
) -> Result<RawObject, ImportError> {
    let type_name = element_name(element);
    let tag = &xml[start..position(reader)?];
    let mut redactions = Vec::new();
    if has_secret_attribute(element)? {
        // Royal never writes attributes: one holding a secret drops them all.
        let bare_tag = if tag.ends_with("/>") {
            format!("<{type_name}/>")
        } else {
            format!("<{type_name}>")
        };
        redactions.push((0..tag.len(), bare_tag));
    }
    Ok(RawObject {
        type_name,
        fields: BTreeMap::new(),
        raw_xml: tag.to_owned(),
        secret_fields: BTreeSet::new(),
        redactions,
    })
}

/// Reads the fields of an object whose start tag began at byte `start`, up
/// to its end tag.
fn read_fields(
    reader: &mut Reader<&[u8]>,
    xml: &str,
    object: &mut RawObject,
    start: usize,
) -> Result<(), ImportError> {
    loop {
        let field_start = position(reader)?;
        let (name, field) = match read(reader)? {
            Event::Start(element) => {
                let field = read_field_value(reader, xml, &element)?;
                (element_name(&element), field)
            }
            Event::Empty(element) => {
                let field = FieldValue {
                    value: RawField::Text(String::new()),
                    secret: has_secret_attribute(&element)?,
                };
                (element_name(&element), field)
            }
            Event::End(_) => {
                object.raw_xml = xml[start..position(reader)?].to_owned();
                return Ok(());
            }
            Event::Eof => return Err(unexpected_end()),
            _ => continue,
        };
        if field.secret || is_secret_value(&name, field.value.as_str()) {
            let span = field_start - start..position(reader)? - start;
            object.redactions.push((span, String::new()));
            object.secret_fields.insert(name.clone());
        }
        object.fields.entry(name).or_insert(field.value);
    }
}

/// The value of a field, and whether a descendant element or an attribute
/// holds a secret.
struct FieldValue {
    value: RawField,
    secret: bool,
}

/// Reads the content of the field opened by `field`, up to its end tag.
fn read_field_value(
    reader: &mut Reader<&[u8]>,
    xml: &str,
    field: &BytesStart<'_>,
) -> Result<FieldValue, ImportError> {
    let inner_start = position(reader)?;
    let mut text = String::new();
    let mut secret = has_secret_attribute(field)?;
    // Open descendant elements with their own text, to check their names.
    let mut open: Vec<(String, String)> = Vec::new();
    let mut has_children = false;
    loop {
        let before = position(reader)?;
        let mut content = String::new();
        match read(reader)? {
            Event::Text(event) => content.push_str(&event.xml10_content()),
            Event::CData(event) => content.push_str(&event),
            Event::GeneralRef(reference) => push_reference(&mut content, &reference),
            Event::Start(element) => {
                secret |= has_secret_attribute(&element)?;
                open.push((element_name(&element), String::new()));
                has_children = true;
            }
            Event::Empty(element) => {
                secret |= has_secret_attribute(&element)?;
                has_children = true;
            }
            Event::End(_) => match open.pop() {
                Some((name, own_text)) => secret |= is_secret_value(&name, &own_text),
                None => {
                    let value = if has_children {
                        RawField::Markup(xml[inner_start..before].to_owned())
                    } else {
                        RawField::Text(text)
                    };
                    return Ok(FieldValue { value, secret });
                }
            },
            Event::Eof => return Err(unexpected_end()),
            _ => {}
        }
        if let Some((_, own_text)) = open.last_mut() {
            own_text.push_str(&content);
        }
        text.push_str(&content);
    }
}

/// Whether an attribute of the element holds a secret: the attribute has a
/// secret name, or the element has one, and the value is not a plain
/// setting.
fn has_secret_attribute(element: &BytesStart<'_>) -> Result<bool, ImportError> {
    let name = element_name(element);
    for attribute in element.attributes() {
        let attribute = attribute.map_err(|_| ImportError::Xml {
            reason: "attributo non valido".to_owned(),
        })?;
        let value = attribute.value.as_ref();
        if is_secret_value(attribute.key.as_ref(), value) || is_secret_value(&name, value) {
            return Ok(true);
        }
    }
    Ok(false)
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
        let fragment =
            "<RoyalVNCConnection><ID>a</ID><URI>vnc.example.com</URI></RoyalVNCConnection>";
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
    fn a_field_with_a_nested_secret_element_is_secret() {
        let xml = "<RTSZDocument><RoyalFolder><ID>a</ID>\
                   <CustomProperties><Property><Name>pin</Name>\
                   <Password>Y2lwaGVydGV4dA==</Password></Property></CustomProperties>\
                   </RoyalFolder></RTSZDocument>";

        let objects = parse_document(xml).unwrap();

        assert!(objects[0].is_secret("CustomProperties"));
        assert_eq!(objects[0].value("CustomProperties"), None);
        assert_eq!(
            objects[0].redacted_xml(),
            "<RoyalFolder><ID>a</ID></RoyalFolder>"
        );
    }

    #[test]
    fn a_field_with_a_secret_attribute_is_secret() {
        let xml = "<RTSZDocument><RoyalFolder><ID>a</ID>\
                   <Password value=\"Y2lwaGVydGV4dA==\"/>\
                   <Extra><Token Passphrase=\"Y2lwaGVydGV4dA==\">x</Token></Extra>\
                   </RoyalFolder></RTSZDocument>";

        let objects = parse_document(xml).unwrap();

        assert!(objects[0].is_secret("Password"));
        assert!(objects[0].is_secret("Extra"));
        assert!(!objects[0].redacted_xml().contains("Y2lwaGVydGV4dA=="));
    }

    #[test]
    fn plain_settings_in_nested_elements_are_not_secret() {
        let xml = "<RTSZDocument><RoyalFolder><ID>a</ID>\
                   <Options><SavePassword>True</SavePassword>\
                   <Item PasswordMode=\"2\"/></Options>\
                   </RoyalFolder></RTSZDocument>";

        let objects = parse_document(xml).unwrap();

        assert!(!objects[0].is_secret("Options"));
        assert!(!objects[0].has_secrets());
    }

    #[test]
    fn secret_attributes_of_the_object_tag_are_dropped() {
        let xml = "<RTSZDocument><RoyalVNCConnection Password=\"Y2lwaGVydGV4dA==\">\
                   <ID>a</ID></RoyalVNCConnection></RTSZDocument>";

        let objects = parse_document(xml).unwrap();

        assert_eq!(
            objects[0].redacted_xml(),
            "<RoyalVNCConnection><ID>a</ID></RoyalVNCConnection>"
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
        let error =
            parse_document("<RTSZDocument><RoyalFolder><Name>secret-ish</Folder>").unwrap_err();

        assert!(matches!(error, ImportError::Xml { .. }));
        assert!(!error.to_string().contains("secret-ish"));
    }

    #[test]
    fn a_truncated_document_is_rejected() {
        let error = parse_document("<RTSZDocument><RoyalFolder><ID>a</ID>").unwrap_err();

        assert!(matches!(error, ImportError::Xml { .. }));
    }
}
