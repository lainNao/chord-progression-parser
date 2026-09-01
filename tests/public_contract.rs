use chord_progression_parser::{format_chord_progression, parse_chord_progression_string};
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

/** Verifies the formatter through the public Rust API and parser contract fixture. */
#[test]
fn formats_the_public_contract_fixture() {
    let ast = parse_chord_progression_string(include_str!("fixtures/public_contract.chord"))
        .expect("the contract input must parse");

    let formatted = format_chord_progression(&ast).expect("the parsed AST must be formattable");

    assert_eq!(
        formatted,
        "@section=Verse\n@repeat=2\n[key=C]C(M9), G/B - Dm(7) - ? - _ - %"
    );
    assert_eq!(
        parse_chord_progression_string(&formatted).expect("formatted source must parse"),
        ast
    );
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

/** Verifies every checked-in malformed input is rejected through the public API. */
#[test]
fn rejects_the_malformed_fixture_without_panicking() {
    for input in include_str!("fixtures/malformed_inputs.txt").lines() {
        let result = std::panic::catch_unwind(|| parse_chord_progression_string(input));
        assert!(result.is_ok(), "public parser panicked for {input:?}");
        assert!(
            result.expect("panic was checked").is_err(),
            "public parser accepted {input:?}"
        );
    }
}

/** Verifies documented behavior changes at the public boundary. */
#[test]
fn applies_the_intentional_compatibility_changes() {
    let sections = parse_chord_progression_string("C\n\n\n\nD")
        .expect("extra blank lines must separate sections");
    assert_eq!(sections.len(), 2);

    let postfix_meta =
        parse_chord_progression_string("C[key=A]").expect_err("postfix metadata must be rejected");
    assert_eq!(postfix_meta.error.code.to_string(), "TKN-1");
}
