use chord_progression_parser::{
    format_chord_progression, parse_chord_progression_string, Ast, SectionMeta,
};
use serde_json::json;

/** A same directive stores a section-name reference without expanding any chords. */
#[test]
fn same_metadata_preserves_the_reference_name_without_resolving_it() {
    for name in ["Verse", "サビ", "曲😀", "0", "A♭"] {
        let source = format!("@same={name}");
        let ast = parse_chord_progression_string(&source).expect("same metadata must parse");
        assert_eq!(
            json!(ast),
            json!([{"metaInfos": [{"type": "same", "value": name}], "chordBlocks": []}])
        );
        assert_eq!(format_chord_progression(&ast).unwrap(), source);
    }
}

/** Names and repetition counts describe the referencing section without choosing a target. */
#[test]
fn same_metadata_can_have_a_section_name_and_repeat_count() {
    let source = "@section=Reprise\n@repeat=2\n@same=Verse";
    let ast = parse_chord_progression_string(source).unwrap();
    assert_eq!(
        json!(ast),
        json!([{
            "metaInfos": [
                {"type": "section", "value": "Reprise"},
                {"type": "repeat", "value": 2},
                {"type": "same", "value": "Verse"}
            ],
            "chordBlocks": []
        }])
    );
    assert_eq!(format_chord_progression(&ast).unwrap(), source);
    let serialized = serde_json::to_string(&ast).unwrap();
    assert_eq!(serde_json::from_str::<Ast>(&serialized).unwrap(), ast);
}

/** Duplicate names retain all definitions and the unexpanded reference in source order. */
#[test]
fn duplicate_section_names_do_not_cause_the_parser_to_choose_a_definition() {
    let source = "@section=Verse\nC\n@section=Verse\nG\n@same=Verse";
    let ast = parse_chord_progression_string(source).unwrap();
    assert_eq!(ast.len(), 3);
    assert_ne!(ast[0].chord_blocks, ast[1].chord_blocks);
    assert_eq!(
        json!(ast[2].meta_infos),
        json!([{"type": "same", "value": "Verse"}])
    );
    assert!(ast[2].chord_blocks.is_empty());
    let formatted = format_chord_progression(&ast).unwrap();
    assert_eq!(parse_chord_progression_string(&formatted).unwrap(), ast);
}

/** Missing and self-referencing targets remain application-level decisions. */
#[test]
fn undefined_and_self_references_are_kept_without_lookup() {
    for source in ["@same=Missing", "@section=Loop\n@same=Loop"] {
        let ast = parse_chord_progression_string(source).unwrap();
        assert_eq!(ast.len(), 1);
        assert!(ast[0].chord_blocks.is_empty());
        assert_eq!(format_chord_progression(&ast).unwrap(), source);
    }
}

/** A complete reference can be followed by another reference or a normally named section. */
#[test]
fn reference_only_sections_remain_separate_and_preserve_forward_and_cyclic_references() {
    let source =
        "@same=Later\n@same=Later\n@section=Later\nC\n@section=A\n@same=B\n@section=B\n@same=A";
    let ast = parse_chord_progression_string(source).unwrap();
    assert_eq!(ast.len(), 5);
    assert_eq!(ast[0], ast[1]);
    assert_eq!(ast[2].chord_blocks.len(), 1);
    for index in [0, 1, 3, 4] {
        assert!(ast[index].chord_blocks.is_empty());
    }
    let formatted = format_chord_progression(&ast).unwrap();
    assert_eq!(parse_chord_progression_string(&formatted).unwrap(), ast);
}

/** Repeat metadata may follow the reference; a subsequent section directive starts anew. */
#[test]
fn repeat_metadata_after_same_belongs_to_the_referencing_section() {
    let source = "@same=Verse\n@repeat=2\n@section=Chorus\nG";
    let ast = parse_chord_progression_string(source).unwrap();
    assert_eq!(ast.len(), 2);
    assert_eq!(
        json!(ast[0].meta_infos),
        json!([{"type": "same", "value": "Verse"}, {"type": "repeat", "value": 2}])
    );
    assert_eq!(
        format_chord_progression(&ast).unwrap(),
        "@same=Verse\n@repeat=2\n\n@section=Chorus\nG"
    );
}

/** Metadata whitespace and line endings use the same syntax as section-name declarations. */
#[test]
fn same_metadata_accepts_structural_whitespace_and_supported_line_endings() {
    for newline in ["\n", "\r", "\r\n"] {
        let source =
            format!(" @ section = Reprise {newline} @ same = 曲😀 {newline} @ repeat = 2 ");
        let ast = parse_chord_progression_string(&source).unwrap();
        assert_eq!(ast.len(), 1);
        assert_eq!(
            format_chord_progression(&ast).unwrap(),
            "@section=Reprise\n@same=曲😀\n@repeat=2"
        );
    }
}

/** Formatting must reject reference names that would inject syntax or lose AST information. */
#[test]
fn formatter_rejects_unrepresentable_reference_names() {
    for name in ["", "Verse Chorus", "Verse\n@section=Other", "Verse-C"] {
        let ast: Ast = serde_json::from_value(json!([{
            "metaInfos": [{"type": "same", "value": name}],
            "chordBlocks": []
        }]))
        .unwrap();
        assert!(format_chord_progression(&ast).is_err());
    }
}

/** Caller-created ASTs must follow the same no-body rule as parsed references. */
#[test]
fn formatter_rejects_reference_sections_with_chord_blocks() {
    let mut ast = parse_chord_progression_string("C").unwrap();
    ast[0]
        .meta_infos
        .push(SectionMeta::Same("Verse".to_owned()));
    assert!(format_chord_progression(&ast).is_err());
}

/** Same references follow the existing metadata value and dedicated-line syntax. */
#[test]
fn malformed_same_directives_report_existing_metadata_errors() {
    for (source, code, offset) in [
        ("@same", "SMIK-2", 5),
        ("@same=", "SMIV-1", 6),
        ("@same=\n", "SMIV-1", 6),
        ("@same=Verse extra", "SMIV-2", 12),
        ("@same=Verse-C", "SMIV-2", 11),
    ] {
        let errors = parse_chord_progression_string(source).expect_err("invalid same metadata");
        assert_eq!(errors.len(), 1, "unexpected diagnostics for {source:?}");
        assert_eq!(errors[0].error.code.to_string(), code);
        assert_eq!(errors[0].position.start_offset, offset);
    }
}

/** A reference covers the entire section, so chord lines cannot be mixed into it. */
#[test]
fn reference_sections_reject_chord_lines_with_their_original_source_positions() {
    for newline in ["\n", "\r", "\r\n"] {
        let source = format!("@section=曲😀{newline}@same=Verse{newline} C-G{newline} %{newline}@section=Next{newline}H");
        let errors =
            parse_chord_progression_string(&source).expect_err("reference section has a body");
        assert_eq!(errors.len(), 3);
        assert_eq!(errors[0].error.code.to_string(), "SMIV-4");
        assert_eq!(errors[0].position.line_number, 3);
        assert_eq!(errors[0].position.column_number, 2);
        assert_eq!(
            errors[0].position.start_offset,
            source[..source.find(" C-G").unwrap()]
                .encode_utf16()
                .count()
                + 1
        );
        assert_eq!(errors[1].error.code.to_string(), "SMIV-4");
        assert_eq!(errors[1].position.line_number, 4);
        assert_eq!(errors[2].error.code.to_string(), "CHO-1");
        assert_eq!(errors[2].position.line_number, 6);
    }
}

/** Reference-only sections cannot become a source of implicit previous-chord expansion. */
#[test]
fn a_reference_starts_a_new_section_and_does_not_establish_a_previous_chord() {
    let ast = parse_chord_progression_string("C\n@same=Verse").unwrap();
    assert_eq!(ast.len(), 2);
    assert!(ast[1].chord_blocks.is_empty());
    let errors = parse_chord_progression_string("C\n@same=Verse\n@section=Next\n%").unwrap_err();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].error.code.to_string(), "CHB-1");
}
