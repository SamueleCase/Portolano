//! Fields common to every object, unmapped fields and unsupported types
//! (SPEC §8.3.5).

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chrono::{TimeZone, Utc};
use portolano_core::{CustomProperty, Icon, NodeKind, PropertyValue};
use portolano_import::{ItemStatus, SecretField};
use pretty_assertions::assert_eq;
use sha2::{Digest, Sha256};

use crate::testutil::*;

const PNG: &[u8] = b"\x89PNG\r\n\x1a\nfake-pixels";

fn folder_with(fields: &[(&str, &str)]) -> String {
    document(&[object("RoyalFolder", 2, 1, fields)])
}

#[test]
fn an_empty_name_becomes_senza_nome() {
    let outcome = import(&folder_with(&[("Name", "  ")]));

    assert_eq!(find(&outcome.root, uuid(2)).name, "(senza nome)");
}

#[test]
fn royal_dates_are_read_as_utc() {
    let outcome = import(&folder_with(&[
        ("Created", "02/01/2026 08:30:00.000"),
        ("Modified", "11/09/2026 17:45:12.345"),
    ]));

    let node = find(&outcome.root, uuid(2));
    assert_eq!(
        node.created,
        Utc.with_ymd_and_hms(2026, 2, 1, 8, 30, 0).unwrap()
    );
    assert_eq!(
        node.modified,
        Utc.with_ymd_and_hms(2026, 11, 9, 17, 45, 12).unwrap()
            + chrono::Duration::milliseconds(345)
    );
}

#[test]
fn an_unreadable_date_uses_the_import_time_and_keeps_the_original() {
    let outcome = import(&folder_with(&[("Created", "ieri")]));

    let node = find(&outcome.root, uuid(2));
    assert_eq!(node.created, outcome.report.imported_at);
    assert_eq!(node.extra["royal.Created"], "ieri");
}

#[test]
fn description_color_and_favorite_are_mapped() {
    let outcome = import(&folder_with(&[
        ("Description", "Server di prova"),
        ("Color", "#2E8B57"),
        ("Favorite", "True"),
    ]));

    let node = find(&outcome.root, uuid(2));
    assert_eq!(node.description.as_deref(), Some("Server di prova"));
    assert_eq!(node.color.as_deref(), Some("#2E8B57"));
    assert!(node.favorite);
}

#[test]
fn a_library_icon_keeps_its_royal_name() {
    let outcome = import(&folder_with(&[("CustomImageName", "Flat/Hardware/Server")]));

    assert_eq!(
        find(&outcome.root, uuid(2)).icon,
        Some(Icon::Royal {
            name: "Flat/Hardware/Server".to_owned()
        })
    );
}

#[test]
fn an_embedded_image_becomes_one_pending_icon_shared_by_its_users() {
    let image = STANDARD.encode(PNG);
    let xml = document(&[
        object("RoyalFolder", 2, 1, &[("CustomImage", &image)]),
        object("RoyalFolder", 3, 1, &[("CustomImage", &image)]),
    ]);

    let outcome = import(&xml);

    let sha256 = hex::encode(Sha256::digest(PNG));
    let expected = Icon::Custom {
        sha256: sha256.clone(),
        mime: "image/png".to_owned(),
    };
    assert_eq!(find(&outcome.root, uuid(2)).icon.as_ref(), Some(&expected));
    assert_eq!(find(&outcome.root, uuid(3)).icon.as_ref(), Some(&expected));
    assert_eq!(outcome.icons.len(), 1);
    assert_eq!(outcome.icons[0].placeholder_sha256, sha256);
    assert_eq!(outcome.icons[0].bytes, PNG);
}

#[test]
fn an_unrecognized_image_stays_in_extra() {
    let image = STANDARD.encode(b"not an image");

    let outcome = import(&folder_with(&[("CustomImage", &image)]));

    assert_eq!(
        find(&outcome.root, uuid(2)).extra["royal.CustomImage"],
        image.as_str()
    );
    assert_eq!(status(&outcome, 2), ItemStatus::Partial);
}

#[test]
fn html_notes_become_text_and_keep_the_original() {
    let html = "&lt;p&gt;Uno &amp;amp; due&lt;/p&gt;&lt;p&gt;tre&lt;/p&gt;";

    let outcome = import(&folder_with(&[("Notes", html)]));

    let node = find(&outcome.root, uuid(2));
    assert_eq!(node.notes.as_deref(), Some("Uno & due\ntre"));
    assert_eq!(node.extra["royal.Notes"], "<p>Uno &amp; due</p><p>tre</p>");
}

#[test]
fn plain_notes_are_kept_as_they_are() {
    let outcome = import(&folder_with(&[("Notes", "solo testo")]));

    let node = find(&outcome.root, uuid(2));
    assert_eq!(node.notes.as_deref(), Some("solo testo"));
    assert!(!node.extra.contains_key("royal.Notes"));
}

#[test]
fn custom_fields_become_text_properties() {
    let outcome = import(&folder_with(&[
        ("CustomField1", "rack 4"),
        ("CustomField2", ""),
        ("CustomField10", "ultimo"),
    ]));

    let properties = &find(&outcome.root, uuid(2)).custom_properties;
    let text = |name: &str, value: &str| CustomProperty {
        name: name.to_owned(),
        value: PropertyValue::Text {
            value: value.to_owned(),
        },
    };
    assert_eq!(
        properties,
        &vec![
            text("CustomField1", "rack 4"),
            text("CustomField10", "ultimo")
        ]
    );
}

#[test]
fn protected_custom_fields_are_requested_as_secrets() {
    let xml = folder_with(&[("CustomProtectedField2", FAKE_CIPHERTEXT)]);

    let (outcome, prompter) = import_answering(&xml, "typed-by-the-user");

    assert_eq!(
        prompter.requests[0].field,
        SecretField::CustomProperty("CustomProtectedField2".to_owned())
    );
    assert_eq!(
        find(&outcome.root, uuid(2)).custom_properties,
        vec![CustomProperty {
            name: "CustomProtectedField2".to_owned(),
            value: PropertyValue::Secret {
                secret: outcome.secrets[0].0.clone()
            },
        }]
    );
    assert_eq!(
        reasons(&outcome, 2),
        vec!["proprietà protetta `CustomProtectedField2`: campi Royal non verificati"]
    );
}

#[test]
fn a_template_is_tagged_and_partial() {
    let outcome = import(&folder_with(&[("IsConnectionTemplate", "True")]));

    assert_eq!(find(&outcome.root, uuid(2)).tags, vec!["royal-template"]);
    assert_eq!(reasons(&outcome, 2), vec!["i modelli non sono supportati"]);
}

#[test]
fn unmapped_fields_go_to_extra_as_strings_and_are_counted() {
    let xml = document(&[
        object(
            "RoyalFolder",
            2,
            1,
            &[("IsExpanded", "True"), ("ColorName", "Royal.Mariner")],
        ),
        object("RoyalFolder", 3, 1, &[("IsExpanded", "False")]),
    ]);

    let outcome = import(&xml);

    let node = find(&outcome.root, uuid(2));
    assert_eq!(node.extra["royal.IsExpanded"], "True");
    assert_eq!(node.extra["royal.ColorName"], "Royal.Mariner");
    let counted = &outcome.report.unmapped_fields["RoyalFolder"];
    assert_eq!((counted["IsExpanded"], counted["ColorName"]), (2, 1));
    assert_eq!(status(&outcome, 2), ItemStatus::Imported);
}

#[test]
fn an_unknown_type_is_kept_verbatim_without_ciphertext() {
    let xml = document(&[object(
        "RoyalVNCConnection",
        2,
        1,
        &[
            ("Name", "vnc"),
            ("URI", "vnc.example.com"),
            ("CredentialPassword", FAKE_CIPHERTEXT),
        ],
    )]);

    let outcome = import(&xml);

    let node = find(&outcome.root, uuid(2));
    let NodeKind::Unsupported(unsupported) = &node.kind else {
        panic!("expected an unsupported node");
    };
    assert_eq!(unsupported.source_type, "RoyalVNCConnection");
    assert!(unsupported.raw.contains("<URI>vnc.example.com</URI>"));
    assert!(!unsupported.raw.contains(FAKE_CIPHERTEXT));
    assert_eq!(node.name, "vnc");
    assert_eq!(status(&outcome, 2), ItemStatus::Unsupported);
    assert_eq!(
        reasons(&outcome, 2),
        vec![
            "tipo Royal non supportato: RoyalVNCConnection",
            "campi cifrati rimossi dall'XML conservato"
        ]
    );
}

#[test]
fn a_field_with_a_nested_secret_is_dropped_from_extra() {
    let nested =
        format!("<Property><Name>pin</Name><Password>{FAKE_CIPHERTEXT}</Password></Property>");

    let outcome = import(&folder_with(&[("CustomProperties", &nested)]));

    let node = find(&outcome.root, uuid(2));
    assert!(!node.extra.contains_key("royal.CustomProperties"));
    assert_eq!(
        reasons(&outcome, 2),
        vec!["campo cifrato `CustomProperties` non importato"]
    );
}

#[test]
fn a_field_with_a_secret_attribute_is_dropped_from_extra() {
    let xml = document(&[format!(
        "<RoyalFolder><ID>{}</ID><ParentID>{}</ParentID>\
         <Password value=\"{FAKE_CIPHERTEXT}\"/></RoyalFolder>",
        id(2),
        id(1)
    )]);

    let outcome = import(&xml);

    assert!(
        !find(&outcome.root, uuid(2))
            .extra
            .contains_key("royal.Password")
    );
    assert_eq!(
        reasons(&outcome, 2),
        vec!["campo cifrato `Password` non importato"]
    );
}

#[test]
fn a_mapped_field_with_a_nested_secret_is_not_used() {
    let nested = format!("<p>nota</p><Secret>{FAKE_CIPHERTEXT}</Secret>");

    let outcome = import(&folder_with(&[("Notes", &nested)]));

    let node = find(&outcome.root, uuid(2));
    assert_eq!(node.notes, None);
    assert!(!node.extra.contains_key("royal.Notes"));
    assert_eq!(
        reasons(&outcome, 2),
        vec!["campo cifrato `Notes` non importato"]
    );
}

#[test]
fn an_unknown_type_with_a_nested_secret_keeps_no_ciphertext() {
    let nested = format!("<Token Passphrase=\"{FAKE_CIPHERTEXT}\">x</Token>");
    let xml = document(&[object(
        "RoyalHologramConnection",
        2,
        1,
        &[("URI", "h.example.com"), ("Options", &nested)],
    )]);

    let outcome = import(&xml);

    let NodeKind::Unsupported(unsupported) = &find(&outcome.root, uuid(2)).kind else {
        panic!("expected an unsupported node");
    };
    assert!(unsupported.raw.contains("<URI>h.example.com</URI>"));
    assert!(!unsupported.raw.contains(FAKE_CIPHERTEXT));
    assert!(!unsupported.raw.contains("<Options>"));
}
