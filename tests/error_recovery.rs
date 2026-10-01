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

/** Commas inside rejected chords must not establish repetition context. */
#[test]
fn rejected_parenthesized_values_are_not_recovered_as_chords() {
    for (source, first_code) in [
        ("H(9,C)-%", "CHO-1"),
        ("[bad=C]D(9,C)-%", "CIMK-3"),
        ("C(7)(9,C)-%", "EXT-4"),
        ("C(9 9,C)-%", "EXT-3"),
    ] {
        let errors = parse_chord_progression_string(source).expect_err("invalid chord");
        assert_eq!(
            errors
                .iter()
                .map(|error| error.error.code.to_string())
                .collect::<Vec<_>>(),
            [first_code, "CHB-1"],
            "parenthesized text became a chord in {source:?}"
        );
        assert_eq!(errors[1].position.start_offset, source.len() - 1);
    }
}

/** Recovery skips nested list commas, but resumes at real chord and bar separators. */
#[test]
fn rejected_chord_lists_do_not_emit_spurious_warnings_or_hide_later_errors() {
    for source in ["H(9,C(9,9)),I-D(11,11)", "H(9,C(9,9)-I-D(11,11)"] {
        let report = parse_chord_progression_string_with_warnings(source);
        let errors = report.result.expect_err("H and I are invalid");
        assert_eq!(
            errors
                .iter()
                .map(|error| error.error.code.to_string())
                .collect::<Vec<_>>(),
            ["CHO-1", "CHO-1"],
            "unexpected recovery for {source:?}"
        );
        assert_eq!(errors[1].position.start_offset, source.find('I').unwrap());
        assert_eq!(report.warnings.len(), 1, "false warning for {source:?}");
        assert_eq!(report.warnings[0].additional_info.as_deref(), Some("11"));
        assert_eq!(
            report.warnings[0].position.start_offset,
            source.rfind("11").unwrap()
        );
    }
}

/** Invalid metadata contents cannot become chords, even through nested delimiters. */
#[test]
fn rejected_metadata_does_not_establish_repetition_or_warn() {
    for prefix in [
        "[key=C,D]E",
        "[key=C,D(9,9)]E",
        "[key=C,[D(9,9)]]E",
        "[key=C,D(9,9)",
    ] {
        let source = format!("{prefix}-%-D(11,11)");
        let report = parse_chord_progression_string_with_warnings(&source);
        let errors = report.result.expect_err("metadata is malformed");
        assert_eq!(
            errors
                .iter()
                .map(|error| error.error.code.to_string())
                .collect::<Vec<_>>(),
            ["CIMV-3", "CHB-1"],
            "unexpected recovery for {source:?}"
        );
        assert_eq!(report.warnings.len(), 1, "false warning for {source:?}");
        assert_eq!(
            report.warnings[0].position.start_offset,
            source.rfind("11").unwrap()
        );
    }
}

/** Nested invalid values are skipped whole before resuming the outer extension list. */
#[test]
fn nested_invalid_extensions_do_not_generate_duplicate_warnings() {
    for nested in ["(11,9)", "((11,9),9)", "[11,9]", "([11,9])"] {
        let source = format!("C(9,{nested},9)-D(11,11)");
        let report = parse_chord_progression_string_with_warnings(&source);
        let errors = report.result.expect_err("nested lists are invalid");
        assert_eq!(errors.len(), 1, "unexpected errors for {source:?}");
        assert_eq!(errors[0].error.code.to_string(), "EXT-2");
        assert_eq!(
            report
                .warnings
                .iter()
                .map(|warning| warning.position.start_offset)
                .collect::<Vec<_>>(),
            [source.find(")-D").unwrap() - 1, source.rfind("11").unwrap()],
            "warnings must refer to outer list values for {source:?}"
        );
    }
}

/** Opaque denominator brackets must not hide later independent chord errors. */
#[test]
fn denominator_text_does_not_leak_bracket_depth_into_recovery() {
    for denominator in ["(x[y)", "(x]y)", "([x[y)"] {
        for separator in [",", "-"] {
            let source = format!("C/{denominator}{separator}H,I-D(11,11)");
            let report = parse_chord_progression_string_with_warnings(&source);
            let errors = report.result.expect_err("H and I are invalid");
            assert_eq!(
                errors
                    .iter()
                    .map(|error| error.error.code.to_string())
                    .collect::<Vec<_>>(),
                ["CHO-1", "CHO-1"],
                "opaque denominator changed recovery for {source:?}"
            );
            assert_eq!(report.warnings.len(), 1);
            assert_eq!(
                report.warnings[0].position.start_offset,
                source.rfind("11").unwrap()
            );
        }
    }
}

/** Deep malformed nesting still recovers at bars and all supported line endings. */
#[test]
fn nested_recovery_preserves_hard_boundaries() {
    for depth in [1, 2, 8, 64, 512] {
        for opening in ["(", "[", "(["] {
            for boundary in ["-", "\n", "\r", "\r\n"] {
                let source = format!("C(9,{}9,9{boundary}%-D(11,11)", opening.repeat(depth));
                let report = parse_chord_progression_string_with_warnings(&source);
                let errors = report.result.expect_err("unclosed nested extension");
                assert_eq!(
                    errors
                        .iter()
                        .map(|error| error.error.code.to_string())
                        .collect::<Vec<_>>(),
                    ["EXT-2", "EXT-3", "CHB-1"],
                    "lost hard boundary for depth {depth}, {opening:?}, {boundary:?}"
                );
                assert_eq!(report.warnings.len(), 1);
                assert_eq!(
                    report.warnings[0].position.start_offset,
                    source.rfind("11").unwrap()
                );
            }
        }
    }
}
