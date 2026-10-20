use chord_progression_parser::{
    format_chord_progression, parse_chord_progression_string,
    parse_chord_progression_string_with_warnings, ChordBlock,
};

/** Keeps known bass spellings and explicit unknowns stable through AST formatting. */
#[test]
fn accepts_complete_chords_and_unidentified_basses() {
    for denominator in ["G", "F#", "Bb", "?", "F#m(7,b5)", "G(9,9)"] {
        let source = format!("[key=C]C/{denominator} - % - %/?");
        let ast = parse_chord_progression_string(&source).expect("valid bass must parse");
        let ChordBlock::Bar(bar) = &ast[0].chord_blocks[0] else {
            panic!("expected a bar");
        };
        assert_eq!(bar[0].denominator.as_deref(), Some(denominator));
        assert_eq!(format_chord_progression(&ast).unwrap(), source);
    }
    let ast = parse_chord_progression_string("C / F#m ( 7 , b5 )").unwrap();
    assert_eq!(format_chord_progression(&ast).unwrap(), "C/F#m(7,b5)");
}

/** Rejects arbitrary text, numeric basses, and malformed complete chord spellings. */
#[test]
fn rejects_invalid_denominators_without_guessing_a_note() {
    for denominator in [
        "5",
        "Unknown",
        "Garbage",
        "G7",
        "G##",
        "Gbb",
        "G(unknown)",
        "G()",
        "G(7)(9)",
        "G(7,)",
        "?m",
        "?(7)",
        "%",
        "_",
        "g",
        "😀あ",
        "G m",
        "G #",
        "? ?",
        "G(1 1)",
    ] {
        let source = format!("C/{denominator}");
        let errors = parse_chord_progression_string(&source).expect_err(&source);
        assert_eq!(errors.len(), 1, "{source}");
        assert_eq!(errors[0].error.code.to_string(), "DEN-1", "{source}");
        assert_eq!(errors[0].position.column_number, 2, "{source}");
        assert_eq!(errors[0].position.length, denominator.chars().count() + 1);
        assert_eq!(errors[0].position.start_offset, 1);
        assert_eq!(errors[0].position.end_offset, source.encode_utf16().count());
    }
}

/** Invalid basses retain UTF-16 positions and do not suppress later errors or warnings. */
#[test]
fn reports_original_ranges_and_recovers_at_chord_boundaries() {
    for separator in [",", "-", "\n", "\r", "\r\n"] {
        let source = format!("@section=😀\r\nC/5{separator}D/Unknown{separator}E(9,9)");
        let report = parse_chord_progression_string_with_warnings(&source);
        let errors = report.result.expect_err("invalid basses must be reported");
        assert_eq!(errors.len(), 2, "{source}");
        for (error, fragment) in errors.iter().zip(["/5", "/Unknown"]) {
            let start_byte = source.find(fragment).unwrap();
            let offset = source[..start_byte].encode_utf16().count();
            assert_eq!(error.error.code.to_string(), "DEN-1");
            assert_eq!(error.position.start_offset, offset);
            assert_eq!(error.position.end_offset, offset + fragment.len());
        }
        assert_eq!(report.warnings.len(), 1, "{source}");
    }
}

/** Externally supplied ASTs with invalid basses must not format as valid source. */
#[test]
fn formatter_rejects_invalid_denominators() {
    let mut ast = parse_chord_progression_string("C/?").unwrap();
    let ChordBlock::Bar(bar) = &mut ast[0].chord_blocks[0] else {
        panic!("expected a bar");
    };
    bar[0].denominator = Some("5".to_owned());
    assert!(format_chord_progression(&ast).is_err());
}
