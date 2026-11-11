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
    let xml =
        "<RTSZDocument><RoyalFolder><Name>uno</Name><Name>due</Name></RoyalFolder></RTSZDocument>";

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
fn text_deep_below_a_secret_element_is_secret() {
    for value in ["<v>Y2lwaGVydGV4dA==</v>", "<v><w>Y2lwaGVydGV4dA==</w></v>"] {
        let xml = format!(
            "<RTSZDocument><RoyalFolder><ID>a</ID>\
             <Settings><Password>{value}</Password></Settings>\
             </RoyalFolder></RTSZDocument>"
        );

        let objects = parse_document(&xml).unwrap();

        assert!(objects[0].is_secret("Settings"), "{value}");
        assert_eq!(
            objects[0].redacted_xml(),
            "<RoyalFolder><ID>a</ID></RoyalFolder>"
        );
    }
}

// Trimming the whole text below every level would take tens of seconds on
// this input; the length check keeps it linear.
#[test]
fn deep_secret_elements_over_much_whitespace_are_checked_in_linear_time() {
    let levels = 16_000;
    let xml = format!(
        "<RTSZDocument><RoyalFolder><ID>a</ID><Settings>{}{}{}</Settings></RoyalFolder></RTSZDocument>",
        "<Password>".repeat(levels),
        " ".repeat(1024 * 1024),
        "</Password>".repeat(levels)
    );
    let started = std::time::Instant::now();

    let objects = parse_document(&xml).unwrap();

    assert!(objects[0].is_secret("Settings"));
    assert!(started.elapsed() < std::time::Duration::from_secs(5));
}

#[test]
fn a_secret_attribute_on_a_deep_descendant_is_secret() {
    let xml = "<RTSZDocument><RoyalFolder><ID>a</ID>\
               <Settings><a><b><c Passphrase=\"Y2lwaGVydGV4dA==\"/></b></a></Settings>\
               </RoyalFolder></RTSZDocument>";

    let objects = parse_document(xml).unwrap();

    assert!(objects[0].is_secret("Settings"));
    assert!(!objects[0].redacted_xml().contains("Y2lwaGVydGV4dA=="));
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
    let error = parse_document("<RTSZDocument><RoyalFolder><Name>secret-ish</Folder>").unwrap_err();

    assert!(matches!(error, ImportError::Xml { .. }));
    assert!(!error.to_string().contains("secret-ish"));
}

#[test]
fn a_truncated_document_is_rejected() {
    let error = parse_document("<RTSZDocument><RoyalFolder><ID>a</ID>").unwrap_err();

    assert!(matches!(error, ImportError::Xml { .. }));
}
