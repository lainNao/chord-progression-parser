#[path = "support/extension_order.rs"]
mod extension_order;
mod support;

use std::str::FromStr;

use chord_progression_parser::{
    format_chord_progression, parse_chord_progression_string,
    parse_chord_progression_string_with_warnings, Chord, ChordBlock, ChordDetailed,
    ChordExpression, Extension,
};

/** Extracts actual chord data for comparison with independently constructed expectations. */
fn chord_from_source(source: &str) -> Chord {
    let ast = parse_chord_progression_string(source).expect(source);
    let ChordBlock::Bar(bar) = &ast[0].chord_blocks[0] else {
        panic!("expected bar")
    };
    let ChordExpression::Chord(chord) = &bar[0].chord_expression else {
        panic!("expected chord")
    };
    chord.clone()
}

/** Crosses token boundaries against literal head values and a specification-owned extension order. */
#[test]
fn checks_primary_modifier_boundaries_against_explicit_notation() {
    let primaries = [
        "", "1", "2", "3", "4", "5", "6", "7", "9", "11", "13", "M7", "M9", "M11", "M13",
    ];
    let modifiers = [
        "", "sus2", "sus4", "b3", "b5", "#5", "b6", "b9", "#9", "b11", "#11", "b13", "#13", "add9",
        "add11", "add13", "omit5",
    ];
    let mut checked = 0;
    for (head, base, accidental, chord_type) in [
        (
            "CM",
            chord_progression_parser::Base::C,
            None,
            chord_progression_parser::ChordType::Major,
        ),
        (
            "Cm",
            chord_progression_parser::Base::C,
            None,
            chord_progression_parser::ChordType::Minor,
        ),
        (
            "Caug",
            chord_progression_parser::Base::C,
            None,
            chord_progression_parser::ChordType::Augmented,
        ),
        (
            "Cdim",
            chord_progression_parser::Base::C,
            None,
            chord_progression_parser::ChordType::Diminished,
        ),
        (
            "F#M",
            chord_progression_parser::Base::F,
            Some(chord_progression_parser::Accidental::Sharp),
            chord_progression_parser::ChordType::Major,
        ),
        (
            "Bbm",
            chord_progression_parser::Base::B,
            Some(chord_progression_parser::Accidental::Flat),
            chord_progression_parser::ChordType::Minor,
        ),
    ] {
        for primary in primaries {
            for first in modifiers {
                for second in modifiers {
                    let source = format!("{head}{primary}{first}{second}(omit5,7)");
                    // An explicit M immediately followed by 7/9/11/13 forms a major extension token.
                    let primary = if head.ends_with('M') {
                        match primary {
                            "7" => "M7",
                            "9" => "M9",
                            "11" => "M11",
                            "13" => "M13",
                            value => value,
                        }
                    } else {
                        primary
                    };
                    let spellings: Vec<_> = [primary, first, second, "omit5", "7"]
                        .into_iter()
                        .filter(|value| !value.is_empty())
                        .collect();
                    let explicit = format!("{head}({})", spellings.join(","));
                    let mut extensions: Vec<_> = spellings
                        .iter()
                        .map(|value| Extension::from_str(value).unwrap())
                        .collect();
                    extensions.sort_by_key(extension_order::expected_rank);
                    let expected = ChordDetailed {
                        base: base.clone(),
                        accidental: accidental.clone(),
                        chord_type: chord_type.clone(),
                        extensions,
                    };
                    let chord = chord_from_source(&source);
                    assert_eq!(chord.plain, source);
                    assert_eq!(chord.detailed, expected, "{source}");
                    assert_eq!(
                        ChordDetailed::from_str(&source).unwrap(),
                        expected,
                        "{source}"
                    );
                    assert_eq!(
                        chord_from_source(&explicit).detailed,
                        expected,
                        "{explicit}"
                    );
                    let denominator = format!("G/{source}");
                    let ast = parse_chord_progression_string(&denominator).unwrap();
                    assert_eq!(format_chord_progression(&ast).unwrap(), denominator);
                    let report = parse_chord_progression_string_with_warnings(&source);
                    let expected_warnings = spellings
                        .iter()
                        .enumerate()
                        .filter(|(index, value)| spellings[..*index].contains(value))
                        .count();
                    assert_eq!(report.warnings.len(), expected_warnings, "{source}");
                    checked += 1;
                }
            }
        }
    }
    assert_eq!(checked, 26_010);
}

/** Rejects extra primary tokens, near-miss names, and invisible Unicode without accepting a prefix. */
#[test]
fn rejects_suffix_injection_and_keeps_recovery_positions_valid() {
    let fragments = [
        "0", "8", "10", "01", "maj7", "sus", "add2", "add4", "omit", "omit3", "no3", "alt", "o",
        "ø", "°", "△7", "♭9", "😀", "\0", "\u{200b}", "\u{2028}", "\u{feff}",
    ];
    for head in ["C", "Cm", "CM", "Caug", "Cdim", "C#", "Bbm"] {
        for fragment in fragments {
            let source = format!("{head}7{fragment}sus4");
            assert!(
                parse_chord_progression_string(&source).is_err(),
                "{source:?}"
            );
            assert!(ChordDetailed::from_str(&source).is_err(), "{source:?}");
            let document = format!("@section=😀\r\nC7(7)-{source}-D7(7)");
            let report = parse_chord_progression_string_with_warnings(&document);
            assert!(report.result.is_err());
            assert_eq!(report.warnings.len(), 2, "{document:?}");
            assert_eq!(support::assert_parse_invariants(&document), (false, 2));
        }
    }
    for first in [
        "1", "2", "3", "4", "5", "6", "7", "9", "11", "13", "M7", "M9", "M11", "M13",
    ] {
        for second in [
            "1", "2", "3", "4", "5", "6", "7", "9", "11", "13", "M7", "M9", "M11", "M13",
        ] {
            let source = format!("C{first}{second}");
            // Concatenating 1 and 1/3 deliberately forms the valid single tokens 11 and 13.
            let accepted = matches!(source.as_str(), "C11" | "C13");
            assert_eq!(
                parse_chord_progression_string(&source).is_ok(),
                accepted,
                "{source}"
            );
            assert_eq!(
                ChordDetailed::from_str(&source).is_ok(),
                accepted,
                "{source}"
            );
        }
    }
}

/** Exercises long atoms, malformed tails, and deeply nested denominators without recursive parsing. */
#[test]
fn handles_large_shorthand_atoms_and_deep_malformed_input() {
    let source = format!("@section=😀\r\nC7{}(7,sus4)/F#m7b5", "sus4".repeat(20_000));
    assert_eq!(support::assert_parse_invariants(&source), (true, 20_001));
    let malformed = format!("{source}😀-D7(7)");
    assert!(parse_chord_progression_string(&malformed).is_err());
    support::assert_parse_invariants(&malformed);
    let invalid_atom = format!("C7{}😀-D7(7)", "omit5".repeat(20_000));
    assert_eq!(support::assert_parse_invariants(&invalid_atom), (false, 1));
    let nested = format!("C/{}G7{}-D7(7)", "(".repeat(20_000), ")".repeat(20_000));
    assert_eq!(support::assert_parse_invariants(&nested), (false, 1));
}
