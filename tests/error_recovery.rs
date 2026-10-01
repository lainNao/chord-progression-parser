use chord_progression_parser::parse_chord_progression_string;

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
