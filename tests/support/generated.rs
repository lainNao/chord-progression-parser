use std::{fmt::Write, str::FromStr};

use chord_progression_parser::{
    parse_chord_progression_string_with_warnings, Accidental, Ast, Base, Chord, ChordBlock,
    ChordDetailed, ChordExpression, ChordInfo, ChordInfoMeta, ChordType, Extension, Key, Section,
    SectionMeta,
};
use chord_progression_parser::{ErrorCode, ParseWarning, WarningCode};
use strum::VariantNames;

/** Reuses a non-empty byte stream to make bounded, reproducible grammar choices. */
struct Choices<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl Choices<'_> {
    /** Selects an index while allowing even short fuzz inputs to form complete documents. */
    fn index(&mut self, length: usize) -> usize {
        let value = self.bytes[self.cursor % self.bytes.len()];
        self.cursor += 1;
        usize::from(value) % length
    }

    /** Varies whitespace only where the documented grammar permits it. */
    fn gap(&mut self) -> &'static str {
        ["", " ", "\t", " \t "][self.index(4)]
    }
}

/** Builds a chord's source spelling and model independently of the formatter. */
fn chord(choices: &mut Choices<'_>) -> (String, ChordExpression) {
    let (base, root) = [
        (Base::A, "A"),
        (Base::B, "B"),
        (Base::C, "C"),
        (Base::D, "D"),
        (Base::E, "E"),
        (Base::F, "F"),
        (Base::G, "G"),
    ][choices.index(7)]
    .clone();
    let (accidental, alteration) = [
        (None, ""),
        (Some(Accidental::Sharp), "#"),
        (Some(Accidental::Flat), "b"),
    ][choices.index(3)]
    .clone();
    let (chord_type, suffix) = [
        (ChordType::Major, ""),
        (ChordType::Major, "M"),
        (ChordType::Minor, "m"),
        (ChordType::Augmented, "aug"),
        (ChordType::Diminished, "dim"),
    ][choices.index(5)]
    .clone();
    let head = format!("{root}{alteration}{suffix}");
    let mut source = head.clone();
    let mut plain = head;
    let mut extensions = Vec::new();
    let count = choices.index(6);
    if count > 0 {
        source.push_str(choices.gap());
        source.push('(');
        plain.push('(');
        for index in 0..count {
            if index > 0 {
                source.push_str(choices.gap());
                source.push(',');
                plain.push(',');
            }
            source.push_str(choices.gap());
            let spelling = Extension::VARIANTS[choices.index(Extension::VARIANTS.len())];
            source.push_str(spelling);
            plain.push_str(spelling);
            extensions.push(Extension::from_str(spelling).expect("declared extension must parse"));
        }
        source.push_str(choices.gap());
        source.push(')');
        plain.push(')');
    }
    (
        source,
        ChordExpression::Chord(Chord {
            plain,
            detailed: ChordDetailed {
                base,
                accidental,
                chord_type,
                extensions,
            },
        }),
    )
}

/** Builds metadata, special expressions, and opaque denominators for one chord. */
fn chord_info(choices: &mut Choices<'_>, has_prior: bool) -> (String, ChordInfo) {
    let mut source = String::new();
    let mut meta_infos = Vec::new();
    for _ in 0..choices.index(3) {
        let (key, spelling) = [
            (Key::C_M, "C"),
            (Key::Fb_M, "Fb"),
            (Key::Es_m, "E#m"),
            (Key::Bb_m, "Bbm"),
            (Key::Fs_M, "F#"),
        ][choices.index(5)]
        .clone();
        write!(
            source,
            "[{}key{}={}{}{}]{}",
            choices.gap(),
            choices.gap(),
            choices.gap(),
            spelling,
            choices.gap(),
            choices.gap()
        )
        .unwrap();
        meta_infos.push(ChordInfoMeta::Key(key));
    }
    let (spelling, chord_expression) = match choices.index(8) {
        0 => ("?".to_owned(), ChordExpression::UnIdentified),
        1 => ("_".to_owned(), ChordExpression::NoChord),
        2 if has_prior => ("%".to_owned(), ChordExpression::Same),
        _ => chord(choices),
    };
    source.push_str(&spelling);
    let denominator = if choices.index(3) == 0 {
        let value = [
            "G",
            "F#m(7,b5)",
            "😀あ",
            "(x[y)",
            "Bb",
            "%",
            "_",
            "?",
            "G(9,9)",
        ][choices.index(9)];
        write!(source, "{}/{}{value}", choices.gap(), choices.gap()).unwrap();
        Some(value.to_owned())
    } else {
        None
    };
    (
        source,
        ChordInfo {
            meta_infos,
            chord_expression,
            denominator,
        },
    )
}

/** Generates valid source and an expected AST without using parsing or formatting as a builder. */
fn document(bytes: &[u8]) -> (String, Ast) {
    assert!(!bytes.is_empty());
    let mut choices = Choices { bytes, cursor: 0 };
    let newline = ["\n", "\r", "\r\n"][choices.index(3)];
    let mut source = String::new();
    let mut ast = Vec::new();
    for section_index in 0..1 + choices.index(4) {
        if section_index > 0 {
            source.push_str(&newline.repeat(1 + choices.index(3)));
        }
        let name = ["Intro", "日本語", "😀", "\0", "A♭", "x\u{2028}y"][choices.index(6)];
        write!(
            source,
            "{}@{}section{}={}{}{}{newline}",
            choices.gap(),
            choices.gap(),
            choices.gap(),
            choices.gap(),
            name,
            choices.gap()
        )
        .unwrap();
        let mut section = Section {
            meta_infos: vec![SectionMeta::Section(name.to_owned())],
            chord_blocks: Vec::new(),
        };
        if choices.index(2) == 0 {
            let count = [0, 1, 2, u32::MAX][choices.index(4)];
            write!(source, "@repeat={count}{newline}").unwrap();
            section.meta_infos.push(SectionMeta::Repeat(count));
        }
        // Blank lines before the first chord keep the metadata in the same section.
        if choices.index(2) == 0 {
            source.push_str(&newline.repeat(2));
        }
        let mut has_prior = false;
        for bar_index in 0..1 + choices.index(5) {
            if bar_index > 0 {
                if choices.index(2) == 0 {
                    source.push_str(newline);
                    section.chord_blocks.push(ChordBlock::Br);
                } else {
                    source.push('-');
                }
            }
            let mut bar = Vec::new();
            for chord_index in 0..1 + choices.index(4) {
                if chord_index > 0 {
                    source.push(',');
                }
                source.push_str(choices.gap());
                let (spelling, info) = chord_info(&mut choices, has_prior);
                source.push_str(&spelling);
                source.push_str(choices.gap());
                bar.push(info);
                has_prior = true;
            }
            if choices.index(2) == 0 {
                source.push(',');
            }
            section.chord_blocks.push(ChordBlock::Bar(bar));
        }
        ast.push(section);
    }
    source.push_str(newline);
    (source, ast)
}

/** Checks that one malformed suffix preserves earlier warnings and later recovery context. */
fn assert_recovery(source: &str, warnings: &[ParseWarning], bytes: &[u8]) {
    let (invalid, first_error, extra_warning) = [
        ("H", ErrorCode::Cho1, false),
        ("H(9,C(9,9))", ErrorCode::Cho1, false),
        ("[key=C,D(9,9)]E", ErrorCode::Cimv3, false),
        ("C(9,(11,9),9)", ErrorCode::Ext2, true),
    ][usize::from(bytes[0]) % 4];
    let (boundary, resets_section) = [
        ("-", false),
        ("\n", false),
        ("\r\n", false),
        ("\n\n", true),
        ("\n@section=Next\n", true),
    ][usize::from(bytes[bytes.len() - 1]) % 5];
    let prefix = source.trim_end_matches(['\r', '\n']);
    let damaged = format!("{prefix}-{invalid}{boundary}%-D(11,11)");
    let report = parse_chord_progression_string_with_warnings(&damaged);
    let errors = report.result.expect_err("the inserted suffix must fail");
    let mut expected_errors = vec![first_error];
    if resets_section {
        expected_errors.push(ErrorCode::Chb1);
    }
    assert_eq!(
        errors
            .iter()
            .map(|error| error.error.code)
            .collect::<Vec<_>>(),
        expected_errors,
        "recovery changed diagnostics for {damaged:?}"
    );
    assert_eq!(
        report.warnings.len(),
        warnings.len() + usize::from(extra_warning) + 1,
        "recovery changed warning count for {damaged:?}"
    );
    assert_eq!(
        &report.warnings[..warnings.len()],
        warnings,
        "recovery changed preceding warnings for {damaged:?}"
    );
    if extra_warning {
        let warning = &report.warnings[warnings.len()];
        assert_eq!(warning.code, WarningCode::DuplicateExtension);
        assert_eq!(warning.additional_info.as_deref(), Some("9"));
        assert_eq!(
            warning.position.start_offset,
            prefix.encode_utf16().count() + 1 + invalid.rfind('9').unwrap()
        );
    }
    let last = report.warnings.last().unwrap();
    assert_eq!(last.code, WarningCode::DuplicateExtension);
    assert_eq!(last.additional_info.as_deref(), Some("11"));
    assert_eq!(
        last.position.start_offset,
        damaged.encode_utf16().count() - 3
    );
    if resets_section {
        assert_eq!(
            errors.last().unwrap().position.start_offset,
            damaged.encode_utf16().count() - "%-D(11,11)".len()
        );
    }
    assert_eq!(
        crate::support::assert_parse_invariants(&damaged),
        (false, report.warnings.len())
    );
}

/** Compares grammar-generated source with its independently built AST and warning count. */
pub fn assert_generated_document(bytes: &[u8]) {
    if bytes.is_empty() {
        return;
    }
    let (source, expected) = document(bytes);
    let mut expected_warnings = 0;
    for section in &expected {
        for block in &section.chord_blocks {
            if let ChordBlock::Bar(bar) = block {
                for info in bar {
                    if let ChordExpression::Chord(chord) = &info.chord_expression {
                        for (index, extension) in chord.detailed.extensions.iter().enumerate() {
                            expected_warnings +=
                                usize::from(chord.detailed.extensions[..index].contains(extension));
                        }
                    }
                }
            }
        }
    }
    let report = parse_chord_progression_string_with_warnings(&source);
    assert_recovery(&source, &report.warnings, bytes);
    assert_eq!(
        report.result,
        Ok(expected),
        "generated source was misparsed: {source:?}"
    );
    assert_eq!(
        report.warnings.len(),
        expected_warnings,
        "incorrect warnings for {source:?}"
    );
    assert_eq!(
        crate::support::assert_parse_invariants(&source),
        (true, expected_warnings)
    );
}
