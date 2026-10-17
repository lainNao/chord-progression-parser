use chord_progression_parser::{
    format_chord_progression, parse_chord_progression_string,
    parse_chord_progression_string_with_warnings, ChordBlock,
};
use serde_json::json;

/** A blank line is layout inside one section, retaining its metadata and chord context. */
#[test]
fn blank_lines_keep_the_current_section_and_repetition_context() {
    let source = "@section=Verse\n@repeat=2\n[key=C]C-G\n\nAm-%";
    let ast = parse_chord_progression_string(source).expect("a blank line must not reset %");

    assert_eq!(ast.len(), 1);
    assert_eq!(
        json!(ast[0].meta_infos),
        json!([
            {"type": "section", "value": "Verse"},
            {"type": "repeat", "value": 2}
        ])
    );
    assert_eq!(
        ast[0]
            .chord_blocks
            .iter()
            .filter(|block| matches!(block, ChordBlock::Br))
            .count(),
        2
    );
    let formatted = format_chord_progression(&ast).expect("parsed AST must round-trip");
    assert_eq!(
        formatted,
        "@section=Verse\n@repeat=2\n[key=C]C - G\n\nAm - %"
    );
    assert_eq!(parse_chord_progression_string(&formatted).unwrap(), ast);
}

/** All supported line endings retain one empty line and collapse longer runs. */
#[test]
fn consecutive_blank_lines_collapse_to_one_inside_the_same_section() {
    for newline in ["\n", "\r", "\r\n"] {
        for count in [2, 3, 5, 1000] {
            let source = format!("[key=F]C{}%", format!("{newline} \t").repeat(count));
            let ast = parse_chord_progression_string(&source).expect("blank lines are layout");

            assert_eq!(ast.len(), 1, "split a section for {newline:?}, {count}");
            assert_eq!(ast[0].chord_blocks.len(), 4);
            assert_eq!(
                &ast[0].chord_blocks[1..3],
                &[ChordBlock::Br, ChordBlock::Br]
            );
            assert_eq!(format_chord_progression(&ast).unwrap(), "[key=F]C\n\n%");
        }
    }
}

/** Ordinary newlines remain distinct from a deliberately inserted blank line. */
#[test]
fn single_and_empty_lines_remain_distinct_after_formatting() {
    let source = "@section=Verse\nC\nG\n\nAm\n\n\nF\nC";
    let ast = parse_chord_progression_string(source).unwrap();
    assert_eq!(ast.len(), 1);
    assert_eq!(
        format_chord_progression(&ast).unwrap(),
        "@section=Verse\nC\nG\n\nAm\n\nF\nC"
    );
}

/** New section metadata still starts a section and resets its repetition reference. */
#[test]
fn explicit_section_boundaries_remain_independent_of_blank_lines() {
    for boundary in ["\n", "\n\n", "\n\n\n"] {
        let source = format!("@section=Verse\nC{boundary}@section=Chorus\nG");
        let ast = parse_chord_progression_string(&source).unwrap();
        assert_eq!(ast.len(), 2);
        assert_eq!(
            format_chord_progression(&ast).unwrap(),
            "@section=Verse\nC\n\n@section=Chorus\nG"
        );

        let repeated = format!("C{boundary}@section=Chorus\n%");
        let errors = parse_chord_progression_string(&repeated).unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error.code.to_string(), "CHB-1");
    }
    for source in ["%", "\n\n%", "@section=Verse\n\n%"] {
        let errors = parse_chord_progression_string(source).unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error.code.to_string(), "CHB-1");
    }
}

/** Previously stored anonymous sections require source-based migration, not silent merging. */
#[test]
fn legacy_anonymous_section_asts_are_not_silently_reinterpreted_by_the_formatter() {
    let mut ast = parse_chord_progression_string("@section=Verse\nC\n@section=Chorus\nG")
        .expect("explicit sections must parse");
    ast[1].meta_infos.clear();
    assert!(format_chord_progression(&ast).is_err());
}

/** Document edges and whitespace before the first chord keep their existing semantics. */
#[test]
fn blank_document_edges_do_not_create_empty_sections_or_chord_lines() {
    let source = "\n\n@section=Verse\n\n\nC\n\nG\n\n\n";
    let ast = parse_chord_progression_string(source).unwrap();
    assert_eq!(ast.len(), 1);
    assert_eq!(
        format_chord_progression(&ast).unwrap(),
        "@section=Verse\nC\n\nG"
    );
    assert!(parse_chord_progression_string("\n \t\r\n\r")
        .unwrap()
        .is_empty());
}

/** Collapsing layout in the AST must never shift diagnostics away from the original source. */
#[test]
fn blank_line_collapse_preserves_original_error_and_warning_offsets() {
    let source = "@section=😀\nC\n\n\nH-C(9,9)";
    let report = parse_chord_progression_string_with_warnings(source);
    let errors = report.result.expect_err("H remains invalid");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].position.line_number, 5);
    assert_eq!(
        errors[0].position.start_offset,
        source[..source.find('H').unwrap()].encode_utf16().count()
    );
    assert_eq!(report.warnings.len(), 1);
    assert_eq!(report.warnings[0].position.line_number, 5);
    assert_eq!(
        report.warnings[0].position.start_offset,
        source[..source.rfind('9').unwrap()].encode_utf16().count()
    );
}
