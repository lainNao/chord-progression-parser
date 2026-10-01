use chord_progression_parser::{
    parse_chord_progression_string, parse_chord_progression_string_with_warnings,
};

/** Resets repetition context at section boundaries even when an earlier line failed. */
#[test]
fn failed_chord_lines_do_not_leak_repetition_context_into_new_sections() {
    for boundary in ["\n\n", "\r\n\r\n", "\n@section=Next\n", "\n@repeat=2\n"] {
        let source = format!("C-H{boundary}%");
        let errors = parse_chord_progression_string(&source).expect_err("H is invalid");
        assert_eq!(
            errors
                .iter()
                .map(|error| error.error.code.to_string())
                .collect::<Vec<_>>(),
            ["CHO-1", "CHB-1"],
            "missing section-boundary diagnostic for {source:?}"
        );
        let repetition = &errors[1];
        assert_eq!(
            repetition.position.start_offset,
            source.encode_utf16().count() - 1
        );
        assert_eq!(repetition.position.length, 1);
    }
}

/** Retains valid preceding expressions across one line break within the same section. */
#[test]
fn failed_lines_keep_repetition_context_inside_their_own_section() {
    let errors = parse_chord_progression_string("C-H\n%")
        .expect_err("an invalid chord must remain an error");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].error.code.to_string(), "CHO-1");
}

/** Recovers at bar separators instead of interpreting the next chord as an extension. */
#[test]
fn malformed_extension_lists_do_not_swallow_later_bars() {
    for (source, expected_codes) in [
        ("C(9,-H", vec!["EXT-2", "EXT-3", "CHO-1"]),
        ("C(9,-C(999)", vec!["EXT-2", "EXT-3", "EXT-1"]),
        ("C(9,-C(9)", vec!["EXT-2", "EXT-3"]),
    ] {
        let report = parse_chord_progression_string_with_warnings(source);
        let errors = report
            .result
            .expect_err("the first extension list is malformed");
        assert_eq!(
            errors
                .iter()
                .map(|error| error.error.code.to_string())
                .collect::<Vec<_>>(),
            expected_codes,
            "later-bar diagnostics were lost for {source:?}"
        );
        assert!(report.warnings.is_empty(), "false warning for {source:?}");
    }
}

/** Duplicate warnings belong to the recovered chord's own extension list. */
#[test]
fn recovered_bar_warnings_use_their_own_extension_context() {
    let source = "C(9,-C(9,9)";
    let report = parse_chord_progression_string_with_warnings(source);
    assert_eq!(
        report
            .result
            .expect_err("the first list is malformed")
            .len(),
        2
    );
    assert_eq!(report.warnings.len(), 1);
    assert_eq!(report.warnings[0].position.start_offset, 9);
    assert_eq!(report.warnings[0].additional_info.as_deref(), Some("9"));
}
