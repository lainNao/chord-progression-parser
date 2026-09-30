use chord_progression_parser::{
    format_chord_progression, parse_chord_progression_string, ChordBlock, ChordExpression,
    ChordType, Extension,
};
use serde::Deserialize;
use strum::VariantNames;

#[derive(Deserialize)]
struct CanonicalSyntaxFixture {
    extensions: Vec<String>,
    valid: Vec<String>,
    invalid: Vec<String>,
}

/** Preserves the root-only modifier as an extension without changing the chord type. */
#[test]
fn root_only_notation_preserves_the_chord_head() {
    for (input, chord_type) in [
        ("C(1)", ChordType::Major),
        ("C#M(1)", ChordType::Major),
        ("Cbm(1)", ChordType::Minor),
        ("Caug(1)", ChordType::Augmented),
        ("Cdim(1)", ChordType::Diminished),
    ] {
        let ast = parse_chord_progression_string(input).expect("root-only notation must parse");
        let ChordBlock::Bar(bar) = &ast[0].chord_blocks[0] else {
            panic!("expected a chord bar");
        };
        let ChordExpression::Chord(chord) = &bar[0].chord_expression else {
            panic!("expected a chord expression");
        };

        assert_eq!(chord.plain, input);
        assert_eq!(chord.detailed.chord_type, chord_type);
        assert_eq!(chord.detailed.extensions, vec![Extension::One]);
        assert_eq!(
            serde_json::to_value(&chord.detailed.extensions).unwrap(),
            serde_json::json!(["1"])
        );
        let serialized = serde_json::to_string(&ast).expect("AST must serialize");
        assert_eq!(
            serde_json::from_str::<chord_progression_parser::Ast>(&serialized).unwrap(),
            ast
        );
        assert_eq!(format_chord_progression(&ast).unwrap(), input);
    }
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
