use std::{error::Error, fmt, fmt::Write};

use crate::model::{
    ast::Ast, bar::Bar, chord_block::ChordBlock, chord_expression::ChordExpression,
    chord_info::ChordInfo, chord_info_meta::ChordInfoMeta, section::Section,
    section_meta::SectionMeta,
};

/** Reports that an AST cannot be represented without changing its structure. */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatError;

impl fmt::Display for FormatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("AST cannot be represented by the chord progression syntax")
    }
}

impl Error for FormatError {}

/** Formats an AST and rejects structures that cannot round-trip through the parser. */
pub fn format_chord_progression(ast: &Ast) -> Result<String, FormatError> {
    let source = format_ast(ast);
    let Ok(reparsed) = crate::parse_chord_progression_string(&source) else {
        return Err(FormatError);
    };

    if &reparsed != ast {
        return Err(FormatError);
    }

    Ok(source)
}

/** Renders an AST before the public round-trip validation. */
fn format_ast(ast: &Ast) -> String {
    let mut source = String::new();
    for (index, section) in ast.iter().enumerate() {
        if index > 0 {
            source.push_str("\n\n");
        }
        write_section(&mut source, section);
    }
    source
}

/** Appends a section directly to the document, placing metadata before its chord lines. */
fn write_section(source: &mut String, section: &Section) {
    for (index, meta) in section.meta_infos.iter().enumerate() {
        if index > 0 {
            source.push('\n');
        }
        write_section_meta(source, meta);
    }
    if !section.meta_infos.is_empty() && !section.chord_blocks.is_empty() {
        source.push('\n');
    }
    write_chord_blocks(source, &section.chord_blocks);
}

/** Writes section metadata without allocating an intermediate string. */
fn write_section_meta(source: &mut String, meta: &SectionMeta) {
    match meta {
        SectionMeta::Section(value) => write!(source, "@section={value}"),
        SectionMeta::Repeat(value) => write!(source, "@repeat={value}"),
    }
    .expect("writing into a String cannot fail");
}

/** Appends bars while preserving explicit line-break blocks. */
fn write_chord_blocks(source: &mut String, chord_blocks: &[ChordBlock]) {
    let mut needs_bar_separator = false;

    for chord_block in chord_blocks {
        match chord_block {
            ChordBlock::Bar(bar) => {
                if needs_bar_separator {
                    source.push_str(" - ");
                }
                write_bar(source, bar);
                needs_bar_separator = true;
            }
            ChordBlock::Br => {
                source.push('\n');
                needs_bar_separator = false;
            }
        }
    }
}

/** Appends comma-separated chord information without allocating one string per chord. */
fn write_bar(source: &mut String, bar: &Bar) {
    for (index, chord_info) in bar.iter().enumerate() {
        if index > 0 {
            source.push_str(", ");
        }
        write_chord_info(source, chord_info);
    }
}

/** Appends chord metadata, expression, and optional denominator. */
fn write_chord_info(source: &mut String, chord_info: &ChordInfo) {
    for meta in &chord_info.meta_infos {
        write_chord_info_meta(source, meta);
    }
    source.push_str(format_chord_expression(&chord_info.chord_expression));

    if let Some(denominator) = &chord_info.denominator {
        source.push('/');
        source.push_str(denominator);
    }
}

/** Writes metadata attached to the following chord expression. */
fn write_chord_info_meta(source: &mut String, meta: &ChordInfoMeta) {
    match meta {
        ChordInfoMeta::Key(key) => write!(source, "[key={key}]"),
    }
    .expect("writing into a String cannot fail");
}

/** Formats a chord expression without discarding its original chord spelling. */
fn format_chord_expression(expression: &ChordExpression) -> &str {
    match expression {
        ChordExpression::Chord(chord) => &chord.plain,
        ChordExpression::UnIdentified => "?",
        ChordExpression::NoChord => "_",
        ChordExpression::Same => "%",
    }
}

#[cfg(test)]
mod tests {
    use super::{format_chord_progression, FormatError};
    use crate::model::{
        base::Base, chord_block::ChordBlock, chord_expression::ChordExpression, section::Section,
        section_meta::SectionMeta,
    };
    use crate::parse_chord_progression_string;

    /** Advances a deterministic pseudo-random state for reproducible source generation. */
    fn next_random(state: &mut u64) -> u64 {
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;
        *state
    }

    /** Keeps formatting stable and preserves the parsed AST through a round trip. */
    #[test]
    fn formats_a_representative_ast_canonically() {
        let input = "@section=Verse\n@repeat=2\n[key=C]C(M9),G/B-Dm(7)\n?-_-%";
        let ast = parse_chord_progression_string(input).expect("fixture must parse");

        let formatted = format_chord_progression(&ast).expect("parsed AST must be formattable");

        assert_eq!(
            formatted,
            "@section=Verse\n@repeat=2\n[key=C]C(M9), G/B - Dm(7)\n? - _ - %"
        );
        assert_eq!(
            parse_chord_progression_string(&formatted).expect("formatted source must parse"),
            ast
        );
    }

    /** Formats the empty AST as an empty document. */
    #[test]
    fn formats_an_empty_ast() {
        assert_eq!(
            format_chord_progression(&Vec::new()).expect("empty AST must be formattable"),
            ""
        );
        assert_eq!(
            FormatError.to_string(),
            "AST cannot be represented by the chord progression syntax"
        );
    }

    /** Preserves enharmonic key spellings as distinct parser variants. */
    #[test]
    fn preserves_enharmonic_key_spellings() {
        let input = "[key=Fb]C-[key=E#]F\n[key=Fbm]C-[key=E#m]F";
        let ast = parse_chord_progression_string(input).expect("fixture must parse");

        let formatted = format_chord_progression(&ast).expect("fixture must be formattable");

        assert_eq!(formatted, "[key=Fb]C - [key=E#]F\n[key=Fbm]C - [key=E#m]F");
        assert_eq!(
            parse_chord_progression_string(&formatted).expect("formatted source must parse"),
            ast
        );
    }

    /** Rejects typed AST shapes that would produce invalid source or lose information. */
    #[test]
    fn rejects_unrepresentable_asts() {
        let mut empty_denominator =
            parse_chord_progression_string("C").expect("fixture must parse");
        let ChordBlock::Bar(bar) = &mut empty_denominator[0].chord_blocks[0] else {
            panic!("fixture must contain a bar");
        };
        bar[0].denominator = Some(String::new());

        let empty_section = vec![Section {
            meta_infos: Vec::new(),
            chord_blocks: Vec::new(),
        }];

        assert_eq!(
            format_chord_progression(&empty_denominator),
            Err(FormatError)
        );
        assert_eq!(format_chord_progression(&empty_section), Err(FormatError));
    }

    /** Rejects AST mutations that would be lost or reinterpreted as syntax. */
    #[test]
    fn rejects_structurally_ambiguous_asts() {
        let mut empty_bar = parse_chord_progression_string("C").expect("fixture must parse");
        let ChordBlock::Bar(bar) = &mut empty_bar[0].chord_blocks[0] else {
            panic!("fixture must contain a bar");
        };
        bar.clear();

        let mut leading_break = parse_chord_progression_string("C").expect("fixture must parse");
        leading_break[0].chord_blocks.insert(0, ChordBlock::Br);

        let mut injected_metadata =
            parse_chord_progression_string("@section=A\nC").expect("fixture must parse");
        injected_metadata[0].meta_infos[0] = SectionMeta::Section("A\n@section=B".to_string());

        let mut inconsistent_chord =
            parse_chord_progression_string("C").expect("fixture must parse");
        let ChordBlock::Bar(bar) = &mut inconsistent_chord[0].chord_blocks[0] else {
            panic!("fixture must contain a bar");
        };
        let ChordExpression::Chord(chord) = &mut bar[0].chord_expression else {
            panic!("fixture must contain a chord");
        };
        chord.detailed.base = Base::D;

        for ast in [
            empty_bar,
            leading_break,
            injected_metadata,
            inconsistent_chord,
        ] {
            assert_eq!(format_chord_progression(&ast), Err(FormatError));
        }
    }

    /** Round-trips every valid document found in an adversarial deterministic corpus. */
    #[test]
    fn round_trips_parser_produced_asts_from_adversarial_sources() {
        let alphabet = [
            'A', 'C', 'D', 'm', '9', '#', 'b', '@', '[', ']', '(', ')', '=', ',', '/', '-', '?',
            '%', '_', ' ', '\t', '\n', '\r',
        ];
        let mut state = 0xa076_1d64_78bd_642f;
        let mut valid_document_count = 0;
        let mut non_empty_ast_count = 0;

        for _ in 0..20_000 {
            let length = (next_random(&mut state) % 48) as usize;
            let mut input = String::new();
            for _ in 0..length {
                let index = (next_random(&mut state) % alphabet.len() as u64) as usize;
                input.push(alphabet[index]);
            }

            let Ok(ast) = parse_chord_progression_string(&input) else {
                continue;
            };
            valid_document_count += 1;
            if !ast.is_empty() {
                non_empty_ast_count += 1;
            }

            let formatted = format_chord_progression(&ast)
                .unwrap_or_else(|error| panic!("formatter rejected {input:?}: {error}"));
            let reparsed = parse_chord_progression_string(&formatted).unwrap_or_else(|error| {
                panic!("formatter produced invalid source {formatted:?} from {input:?}: {error:?}")
            });
            assert_eq!(reparsed, ast, "round trip changed AST for {input:?}");
        }

        assert!(
            valid_document_count >= 100,
            "adversarial corpus did not exercise enough valid documents"
        );
        assert!(
            non_empty_ast_count >= 40,
            "adversarial corpus only found {non_empty_ast_count} non-empty ASTs"
        );
    }
}
