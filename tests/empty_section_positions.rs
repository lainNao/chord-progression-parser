use chord_progression_parser::{
    format_chord_progression, parse_chord_progression_string,
    parse_chord_progression_string_with_warnings, SectionMeta,
};

/** Named empty sections are real definitions, including an unfinished editor template. */
#[test]
fn consecutive_empty_named_sections_keep_their_own_identity() {
    let lines = [
        "@section=intro",
        "",
        "@section=verse@1",
        "",
        "@section=pre-chorus@1",
        "",
        "@section=chorus",
        "",
        "@section=interlude",
        "",
        "@section=verse",
        "@same=verse@1",
        "",
    ];
    for newline in ["\n", "\r", "\r\n"] {
        let source = lines.join(newline);
        let ast = parse_chord_progression_string(&source).unwrap();
        assert_eq!(ast.len(), 6);
        assert_eq!(ast[1].meta_infos, [SectionMeta::Section("verse@1".into())]);
        assert_eq!(
            ast[5].meta_infos,
            [
                SectionMeta::Section("verse".into()),
                SectionMeta::Same("verse@1".into())
            ]
        );
        assert!(ast.iter().all(|section| section.chord_blocks.is_empty()));
        let formatted = format_chord_progression(&ast).unwrap();
        assert_eq!(parse_chord_progression_string(&formatted).unwrap(), ast);
    }
}

/** Values keep exact UTF-16 offsets and Unicode scalar columns across all newline kinds. */
#[test]
fn metadata_source_map_addresses_values_and_ast_slots() {
    for newline in ["\n", "\r", "\r\n"] {
        let source = format!("  @section=😀 / 主題-A{newline}\t@section = \tverse@別😀  {newline}@repeat = 2{newline}\t@same = verse@別😀\t");
        let report = parse_chord_progression_string_with_warnings(&source);
        let ast = report.result.unwrap();
        let values = ["😀 / 主題-A", "verse@別😀", "2", "verse@別😀"];
        let indexes = [(0, 0), (1, 0), (1, 1), (1, 2)];
        assert_eq!(report.section_meta_positions.len(), values.len());
        let utf16: Vec<u16> = source.encode_utf16().collect();
        for ((entry, value), (section, meta)) in report
            .section_meta_positions
            .iter()
            .zip(values)
            .zip(indexes)
        {
            assert_eq!((entry.section_index, entry.meta_index), (section, meta));
            assert!(ast
                .get(section)
                .and_then(|s| s.meta_infos.get(meta))
                .is_some());
            let position = &entry.position;
            assert_eq!(
                String::from_utf16(&utf16[position.start_offset..position.end_offset]).unwrap(),
                value
            );
            assert_eq!(position.length, value.chars().count());
        }
        let last = &report.section_meta_positions[3].position;
        assert_eq!(last.line_number, 4);
        assert_eq!(last.column_number, 10);
    }
}
