use chord_progression_parser::{
    format_chord_progression, parse_chord_progression_string, Extension,
};
use serde::Deserialize;
use strum::VariantNames;

#[derive(Deserialize)]
struct CanonicalSyntaxFixture {
    extensions: Vec<String>,
    valid: Vec<String>,
    invalid: Vec<String>,
}

/** Checks the shared syntax contract, every extension, and AST round trips. */
#[test]
fn canonical_syntax_matches_the_parser() {
    let fixture: CanonicalSyntaxFixture =
        serde_json::from_str(include_str!("fixtures/canonical_syntax.json"))
            .expect("canonical syntax fixture must be valid JSON");

    assert_eq!(fixture.extensions, Extension::VARIANTS);

    // Generate a valid chord for every declared extension, including future additions.
    let extension_chords = fixture
        .extensions
        .iter()
        .map(|extension| format!("C({extension})"));
    for input in fixture.valid.into_iter().chain(extension_chords) {
        let ast = parse_chord_progression_string(&input)
            .unwrap_or_else(|errors| panic!("valid fixture {input:?} failed: {errors:?}"));
        let formatted = format_chord_progression(&ast)
            .unwrap_or_else(|error| panic!("valid fixture {input:?} failed to format: {error:?}"));
        assert_eq!(
            parse_chord_progression_string(&formatted).expect("formatted fixture must parse"),
            ast,
            "AST round trip failed for {input:?}"
        );
    }

    for input in fixture.invalid {
        assert!(
            parse_chord_progression_string(&input).is_err(),
            "invalid fixture {input:?} must be rejected"
        );
    }
}
