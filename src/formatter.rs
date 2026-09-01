use std::{error::Error, fmt};

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
    let Ok(reparsed) = crate::parser::parse(&source) else {
        return Err(FormatError);
    };

    if &reparsed != ast {
        return Err(FormatError);
    }

    Ok(source)
}

/** Renders an AST before the public round-trip validation. */
fn format_ast(ast: &Ast) -> String {
    ast.iter()
        .map(format_section)
        .collect::<Vec<_>>()
        .join("\n\n")
}

/** Formats one section, placing metadata before its chord lines. */
fn format_section(section: &Section) -> String {
    let mut source = section
        .meta_infos
        .iter()
        .map(format_section_meta)
        .collect::<Vec<_>>()
        .join("\n");
    let chord_blocks = format_chord_blocks(&section.chord_blocks);

    if !source.is_empty() && !chord_blocks.is_empty() {
        source.push('\n');
    }
    source.push_str(&chord_blocks);

    source
}

/** Formats section metadata using its concrete syntax. */
fn format_section_meta(meta: &SectionMeta) -> String {
    match meta {
        SectionMeta::Section(value) => format!("@section={value}"),
        SectionMeta::Repeat(value) => format!("@repeat={value}"),
    }
}

/** Formats bars and preserves explicit line-break blocks. */
fn format_chord_blocks(chord_blocks: &[ChordBlock]) -> String {
    let mut source = String::new();
    let mut needs_bar_separator = false;

    for chord_block in chord_blocks {
        match chord_block {
            ChordBlock::Bar(bar) => {
                if needs_bar_separator {
                    source.push_str(" - ");
                }
                source.push_str(&format_bar(bar));
                needs_bar_separator = true;
            }
            ChordBlock::Br => {
                source.push('\n');
                needs_bar_separator = false;
            }
        }
    }

    source
}

/** Formats the chord information grouped within one bar. */
fn format_bar(bar: &Bar) -> String {
    bar.iter()
        .map(format_chord_info)
        .collect::<Vec<_>>()
        .join(", ")
}

/** Formats chord metadata, expression, and optional denominator. */
fn format_chord_info(chord_info: &ChordInfo) -> String {
    let mut source = chord_info
        .meta_infos
        .iter()
        .map(format_chord_info_meta)
        .collect::<String>();
    source.push_str(format_chord_expression(&chord_info.chord_expression));

    if let Some(denominator) = &chord_info.denominator {
        source.push('/');
        source.push_str(denominator);
    }

    source
}

/** Formats metadata attached to the following chord expression. */
fn format_chord_info_meta(meta: &ChordInfoMeta) -> String {
    match meta {
        ChordInfoMeta::Key(key) => format!("[key={key}]"),
    }
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
    use crate::model::{chord_block::ChordBlock, section::Section};
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
