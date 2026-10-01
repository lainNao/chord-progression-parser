use chord_progression_parser::{
    format_chord_progression, parse_chord_progression_string_with_warnings, Position,
};

/** Advances a deterministic state so failing mutations can be reproduced. */
fn next_random(state: &mut u64) -> usize {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state as usize
}

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

/** Exercises diagnostic ranges and successful round trips after mutating Unicode documents. */
#[test]
fn mutated_documents_keep_diagnostics_aligned_with_the_original_source() {
    let documents = [
        "C(9,9)-%\n\n@section=😀\n[key=E#]E#M(1,1)/あ😀(9,11),_-%",
        "@section=日本語\r\n@repeat=4294967295\r\n[key=Fb]C#dim(7,b5)/Gm(9,11)-?\r\nC(9,9,9)",
        "C/D(あ,😀)-[key=C#]Fbm(1,1)\r\r[key=E#]C(9,9)-%",
        "C-H\n\n%",
        "@section=😀\n\nC(9,9)",
        "C/D(9,11)-C(7,7)\n[key=Bb]C(1)-%",
    ];
    let alphabet: Vec<char> = "ACm79@[]()=,/-?%_ \t\n\rあ♭😀\0\u{2028}\u{200b}"
        .chars()
        .collect();
    let mut state = 0x4d59_5df4_d0f3_3173;
    let mut valid_documents = 0;
    let mut warnings = 0;

    for _ in 0..2_000 {
        let mut characters: Vec<char> = documents[next_random(&mut state) % documents.len()]
            .chars()
            .collect();
        for _ in 0..next_random(&mut state) % 9 {
            let index = next_random(&mut state) % (characters.len() + 1);
            match next_random(&mut state) % 3 {
                0 if index < characters.len() => {
                    characters.remove(index);
                }
                1 if index < characters.len() => {
                    characters[index] = alphabet[next_random(&mut state) % alphabet.len()];
                }
                _ => characters.insert(index, alphabet[next_random(&mut state) % alphabet.len()]),
            }
        }

        let input: String = characters.into_iter().collect();
        let source: Vec<u16> = input.encode_utf16().collect();
        let report = parse_chord_progression_string_with_warnings(&input);
        let mut previous_offset = 0;
        for warning in report.warnings {
            assert_source_position(&source, &warning.position);
            assert!(previous_offset <= warning.position.start_offset);
            previous_offset = warning.position.start_offset;
            warnings += 1;
        }

        match report.result {
            Ok(ast) => {
                let formatted = format_chord_progression(&ast)
                    .unwrap_or_else(|error| panic!("formatter rejected {input:?}: {error}"));
                assert_eq!(
                    parse_chord_progression_string_with_warnings(&formatted).result,
                    Ok(ast),
                    "round trip changed {input:?}"
                );
                valid_documents += 1;
            }
            Err(errors) => {
                assert!(!errors.is_empty());
                let mut previous_offset = 0;
                for error in errors {
                    assert_source_position(&source, &error.position);
                    assert!(previous_offset <= error.position.start_offset);
                    previous_offset = error.position.start_offset;
                }
            }
        }
    }

    assert!(
        valid_documents >= 100,
        "corpus must exercise successful parses"
    );
    assert!(
        warnings >= 100,
        "corpus must exercise warning source ranges"
    );
}
