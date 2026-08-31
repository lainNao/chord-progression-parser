use chord_progression_parser::parse_chord_progression_string;
use serde_json::{json, Value};

/** Verifies the JSON contract through the public Rust entry point. */
#[test]
fn parses_the_public_contract_fixture() {
    let input = include_str!("fixtures/public_contract.chord");
    let expected: Value = serde_json::from_str(include_str!("fixtures/public_contract.json"))
        .expect("the checked-in contract fixture must be valid JSON");

    let ast = parse_chord_progression_string(input).expect("the contract input must parse");

    assert_eq!(json!({ "success": true, "ast": ast }), expected);
}

/** Keeps a representative public error code and source position stable. */
#[test]
fn reports_extension_error_at_the_invalid_extension() {
    let error = parse_chord_progression_string("C(9,111)")
        .expect_err("an unknown extension must be rejected");

    assert_eq!(error.error.code.to_string(), "EXT-1");
    assert_eq!(error.position.line_number, 1);
    assert_eq!(error.position.column_number, 5);
    assert_eq!(error.position.length, 3);
}

/** Keeps an empty document distinct from a document with one empty section. */
#[test]
fn parses_an_empty_document_as_an_empty_ast() {
    let ast = parse_chord_progression_string("").expect("an empty document must parse");

    assert_eq!(json!(ast), json!([]));
}
