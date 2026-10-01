use chord_progression_parser::{
    format_chord_progression, parse_chord_progression_string,
    parse_chord_progression_string_with_warnings, Chord, ChordBlock, ChordDetailed,
    ChordExpression, ChordInfo, Position, Section,
};

/** Checks display coordinates against the exact UTF-16 range exposed to JavaScript. */
fn assert_source_position(source: &[u16], position: &Position) {
    assert!(position.start_offset <= position.end_offset);
    assert!(position.end_offset <= source.len());
    let before = String::from_utf16(&source[..position.start_offset])
        .expect("diagnostic start must not split a surrogate pair");
    let fragment = String::from_utf16(&source[position.start_offset..position.end_offset])
        .expect("diagnostic end must not split a surrogate pair");
    let line_number = before.matches('\r').count() + before.matches('\n').count()
        - before.matches("\r\n").count()
        + 1;
    let column_number = before
        .rsplit(['\r', '\n'])
        .next()
        .expect("split always contains a line")
        .chars()
        .count()
        + 1;

    assert_eq!(position.line_number, line_number);
    assert_eq!(position.column_number, column_number);
    assert_eq!(position.length, fragment.chars().count());
}

/** Checks source ranges and AST round trips for both mutation tests and guided fuzzing. */
pub fn assert_parse_invariants(input: &str) -> (bool, usize) {
    let source: Vec<u16> = input.encode_utf16().collect();
    let report = parse_chord_progression_string_with_warnings(input);
    assert_eq!(report.result, parse_chord_progression_string(input));
    // The standalone chord parser must agree with the document parser on every accepted input.
    if let Ok(detailed) = input.parse::<ChordDetailed>() {
        let expected = vec![Section {
            meta_infos: Vec::new(),
            chord_blocks: vec![ChordBlock::Bar(vec![ChordInfo {
                meta_infos: Vec::new(),
                denominator: None,
                chord_expression: ChordExpression::Chord(Chord {
                    plain: input.to_string(),
                    detailed,
                }),
            }])],
        }];
        assert_eq!(report.result.as_ref().ok(), Some(&expected));
    }
    let mut previous_offset = 0;
    for warning in &report.warnings {
        assert_source_position(&source, &warning.position);
        assert!(previous_offset <= warning.position.start_offset);
        previous_offset = warning.position.start_offset;
        let fragment =
            String::from_utf16(&source[warning.position.start_offset..warning.position.end_offset])
                .expect("warning range must contain whole characters");
        assert_eq!(warning.additional_info.as_deref(), Some(fragment.as_str()));
    }
    let warning_count = report.warnings.len();

    match report.result {
        Ok(ast) => {
            let formatted = format_chord_progression(&ast)
                .unwrap_or_else(|error| panic!("formatter rejected {input:?}: {error}"));
            let reparsed = parse_chord_progression_string_with_warnings(&formatted);
            assert_eq!(reparsed.result, Ok(ast), "round trip changed {input:?}");
            assert_eq!(
                reparsed
                    .warnings
                    .iter()
                    .map(|warning| (&warning.code, &warning.additional_info))
                    .collect::<Vec<_>>(),
                report
                    .warnings
                    .iter()
                    .map(|warning| (&warning.code, &warning.additional_info))
                    .collect::<Vec<_>>(),
                "formatting changed warnings for {input:?}"
            );
            (true, warning_count)
        }
        Err(errors) => {
            assert!(!errors.is_empty());
            let mut previous_offset = 0;
            for error in errors {
                assert_source_position(&source, &error.position);
                assert!(previous_offset <= error.position.start_offset);
                previous_offset = error.position.start_offset;
            }
            (false, warning_count)
        }
    }
}
