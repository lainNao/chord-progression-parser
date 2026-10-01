use chord_progression_parser::{
    format_chord_progression, parse_chord_progression_string,
    parse_chord_progression_string_with_warnings, Chord, ChordBlock, ChordDetailed,
    ChordExpression, ChordInfo, Position, Section,
};

/** Describes one Unicode scalar boundary in the original source. */
struct SourceCoordinate {
    line: usize,
    column: usize,
    scalar_index: usize,
}

/** Indexes UTF-16 boundaries once so checking many diagnostics stays linear. */
fn source_coordinates(input: &str) -> Vec<Option<SourceCoordinate>> {
    let mut coordinates = Vec::with_capacity(input.encode_utf16().count() + 1);
    let mut line = 1;
    let mut column = 1;
    let mut previous_was_cr = false;
    coordinates.push(Some(SourceCoordinate {
        line,
        column,
        scalar_index: 0,
    }));

    for (index, character) in input.chars().enumerate() {
        if character.len_utf16() == 2 {
            // No diagnostic boundary may split an astral character's surrogate pair.
            coordinates.push(None);
        }
        match character {
            '\r' => {
                line += 1;
                column = 1;
            }
            '\n' => {
                line += usize::from(!previous_was_cr);
                column = 1;
            }
            _ => column += 1,
        }
        previous_was_cr = character == '\r';
        coordinates.push(Some(SourceCoordinate {
            line,
            column,
            scalar_index: index + 1,
        }));
    }
    coordinates
}

/** Checks display coordinates and scalar length at exact UTF-16 source boundaries. */
fn assert_source_position(coordinates: &[Option<SourceCoordinate>], position: &Position) {
    assert!(position.start_offset <= position.end_offset);
    let start = coordinates
        .get(position.start_offset)
        .and_then(Option::as_ref)
        .expect("diagnostic start must be a Unicode boundary inside the source");
    let end = coordinates
        .get(position.end_offset)
        .and_then(Option::as_ref)
        .expect("diagnostic end must be a Unicode boundary inside the source");
    assert_eq!(position.line_number, start.line);
    assert_eq!(position.column_number, start.column);
    assert_eq!(position.length, end.scalar_index - start.scalar_index);
}

/** Checks source ranges and AST round trips for both mutation tests and guided fuzzing. */
pub fn assert_parse_invariants(input: &str) -> (bool, usize) {
    let source: Vec<u16> = input.encode_utf16().collect();
    let coordinates = source_coordinates(input);
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
        assert_source_position(&coordinates, &warning.position);
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
                assert_source_position(&coordinates, &error.position);
                assert!(previous_offset <= error.position.start_offset);
                previous_offset = error.position.start_offset;
            }
            (false, warning_count)
        }
    }
}
