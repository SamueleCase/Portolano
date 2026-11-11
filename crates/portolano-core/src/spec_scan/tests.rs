use pretty_assertions::assert_eq;

use super::*;

#[test]
fn prompt_value_is_the_serialized_form_of_prompt() {
    let expected = serde_json::to_value(CredentialSpec::Prompt).unwrap();

    assert_eq!(prompt_value(), expected);
}

#[test]
fn embedded_spec_ignores_objects_whose_mode_is_not_a_credential_mode() {
    let resolution = serde_json::json!({ "mode": "fit" });

    assert_eq!(embedded_spec(&resolution), None);
}
