use chord_progression_parser::{
    format_chord_progression, parse_chord_progression_string,
    parse_chord_progression_string_with_warnings, Extension, WarningCode,
};
use strum::VariantNames;

/** Warns on every repeated occurrence while preserving the AST and legacy Rust API. */
#[test]
fn preserves_duplicates_and_warns_at_each_repeated_extension() {
    let input = "C(9,11,9,9)";
    let report = parse_chord_progression_string_with_warnings(input);
    assert_eq!(report.result, parse_chord_progression_string(input));
    assert_eq!(report.warnings.len(), 2);
    for (warning, offset) in report.warnings.iter().zip([7, 9]) {
        assert_eq!(warning.code, WarningCode::DuplicateExtension);
        assert_eq!(warning.code.to_string(), "DUPLICATE_EXTENSION");
        assert_eq!(warning.additional_info.as_deref(), Some("9"));
        assert_eq!(warning.position.start_offset, offset);
        assert_eq!(warning.position.end_offset, offset + 1);
        assert_eq!(warning.position.column_number, offset + 1);
        assert_eq!(warning.position.length, 1);
    }
    let ast = report.result.expect("duplicates must not be errors");
    assert_eq!(format_chord_progression(&ast).unwrap(), input);
}

/** Handles every canonical extension, including case-sensitive and altered spellings. */
#[test]
fn warns_on_all_extension_values_but_not_related_pitches_or_other_chords() {
    for extension in Extension::VARIANTS {
        let input = format!("C({extension},{extension})");
        let report = parse_chord_progression_string_with_warnings(&input);
        assert!(report.result.is_ok(), "{input}");
        assert_eq!(report.warnings.len(), 1, "{input}");
        assert_eq!(
            report.warnings[0].additional_info.as_deref(),
            Some(*extension)
        );
    }
    for input in ["", "C(9)-D(9)", "C(9,add9,M9,#9,b9)", "C(1,11)", "C/G(9,9)"] {
        let report = parse_chord_progression_string_with_warnings(input);
        assert!(report.result.is_ok(), "{input}");
        assert!(report.warnings.is_empty(), "{input}");
    }
}

/** Retains independently encountered warnings when error recovery rejects the document. */
#[test]
fn reports_errors_and_warnings_without_mixing_their_codes() {
    let input = "H-C(9,9,111,9)-D(11,11)";
    let report = parse_chord_progression_string_with_warnings(input);
    assert_eq!(report.result, parse_chord_progression_string(input));
    let errors = report.result.expect_err("unknown tokens must still fail");
    assert_eq!(errors.len(), 2);
    assert_eq!(errors[0].error.code.to_string(), "CHO-1");
    assert_eq!(errors[1].error.code.to_string(), "EXT-1");
    assert_eq!(report.warnings.len(), 3);
    assert_eq!(
        report
            .warnings
            .iter()
            .map(|warning| warning.position.start_offset)
            .collect::<Vec<_>>(),
        [6, 12, 20],
    );
    for input in [
        "?(9,9)",
        "_(9,9)",
        "C-%(9,9)",
        "C(unknown,unknown)",
        "H(9,9)",
    ] {
        let report = parse_chord_progression_string_with_warnings(input);
        assert!(report.result.is_err(), "{input}");
        assert!(report.warnings.is_empty(), "{input}");
    }
    let unclosed = parse_chord_progression_string_with_warnings("C(9,9");
    assert!(unclosed.result.is_err());
    assert_eq!(unclosed.warnings.len(), 1);
}

/** Uses original CRLF, whitespace, and UTF-16 ranges rather than formatted AST offsets. */
#[test]
fn preserves_warning_positions_after_unicode_and_line_breaks() {
    let input = "@section=😀\r\n[key=C] C(9, 9) - D(M9,M9)";
    let report = parse_chord_progression_string_with_warnings(input);
    assert!(report.result.is_ok());
    assert_eq!(report.warnings.len(), 2);
    for (warning, value) in report.warnings.iter().zip(["9", "M9"]) {
        let byte_offset = if value == "9" {
            input.find(" 9)").unwrap() + 1
        } else {
            input.rfind(value).unwrap()
        };
        let offset = input[..byte_offset].encode_utf16().count();
        assert_eq!(warning.position.line_number, 2);
        assert_eq!(warning.position.start_offset, offset);
        assert_eq!(warning.position.end_offset, offset + value.len());
        assert_eq!(warning.position.length, value.len());
    }
}

/** Keeps repeated-extension analysis linear even when a late extension repeats many times. */
#[test]
fn handles_long_duplicate_lists() {
    let input = format!(
        "C({},{} )",
        vec!["9"; 10_000].join(","),
        vec!["M9"; 10_000].join(",")
    );
    let report = parse_chord_progression_string_with_warnings(&input);
    assert!(report.result.is_ok());
    assert_eq!(report.warnings.len(), 19_998);
}
