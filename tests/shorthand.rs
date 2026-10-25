#[path = "support/extension_order.rs"]
mod extension_order;

use std::str::FromStr;

use chord_progression_parser::{
    format_chord_progression, parse_chord_progression_string,
    parse_chord_progression_string_with_warnings, Accidental, Ast, Chord, ChordBlock,
    ChordDetailed, ChordExpression, ChordInfo, ChordType, Extension, Section, WarningCode,
};
use serde::Deserialize;
use strum::VariantNames;

#[derive(Deserialize)]
struct ShorthandCase {
    source: String,
    expected: Chord,
}

/** Loads literal public-contract expectations without invoking production parsing or sorting. */
fn shorthand_cases() -> Vec<ShorthandCase> {
    serde_json::from_str(include_str!("fixtures/shorthand.json")).expect("valid shorthand fixture")
}

/** Builds a formatter fixture directly from a specified chord rather than parser output. */
fn chord_ast(chord: Chord) -> Ast {
    vec![Section {
        meta_infos: Vec::new(),
        chord_blocks: vec![ChordBlock::Bar(vec![ChordInfo {
            meta_infos: Vec::new(),
            denominator: None,
            chord_expression: ChordExpression::Chord(chord),
        }])],
    }]
}

/** Extracts one chord through the public document parser. */
fn parse_chord(source: &str) -> Chord {
    let ast = parse_chord_progression_string(source).expect(source);
    let ChordBlock::Bar(bar) = &ast[0].chord_blocks[0] else {
        panic!("expected one bar");
    };
    let ChordExpression::Chord(chord) = &bar[0].chord_expression else {
        panic!("expected one chord");
    };
    chord.clone()
}

/** Verifies accepted document syntax against literal AST fields, not another parser path. */
#[test]
fn document_parser_matches_literal_shorthand_contract() {
    for case in shorthand_cases() {
        assert_eq!(parse_chord(&case.source), case.expected, "{}", case.source);
    }
}

/** Gives the public standalone parser its own independently specified detailed expectations. */
#[test]
fn standalone_parser_matches_literal_shorthand_contract() {
    for case in shorthand_cases() {
        assert_eq!(
            ChordDetailed::from_str(&case.source).unwrap(),
            case.expected.detailed,
            "{}",
            case.source
        );
    }
}

/** Checks slash grammar validation and preservation of each compact input spelling. */
#[test]
fn denominators_accept_the_same_contract_and_retain_spelling() {
    for case in shorthand_cases() {
        let source = format!("F/{}", case.source);
        let ast = parse_chord_progression_string(&source).expect(&source);
        let ChordBlock::Bar(bar) = &ast[0].chord_blocks[0] else {
            panic!("expected bar")
        };
        assert_eq!(
            bar[0].denominator.as_deref(),
            Some(case.expected.plain.as_str()),
            "{source}"
        );
    }
}

/** Checks formatter spelling using directly constructed ASTs, independently of initial parsing. */
#[test]
fn formatter_preserves_the_literal_shorthand_spelling() {
    for case in shorthand_cases() {
        let plain = case.expected.plain.clone();
        assert_eq!(
            format_chord_progression(&chord_ast(case.expected)).unwrap(),
            plain,
            "{}",
            case.source
        );
    }
}

/** Gives M-prefixed extension tokens precedence and keeps root accidentals unambiguous. */
#[test]
fn distinguishes_major_extensions_and_root_accidentals() {
    let major_seven = parse_chord("CM7");
    assert_eq!(major_seven.detailed.chord_type, ChordType::Major);
    assert_eq!(major_seven.detailed.extensions, [Extension::MajorSeven]);
    assert_eq!(parse_chord("CM(7)").detailed.extensions, [Extension::Seven]);
    assert_eq!(parse_chord("CmM7").detailed.chord_type, ChordType::Minor);
    for (source, accidental) in [("C#5", Accidental::Sharp), ("Cb5", Accidental::Flat)] {
        let chord = parse_chord(source);
        assert_eq!(chord.detailed.accidental, Some(accidental));
        assert_eq!(chord.detailed.extensions, [Extension::Five]);
    }
    assert_eq!(parse_chord("C(#5)").detailed.accidental, None);
    assert_eq!(
        parse_chord("C(b5)").detailed.extensions,
        [Extension::FlatFive]
    );
    // Shorthand does not insert an implicit seventh or alter a diminished seventh's stored token.
    assert_eq!(parse_chord("C9").detailed.extensions, [Extension::Nine]);
    assert_eq!(parse_chord("Cdim7").detailed.extensions, [Extension::Seven]);
}

/** Sorts structured data independently of input order, preserving duplicates and input spelling. */
#[test]
fn orders_extensions_without_rewriting_plain_or_discarding_duplicates() {
    let first = parse_chord("C7sus4");
    let second = parse_chord("Csus4(7)");
    assert_eq!(first.detailed, second.detailed);
    assert_ne!(first.plain, second.plain);
    let source = "C ( omit5 , #11 , add9 , 7 , sus4 , b9 , 7 )";
    let chord = parse_chord(source);
    assert_eq!(chord.plain, "C(omit5,#11,add9,7,sus4,b9,7)");
    assert_eq!(
        chord.detailed.extensions,
        [
            Extension::Seven,
            Extension::Seven,
            Extension::Sus4,
            Extension::FlatNine,
            Extension::SharpEleven,
            Extension::Add9,
            Extension::Omit5,
        ]
    );
    assert_eq!(ChordDetailed::from_str(source).unwrap(), chord.detailed);
}

/** Rejects externally supplied ASTs that violate the structured ordering invariant. */
#[test]
fn formatter_rejects_old_unsorted_extensions() {
    let mut chord = Chord {
        plain: "Csus4(7)".to_owned(),
        detailed: ChordDetailed {
            base: chord_progression_parser::Base::C,
            accidental: None,
            chord_type: ChordType::Major,
            extensions: vec![Extension::Sus4, Extension::Seven],
        },
    };
    assert!(
        format_chord_progression(&chord_ast(chord.clone())).is_err(),
        "old unsorted ASTs require reparsing"
    );
    chord.detailed.extensions = vec![Extension::Seven, Extension::Sus4];
    assert_eq!(
        format_chord_progression(&chord_ast(chord)).unwrap(),
        "Csus4(7)"
    );
}

/** Keeps unsupported aliases, split atoms, and consecutive primary numbers rejected. */
#[test]
fn rejects_unagreed_or_ambiguous_shorthand_in_every_chord_context() {
    for source in [
        "C0",
        "C8",
        "C10",
        "C01",
        "C79",
        "C69",
        "C713",
        "C111",
        "C14",
        "CM79",
        "C7M7",
        "Csus47",
        "C7 sus4",
        "C maj7",
        "Cmaj7",
        "Cmmaj7",
        "C(maj7)",
        "Csus",
        "C7sus",
        "Cadd2",
        "Cadd4",
        "C7alt",
        "Cno3",
        "Comit3",
        "Comit1",
        "Co",
        "Co7",
        "Cø7",
        "C°7",
        "C+7",
        "C7aug",
        "C7+",
        "C△7",
        "CΔ7",
        "C7()",
        "C7(9)(13)",
        "C(7)sus4",
        "C7(9,)",
        "C7(unknown)",
        "C7😀",
        "C7♭9",
        "C7\u{200b}",
    ] {
        assert!(parse_chord_progression_string(source).is_err(), "{source}");
        assert!(ChordDetailed::from_str(source).is_err(), "{source}");
        assert!(
            parse_chord_progression_string(&format!("F/{source}")).is_err(),
            "{source}"
        );
    }
    for source in ["C6/9", "[key=C7]C", "?7", "_7", "C-%7"] {
        assert!(parse_chord_progression_string(source).is_err(), "{source}");
    }
    assert!(
        parse_chord_progression_string("Cm(o)").is_ok(),
        "existing parenthesized o remains valid"
    );
}

/** Pins public diagnostic codes and original source spans independently of rejection-only tests. */
#[test]
fn reports_shorthand_failures_in_the_correct_diagnostic_domain() {
    for (chord, code, fragment, relative_offset) in [
        ("C79", "CHO-1", "C79", 0),
        ("Cmaj7", "CHO-1", "Cmaj7", 0),
        ("Csus", "CHO-1", "Csus", 0),
        ("C7(unknown)", "EXT-1", "unknown", 3),
        ("C7(9)(13)", "EXT-4", "(", 5),
        ("C7()", "EXT-2", "(", 2),
        ("C7(9,)", "EXT-2", ",", 4),
        ("C6/9", "DEN-1", "/9", 2),
        ("C/F#m79", "DEN-1", "/F#m79", 1),
        ("[key=C7]C", "CIMV-4", "C7", 5),
        ("?7", "CHO-1", "?7", 0),
        ("C7 sus4", "TKN-1", "sus4", 3),
    ] {
        let prefix = "@section=😀\r\n";
        let source = format!("{prefix}{chord}");
        let errors = parse_chord_progression_string(&source).expect_err(&source);
        assert_eq!(errors.len(), 1, "{source:?}");
        let error = &errors[0];
        assert_eq!(error.error.code.to_string(), code, "{source:?}");
        assert_eq!(error.position.line_number, 2);
        assert_eq!(error.position.column_number, relative_offset + 1);
        assert_eq!(error.position.length, fragment.chars().count());
        let start = prefix.encode_utf16().count() + relative_offset;
        assert_eq!(error.position.start_offset, start);
        assert_eq!(
            error.position.end_offset,
            start + fragment.encode_utf16().count()
        );
    }
}

/** Finds duplicate spans before sorting, across both notations and after Unicode metadata. */
#[test]
fn keeps_original_duplicate_positions_across_parentheses() {
    let source = "@section=😀\r\nC7sus4sus4 ( 7 , sus4 ) - Comit5omit5 - C/G7(7)";
    let report = parse_chord_progression_string_with_warnings(source);
    assert!(report.result.is_ok());
    let fragments = ["sus4", "7", "sus4", "omit5"];
    let starts = [
        source.find("sus4sus4").unwrap() + 4,
        source.find(" 7 ").unwrap() + 1,
        source.find(", sus4").unwrap() + 2,
        source.find("omit5omit5").unwrap() + 5,
    ];
    assert_eq!(report.warnings.len(), fragments.len());
    for ((warning, fragment), start) in report.warnings.iter().zip(fragments).zip(starts) {
        assert_eq!(warning.code, WarningCode::DuplicateExtension);
        assert_eq!(warning.additional_info.as_deref(), Some(fragment));
        let offset = source[..start].encode_utf16().count();
        assert_eq!(warning.position.start_offset, offset);
        assert_eq!(warning.position.end_offset, offset + fragment.len());
        assert_eq!(warning.position.line_number, 2);
        assert_eq!(warning.position.length, fragment.len());
        assert_eq!(
            warning.position.column_number,
            source[source.find('\n').unwrap() + 1..start]
                .chars()
                .count()
                + 1
        );
    }
}

/** Preserves warnings in malformed groups and resumes at the following chord. */
#[test]
fn recovers_after_malformed_shorthand_extension_groups() {
    let report =
        parse_chord_progression_string_with_warnings("C7(7,unknown,7)-D7(7)-C69-Eomit5(omit5)");
    let errors = report
        .result
        .expect_err("invalid extensions and shorthand must fail");
    assert_eq!(
        errors
            .iter()
            .map(|error| error.error.code.to_string())
            .collect::<Vec<_>>(),
        ["EXT-1", "CHO-1"]
    );
    assert_eq!(report.warnings.len(), 4);
    assert_eq!(
        report.warnings.last().unwrap().additional_info.as_deref(),
        Some("omit5")
    );
}

/** Locks the complete AST order independently of enum declaration order. */
#[test]
fn has_a_complete_stable_extension_order() {
    let expected = extension_order::expected_order();
    let mut declared: Vec<_> = Extension::VARIANTS
        .iter()
        .map(|value| Extension::from_str(value).unwrap())
        .collect();
    declared.sort_by_key(extension_order::expected_rank);
    assert_eq!(
        expected, declared,
        "the order specification must cover every variant exactly once"
    );
    let source = format!(
        "C({})",
        expected
            .iter()
            .rev()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
    );
    assert_eq!(parse_chord(&source).detailed.extensions, expected);
}
