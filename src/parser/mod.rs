mod types;

use std::str::FromStr;

use crate::errors;
use crate::tokenizer::types::token::Token;

use types::ast::Ast;
use types::chord::Chord;
use types::chord_detailed::ChordDetailed;
use types::chord_info::ChordInfo;
use types::chord_expression::ChordExpression;
use types::chord_info_meta::ChordInfoMeta;
use types::section::Section;
use types::section_meta::SectionMeta;

use self::types::extension::Extension;

pub fn parse(tokens: &[Token]) -> Result<Ast, String> {
    let mut sections: Vec<Section> = vec![Section {
        meta_infos: Vec::new(),
        chord_blocks: Vec::new(),
    }];
    let mut tokens = tokens.iter().peekable();
    let mut tmp_chord_info_meta_infos: Vec<ChordInfoMeta> = Vec::new();

    while let Some(token) = tokens.next() {
        match token {
            // section meta info
            Token::SectionMetaInfoStart => {
                let is_new_section = 
                    // last section's chord_blocks is not empty
                    !sections.last().unwrap().chord_blocks.is_empty();

                // if is_new_section, initialize new section
                if is_new_section {
                    sections.push(Section {
                        meta_infos: Vec::new(),
                        chord_blocks: Vec::new(),
                    });
                }

                // if next token is not Token::SectionMetaInfoKey, return error
                let section_meta_info_key = match tokens.next() {
                    Some(Token::SectionMetaInfoKey(value)) => value,
                    _ => {
                        return Err(errors::SECTION_META_INFO_KEY_SHOULD_NOT_CONTAINS_LINE_BREAK
                            .to_string())
                    }
                };

                // if next token is not Token::Equal, return error
                match tokens.next() {
                    Some(Token::Equal) => {}
                    _ => {
                        return Err(errors::SECTION_META_INFO_KEY_SHOULD_NOT_CONTAINS_LINE_BREAK
                            .to_string())
                    }
                }

                // if next token is not Token::SectionMetaInfoValue, return error
                let section_meta_info_value = match tokens.next() {
                    Some(Token::SectionMetaInfoValue(value)) => value,
                    _ => {
                        return Err(errors::SECTION_META_INFO_VALUE_SHOULD_NOT_BE_EMPTY.to_string())
                    }
                };

                // add section meta info to last section
                match section_meta_info_key.as_str() {
                    "section" => {
                        sections
                            .last_mut()
                            .unwrap()
                            .meta_infos
                            .push(SectionMeta::Section {
                                value: section_meta_info_value.clone(),
                            });
                    }
                    "repeat" => {
                        // if section_meta_info_value cannot parse as u32, return error
                        if section_meta_info_value.parse::<u32>().is_err() {
                            return Err(
                                errors::SECTION_META_INFO_VALUE_OF_REPEAT_NEEDS_TO_BE_NUMBER
                                    .to_string(),
                            );
                        }

                        sections
                            .last_mut()
                            .unwrap()
                            .meta_infos
                            .push(SectionMeta::Repeat {
                                value: section_meta_info_value.parse::<u32>().unwrap(),
                            });
                    }
                    _ => {
                        return Err([
                            errors::SECTION_META_INFO_KEY_IS_INVALID.to_string(),
                            section_meta_info_key.to_string(),
                        ].join(": "));
                    }
                }

                match tokens.peek() {
                    Some(Token::LineBreak) => {}
                    _ => {
                        return Err(
                            errors::SECTION_META_INFO_VALUE_NEEDS_LINE_BREAK_AFTER.to_string()
                        );
                    }
                }
            }
            // meta info
            Token::MetaInfoStart => {
                //(

                // if next token is not Token::MetaInfoKey, return error
                let meta_info_key = match tokens.next() {
                    Some(Token::MetaInfoKey(value)) => value,
                    _ => return Err(errors::META_INFO_KEY_SHOULD_NOT_BE_EMPTY.to_string()),
                };

                // if next token is not Token::Equal, return error
                match tokens.next() {
                    Some(Token::Equal) => {}
                    _ => {
                        return Err(errors::META_INFO_KEY_SHOULD_NOT_CONTAINS_LINE_BREAK.to_string())
                    }
                }

                // if next token is not Token::MetaInfoValue, return error
                let meta_info_value = match tokens.next() {
                    Some(Token::MetaInfoValue(value)) => value,
                    _ => return Err(errors::META_INFO_VALUE_SHOULD_NOT_BE_EMPTY.to_string()),
                };

                // add meta info to last chord block
                match meta_info_key.as_str() {
                    "key" => {
                        let key_name = match meta_info_value.parse() {
                            Ok(key) => key,
                            Err(_) => return Err(errors::META_INFO_VALUE_IS_INVALID.to_string()),
                        };

                        // add ChordInfoMeta to temporary variable
                        tmp_chord_info_meta_infos.push(ChordInfoMeta::Key { value: key_name });
                    }
                    _ => {
                        return Err(errors::META_INFO_KEY_IS_INVALID.to_string());
                    }
                }

                // if next token is not Token::MetaInfoEnd, return error
                match tokens.next() {
                    Some(Token::MetaInfoEnd) => {}
                    _ => {
                        return Err(
                            errors::META_INFO_VALUE_NEEDS_CLOSE_PARENTHESIS_AFTER.to_string()
                        )
                    }
                }
            }
            // chord
            Token::Chord(chord_string) => {

                if chord_string.eq("-") || chord_string.eq("?") || chord_string.eq("%") {
                    // if chord_blocks is empty, make new chord_block
                    if sections.last_mut().unwrap().chord_blocks.is_empty() {
                        if chord_string == "%" {
                            return Err(
                                errors::SAME_CHORD_SYMBOL_SHOULD_NOT_BE_PLACED_FIRST_OF_CHORD_BLOCK.to_string()
                            );
                        }
                        sections.last_mut().unwrap().chord_blocks.push(Vec::new());
                    }

                    // add ChordInfo to last chord block
                    sections
                        .last_mut()
                        .unwrap()
                        .chord_blocks
                        .last_mut()
                        .unwrap()
                        .push(ChordInfo {
                            chord: match chord_string.as_str() {
                                "?" => ChordExpression::Unidentified,
                                "%" => ChordExpression::Same,
                                _ => {
                                    return Err(errors::CHORD_IS_INVALID.to_string());
                                }
                            },
                            denominator: None,
                            meta_infos: tmp_chord_info_meta_infos.clone(),
                        });

                    // reset tmp_chord_info_meta_infos
                    tmp_chord_info_meta_infos = Vec::new();

                    continue;
                }

                let chord_detailed_result = ChordDetailed::from_str(chord_string);
                if let Ok(detailed) = chord_detailed_result {
                    let chord = Chord {
                        plain: chord_string.clone(),
                        detailed,
                    };

                    // if chord_blocks is empty, make new chord_block
                    if sections.last_mut().unwrap().chord_blocks.is_empty() {
                        sections.last_mut().unwrap().chord_blocks.push(Vec::new());
                    }

                    // make chord info and add to last chord block
                    sections
                        .last_mut()
                        .unwrap()
                        .chord_blocks
                        .last_mut()
                        .unwrap()
                        .push(ChordInfo {
                            chord: ChordExpression::Chord(chord),
                            denominator: None,
                            meta_infos: tmp_chord_info_meta_infos.clone(),
                        });

                    // reset tmp_chord_info_meta_infos
                    tmp_chord_info_meta_infos = Vec::new();
                } else {
                    return Err([
                        errors::CHORD_IS_INVALID.to_string(),
                        chord_detailed_result.err().unwrap(),
                        chord_string.to_string(),
                    ]
                    .join(": "));
                }
            }
            Token::LineBreak => {
                // if Token::LineBreak appears two or more times in a row, create new section
                match tokens.peek() {
                    Some(Token::LineBreak) => {
                        // if last section is empty, remove it
                        if sections.last().unwrap().chord_blocks.is_empty() {
                            sections.pop();
                        }

                        // create new section
                        sections.push(Section {
                            meta_infos: Vec::new(),
                            chord_blocks: Vec::new(),
                        });
                    }
                    _ => { /* Nothing */ }
                }
            }
            Token::Extension(ext_str) => { 
                match &sections
                .last_mut()
                .unwrap()
                .chord_blocks
                .last_mut()
                .unwrap()
                .last_mut()
                .unwrap()
                .chord {
                    ChordExpression::Unidentified => {},
                    ChordExpression::Same => {},
                    ChordExpression::NoChord => {},
                    ChordExpression::Chord(c) => {

                        let mut parsed_extensions = vec![
                            Extension::from_str(ext_str).unwrap()
                        ];
                        for t in tokens.by_ref() {
                            match t {
                                Token::ExtensionEnd => {
                                    break;
                                }
                                Token::Comma => {
                                    continue;
                                }
                                Token::Extension(ext_str) => {
                                    parsed_extensions.push(Extension::from_str(ext_str).unwrap());
                                }
                                _ => {
                                    return Err(errors::INVALID_EXTENSION.to_string());
                                }
                            }
                        }
                        let extension_str_with_parenthesis = format!("({})", 
                            parsed_extensions.iter().map(|e| e.to_string()).collect::<Vec<String>>().join(",")
                        );
                        
                        sections
                            .last_mut()
                            .unwrap()
                            .chord_blocks
                            .last_mut()
                            .unwrap()
                            .last_mut()
                            .unwrap()
                            .chord = ChordExpression::Chord(Chord {
                                    plain: [c.plain.clone(), extension_str_with_parenthesis.to_string()].concat(),
                                    detailed: ChordDetailed { 
                                        base: c.detailed.base.clone(),
                                        accidental: c.detailed.accidental.clone(),
                                        chord_type: c.detailed.chord_type.clone(),
                                        extensions: parsed_extensions, 
                                    },
                                });
                    }
                }
            }
            Token::Denominator(denominator) => {

                if sections
                    .last_mut()
                    .unwrap()
                    .chord_blocks
                    .last().is_none() {
                        return Err(errors::CHORD_SHOULD_NOT_BE_EMPTY.to_string());
                    }

                // if denominator is already set, DENOMINATOR_IS_LIMITED_TO_ONE_PER_CHORD
                if sections
                    .last_mut()
                    .unwrap()
                    .chord_blocks
                    .last_mut()
                    .unwrap()
                    .last_mut()
                    .unwrap()
                    .denominator.is_some() {
                        return Err(errors::DENOMINATOR_IS_LIMITED_TO_ONE_PER_CHORD.to_string());
                    }

                sections
                    .last_mut()
                    .unwrap()
                    .chord_blocks
                    .last_mut()
                    .unwrap()
                    .last_mut()
                    .unwrap()
                    .denominator = Some(denominator.clone());
            },
            Token::Comma => { /* Nothing */ }
            Token::ChordBlockSeparator => { 
                match tokens.peek() {                
                    Some(Token::ChordBlockSeparator) => {

                        // if chord_blocks is empty, make new chord_block
                        if sections.last_mut().unwrap().chord_blocks.is_empty() {
                            sections.last_mut().unwrap().chord_blocks.push(Vec::new());
                        }

                        // add ChordInfo to last chord block
                        sections
                            .last_mut()
                            .unwrap()
                            .chord_blocks
                            .last_mut()
                            .unwrap()
                            .push(ChordInfo {
                                chord: ChordExpression::NoChord,
                                denominator: None,
                                meta_infos: tmp_chord_info_meta_infos.clone(),
                            });                        
                    },
                    _ => { /* Nothing */ }
                }
            }
            Token::Equal => { /* Nothing */ }
            Token::Slash => { /* Nothing */ }
            Token::ExtensionStart => { 

                // if next token is not Extension, error
                match tokens.peek() {
                    Some(Token::Extension(_)) => { /* Nothing */ }
                    _ => {
                        return Err(errors::EXTENSION_MUST_NOT_BE_EMPTY.to_string());
                    }
                }
            }
            Token::ExtensionEnd => { /* Nothing */}
            _ => {
                // invalid token
                return Err([
                    errors::INVALID_TOKEN_TYPE.to_string(),
                    token.to_string(),
                ].join(": "));
            }
        }
    }

    Ok(sections)
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::accidental::Accidental;
    use types::base::Base;
    use types::chord_info::ChordInfo;
    use types::chord_type::ChordType;
    use types::extension::Extension;

    #[cfg(test)]
    mod success {
        use super::*;


        #[test]
        fn no_chord() {
            let input = [
                Token::ChordBlockSeparator,
                Token::ChordBlockSeparator,
            ];
            let result = parse(&input);

            println!("1111 {:?}", result);
            assert_eq!(result.unwrap(), [
                Section {
                    meta_infos: Vec::new(),
                    chord_blocks: vec![vec![ChordInfo {
                        chord: ChordExpression::NoChord,
                        denominator: None,
                        meta_infos: Vec::new(),
                    },],],
                }
            ]);
        }

        #[test]
        fn section_meta_info() {
            let input = [
                Token::SectionMetaInfoStart,
                Token::SectionMetaInfoKey("section".to_string()),
                Token::Equal,
                Token::SectionMetaInfoValue("A".to_string()),
                Token::LineBreak,
                Token::SectionMetaInfoStart,
                Token::SectionMetaInfoKey("repeat".to_string()),
                Token::Equal,
                Token::SectionMetaInfoValue("3".to_string()),
                Token::LineBreak,
            ];

            assert_eq!(
                parse(&input),
                Ok([Section {
                    meta_infos: vec![
                        SectionMeta::Section {
                            value: "A".to_string(),
                        },
                        SectionMeta::Repeat {
                            value: 3,
                        },
                    ],
                    chord_blocks: Vec::new(),
                }]
                .to_vec())
            );
        }

        #[test]
        fn multiple_section_meta_info() {
            let input = [
                Token::LineBreak,
                Token::SectionMetaInfoStart,
                Token::SectionMetaInfoKey("section".to_string()),
                Token::Equal,
                Token::SectionMetaInfoValue("A".to_string()),
                Token::LineBreak,
                Token::SectionMetaInfoStart,
                Token::SectionMetaInfoKey("section".to_string()),
                Token::Equal,
                Token::SectionMetaInfoValue("AA".to_string()),
                Token::LineBreak,
            ];

            assert_eq!(
                parse(&input),
                Ok([Section {
                    meta_infos: vec![
                        SectionMeta::Section {
                            value: "A".to_string(),
                        },
                        SectionMeta::Section {
                            value: "AA".to_string(),
                        }
                    ],
                    chord_blocks: Vec::new(),
                }]
                .to_vec())
            );
        }

        #[test]
        fn chord_blocks_with_fraction_chord() {
            let input = [
                Token::LineBreak,
                Token::ChordBlockSeparator,
                Token::Chord("C".to_string()),
                Token::ChordBlockSeparator,
                Token::Chord("G".to_string()),
                Token::Slash,
                Token::Denominator("Bb".to_string()),
                Token::ChordBlockSeparator,
                Token::Chord("Am".to_string()),
                Token::ChordBlockSeparator,
                Token::Chord("Em".to_string()),
                Token::Slash,
                Token::Denominator("G".to_string()),
                Token::ChordBlockSeparator,
                Token::LineBreak,
                Token::ChordBlockSeparator,
                Token::Chord("F#m".to_string()),
                Token::ExtensionStart,
                Token::Extension("7".to_string()),
                Token::Comma,
                Token::Extension("b5".to_string()),
                Token::ExtensionEnd,
                Token::Slash,
                Token::Denominator("F#m(7,b5)".to_string()),
                Token::ChordBlockSeparator,
                Token::Chord("Fbm".to_string()),
                Token::ExtensionStart,
                Token::Extension("13".to_string()),
                Token::ExtensionEnd,
                Token::Slash,
                Token::Denominator("G7".to_string()),
                Token::ChordBlockSeparator,
                Token::LineBreak,
            ];

            let expected = [Section {
                meta_infos: Vec::new(),
                chord_blocks: vec![vec![
                    ChordInfo {
                        meta_infos: Vec::new(),
                        denominator: None,
                        chord: ChordExpression::Chord(Chord {
                            plain: "C".to_string(),
                            detailed: ChordDetailed {
                                base: Base::C,
                                accidental: None,
                                chord_type: ChordType::Major,
                                extensions: Vec::new(),
                            },
                        }),
                    },
                    ChordInfo {
                        chord: ChordExpression::Chord(Chord {
                            plain: "G".to_string(),
                            detailed: ChordDetailed {
                                base: Base::G,
                                accidental: None,
                                chord_type: ChordType::Major,
                                extensions: Vec::new(),
                            },
                        }),
                        denominator: Some("Bb".to_string()),
                        meta_infos: Vec::new(),
                    },
                    ChordInfo {
                        chord: ChordExpression::Chord(Chord {
                            plain: "Am".to_string(),
                            detailed: ChordDetailed {
                                base: Base::A,
                                accidental: None,
                                chord_type: ChordType::Minor,
                                extensions: Vec::new(),
                            },
                        }),
                        denominator: None,
                        meta_infos: Vec::new(),
                    },
                    ChordInfo {
                        chord: ChordExpression::Chord(Chord {
                            plain: "Em".to_string(),
                            detailed: ChordDetailed {
                                base: Base::E,
                                accidental: None,
                                chord_type: ChordType::Minor,
                                extensions: Vec::new(),
                            },
                        }),
                        denominator: Some("G".to_string()),
                        meta_infos: Vec::new(),
                    },
                    ChordInfo {
                        chord: ChordExpression::Chord(Chord {
                            plain: "F#m(7,b5)".to_string(),
                            detailed: ChordDetailed {
                                base: Base::F,
                                accidental: Some(Accidental::Sharp),
                                chord_type: ChordType::Minor,
                                extensions: vec![
                                    Extension::Seven,
                                    Extension::FlatFive,
                                ],
                            },
                        }),
                        denominator: Some("F#m(7,b5)".to_string()),
                        meta_infos: Vec::new(),
                    },
                    ChordInfo {
                        chord: ChordExpression::Chord(Chord {
                            plain: "Fbm(13)".to_string(),
                            detailed: ChordDetailed {
                                base: Base::F,
                                accidental: Some(Accidental::Flat),
                                chord_type: ChordType::Minor,
                                extensions: vec![Extension::Thirteen],
                            },
                        }),
                        denominator: Some("G7".to_string()),
                        meta_infos: Vec::new(),
                    },
                ]],
            }];
            let parsed_result = parse(&input);
            assert_eq!(parsed_result, Ok(expected.to_vec()));
        }

        #[test]
        // SAME_CHORD_SYMBOL_SHOULD_NOT_BE_PLACED_FIRST_OF_CHORD_BLOCK
        fn same_chord_symbol_should_not_be_placed_first_of_chord_block() {
            let input = [
                Token::ChordBlockSeparator,
                Token::Chord("%".to_string()),
                Token::ChordBlockSeparator,
            ];

            assert_eq!(
                parse(&input),
                Err(errors::SAME_CHORD_SYMBOL_SHOULD_NOT_BE_PLACED_FIRST_OF_CHORD_BLOCK.to_string())
            );
        }

        #[test]
        fn chord_blocks_with_expressions() {
            let input = [
                Token::ChordBlockSeparator,
                Token::Chord("?".to_string()),
                Token::ChordBlockSeparator,
                Token::Chord("%".to_string()),
                Token::ChordBlockSeparator,
            ];

            assert_eq!(
                parse(&input),
                Ok([
                    Section {
                        meta_infos: Vec::new(),
                        chord_blocks: vec![vec![
                            ChordInfo {
                                chord: ChordExpression::Unidentified,
                                denominator: None,
                                meta_infos: Vec::new(),
                            },
                            ChordInfo {
                                chord: ChordExpression::Same,
                                denominator: None,
                                meta_infos: Vec::new(),
                            },
                        ],],
                    },
                ]
                .to_vec())
            );
        }

        #[test]
        fn multiple_section_without_section_meta() {
            let input = [
                Token::ChordBlockSeparator,
                Token::Chord("C".to_string()),
                Token::ChordBlockSeparator,
                Token::LineBreak,
                Token::LineBreak,
                Token::ChordBlockSeparator,
                Token::Chord("C".to_string()),
                Token::ChordBlockSeparator,
            ];
            assert_eq!(
                parse(&input),
                Ok([
                    Section {
                        meta_infos: Vec::new(),
                        chord_blocks: vec![vec![ChordInfo {
                            chord: ChordExpression::Chord(Chord {
                                plain: "C".to_string(),
                                detailed: ChordDetailed {
                                    base: Base::C,
                                    accidental: None,
                                    chord_type: ChordType::Major,
                                    extensions: Vec::new(),
                                },
                            },),
                            denominator: None,
                            meta_infos: Vec::new(),
                        },],],
                    },
                    Section {
                        meta_infos: Vec::new(),
                        chord_blocks: vec![vec![ChordInfo {
                            chord: ChordExpression::Chord(Chord {
                                plain: "C".to_string(),
                                detailed: ChordDetailed {
                                    base: Base::C,
                                    accidental: None,
                                    chord_type: ChordType::Major,
                                    extensions: Vec::new(),
                                },
                            },),
                            denominator: None,
                            meta_infos: Vec::new(),
                        },],],
                    }
                ]
                .to_vec())
            );
        }
    }

    #[cfg(test)]
    mod failure {
        use crate::{tokenizer::types::token::Token, errors, parser::{parse, types::{chord_info::ChordInfo, chord_expression::ChordExpression, chord::Chord, base::Base, extension::Extension, chord_type::ChordType, chord_detailed::ChordDetailed, section::Section}}};

        #[test]
        fn section_meta_info_value_of_repeat_needs_to_be_number() {
            let input = [
                Token::SectionMetaInfoStart,
                Token::SectionMetaInfoKey("repeat".to_string()),
                Token::Equal,
                Token::SectionMetaInfoValue("A".to_string()),
                Token::LineBreak,
            ];

            assert_eq!(
                parse(&input),
                Err(errors::SECTION_META_INFO_VALUE_OF_REPEAT_NEEDS_TO_BE_NUMBER.to_string())
            );
        }

        #[test]
        fn section_meta_info_key_is_invalid() {
            let input = [
                Token::SectionMetaInfoStart,
                Token::SectionMetaInfoKey("asdf".to_string()),
                Token::Equal,
                Token::SectionMetaInfoValue("A".to_string()),
                Token::LineBreak,
            ];

            assert_eq!(
                parse(&input),
                Err([errors::SECTION_META_INFO_KEY_IS_INVALID.to_string(), ": asdf".to_string()].concat())
            );
        }

        #[test]
        fn section_meta_info_value_needs_line_break_after() {
            let input = [
                Token::SectionMetaInfoStart,
                Token::SectionMetaInfoKey("section".to_string()),
                Token::Equal,
                Token::SectionMetaInfoValue("A".to_string()),
                Token::ChordBlockSeparator,
                Token::LineBreak,
            ];

            assert_eq!(
                parse(&input),
                Err(errors::SECTION_META_INFO_VALUE_NEEDS_LINE_BREAK_AFTER.to_string())
            );
        }

        #[test]
        fn chord_should_not_be_empty() {
            let input = [
                Token::ChordBlockSeparator,
                Token::Slash,
                Token::Denominator("D".to_string()),
                Token::ChordBlockSeparator,
            ];

            assert_eq!(parse(&input), Err(errors::CHORD_SHOULD_NOT_BE_EMPTY.to_string()));
        }

        #[test]
        fn denominator_is_limited_to_one_per_chord() {
            let input = [
                Token::ChordBlockSeparator,
                Token::Chord("C".to_string()),
                Token::Slash,
                Token::Denominator("D".to_string()),
                Token::Slash,
                Token::Denominator("E".to_string()),
                Token::ChordBlockSeparator,
            ];

            assert_eq!(
                parse(&input),
                Err(errors::DENOMINATOR_IS_LIMITED_TO_ONE_PER_CHORD.to_string())
            );
        }

        #[test]
        fn extension_must_not_be_empty() {
            let input = [
                Token::ChordBlockSeparator,
                Token::Chord("C".to_string()),
                Token::ExtensionStart,
                Token::ExtensionEnd,
                Token::ChordBlockSeparator,
            ];

            assert_eq!(
                parse(&input),
                Err(errors::EXTENSION_MUST_NOT_BE_EMPTY.to_string())
            );
        }

        
    }
}
