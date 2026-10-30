use chord_progression_parser::{
    format_chord_progression, parse_chord_progression_string, SectionMeta,
};

/** Names retain punctuation and internal whitespace without interpreting chord or metadata syntax. */
#[test]
fn section_and_reference_names_are_literal_line_values() {
    for name in [
        "pre-chorus",
        "Verse extra",
        "提示部 / 第1主題 (A♭) 😀",
        "A\t  B",
        "@repeat=2",
        "[key=C] C-G, (verse) = chorus",
        "-",
        "=",
        "#intro; // not a comment",
        "\u{3000}A\u{3000}",
    ] {
        for newline in ["\n", "\r", "\r\n"] {
            let source = format!(" @ section = \t{name} \t{newline}C{newline}@same=\t{name}  ");
            let ast = parse_chord_progression_string(&source).unwrap();
            assert_eq!(ast.len(), 2, "{source:?}");
            assert_eq!(
                ast[0].meta_infos,
                vec![SectionMeta::Section(name.to_owned())]
            );
            assert_eq!(ast[1].meta_infos, vec![SectionMeta::Same(name.to_owned())]);
            assert!(ast[1].chord_blocks.is_empty());
            let formatted = format_chord_progression(&ast).unwrap();
            assert_eq!(formatted, format!("@section={name}\nC\n\n@same={name}"));
            assert_eq!(parse_chord_progression_string(&formatted).unwrap(), ast);
        }
    }
}

/** Metadata-looking text on the same line remains part of the name; the next line keeps its role. */
#[test]
fn literal_names_do_not_swallow_or_inject_metadata_lines() {
    let ast = parse_chord_progression_string(
        "@section=verse @same=chorus\nC\n@same=verse @repeat=2\n@repeat=3\n@section=outro\nG",
    )
    .unwrap();
    assert_eq!(ast.len(), 3);
    assert_eq!(
        ast[0].meta_infos,
        vec![SectionMeta::Section("verse @same=chorus".into())]
    );
    assert_eq!(
        ast[1].meta_infos,
        vec![
            SectionMeta::Same("verse @repeat=2".into()),
            SectionMeta::Repeat(3)
        ]
    );
    assert_eq!(
        ast[2].meta_infos,
        vec![SectionMeta::Section("outro".into())]
    );
}

/** Empty names stay invalid, with diagnostics at the unchanged line ending or EOF boundary. */
#[test]
fn empty_literal_names_report_the_original_boundary() {
    for key in ["section", "same"] {
        for suffix in ["", " \t", "\n", " \t\r\n"] {
            let source = format!("@{key}={suffix}");
            let errors = parse_chord_progression_string(&source).unwrap_err();
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].error.code.to_string(), "SMIV-1");
            let boundary = source.find(['\r', '\n']).unwrap_or(source.len());
            assert_eq!(errors[0].position.start_offset, boundary);
        }
    }
}

/** Raw UTF-8 slicing must not change public UTF-16 offsets after Unicode names and CRLF. */
#[test]
fn unicode_literal_names_keep_later_error_positions() {
    let source = "@section=😀 / 主題-A\r\nH\r\n@same=😀 / 主題-A\r\nC";
    let errors = parse_chord_progression_string(source).unwrap_err();
    assert_eq!(errors.len(), 2);
    for (error, (code, line, marker)) in errors.iter().zip([("CHO-1", 2, "H"), ("SMIV-4", 4, "C")])
    {
        let offset = source[..source.rfind(marker).unwrap()]
            .encode_utf16()
            .count();
        assert_eq!(error.error.code.to_string(), code);
        assert_eq!(error.position.line_number, line);
        assert_eq!(error.position.column_number, 1);
        assert_eq!(error.position.start_offset, offset);
        assert_eq!(error.position.end_offset, offset + 1);
    }
}
