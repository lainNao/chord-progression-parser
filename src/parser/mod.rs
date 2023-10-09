mod types;

use crate::errors;
use crate::tokenizer::types::token::Token;

use types::ast::Ast;
use types::chord::Chord;
use types::chord_detailed::ChordDetailed;
use types::chord_info::{ChordInfo, ChordOrUnidentified};
use types::chord_info_meta::ChordInfoMeta;
use types::section::Section;
use types::section_meta::SectionMeta;

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
                // TODO セクションメタインフォが無くても初期化されうるよ。どこで初期化必要か考えてね
                //      ・そもそもセクションが空配列な時
                //      ・改行が2つ以上重なる時（空行ができる時）
                //      ・以下の時

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
                        return Err(errors::SECTION_META_INFO_KEY_IS_INVALID.to_string());
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
                // if next token is denominator, read denominator
                let denominator: Option<String> = match tokens.peek() {
                    Some(Token::Slash) => {
                        tokens.next();
                        match tokens.next() {
                            Some(Token::Denominator(denominator)) => Some(denominator.clone()),
                            Some(Token::Slash) => {
                                return Err(
                                    errors::CHORD_SHOULD_NOT_CONTAINS_MULTIPLE_SLASHES.to_string()
                                )
                            }
                            _ => return Err("slashの後にdenominatorがないよ！".to_string()),
                        }
                    }
                    _ => None,
                };

                if chord_string == "?" {
                    // if chord_blocks is empty, make new chord_block
                    if sections.last_mut().unwrap().chord_blocks.is_empty() {
                        sections.last_mut().unwrap().chord_blocks.push(Vec::new());
                    }

                    // add unidentified chord to last chord block
                    sections
                        .last_mut()
                        .unwrap()
                        .chord_blocks
                        .last_mut()
                        .unwrap()
                        .push(ChordInfo {
                            chord: ChordOrUnidentified::Unidentified,
                            denominator,
                            meta_infos: tmp_chord_info_meta_infos.clone(),
                        });

                    // reset tmp_chord_info_meta_infos
                    tmp_chord_info_meta_infos = Vec::new();

                    continue;
                }

                let result = ChordDetailed::from_str(chord_string);
                if let Ok(detailed) = result {
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
                            chord: ChordOrUnidentified::Chord(chord),
                            denominator,
                            meta_infos: tmp_chord_info_meta_infos.clone(),
                        });

                    // reset tmp_chord_info_meta_infos
                    tmp_chord_info_meta_infos = Vec::new();
                } else {
                    return Err([
                        errors::CHORD_IS_INVALID.to_string(),
                        result.err().unwrap(),
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
            Token::Denominator(_) => { /* Nothing */ }
            Token::Comma => { /* Nothing */ }
            Token::ChordBlockSeparator => { /* Nothing */ } //
            Token::Equal => { /* Nothing */ }
            Token::Slash => { /* Nothing */ }
            _ => {
                // invalid token
                return Err(errors::INVALID_TOKEN_TYPE.to_string());
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
        fn section_meta_info() {
            let input = [
                Token::SectionMetaInfoStart,
                Token::SectionMetaInfoKey("section".to_string()),
                Token::Equal,
                Token::SectionMetaInfoValue("A".to_string()),
                Token::LineBreak,
            ];

            assert_eq!(
                parse(&input),
                Ok([Section {
                    meta_infos: vec![SectionMeta::Section {
                        value: "A".to_string(),
                    }],
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
                Token::Chord("F#m7-5".to_string()),
                Token::Slash,
                Token::Denominator("F#m7-5".to_string()),
                Token::ChordBlockSeparator,
                Token::Chord("Fbm13".to_string()),
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
                        chord: ChordOrUnidentified::Chord(Chord {
                            plain: "C".to_string(),
                            detailed: ChordDetailed {
                                base: Base::C,
                                accidental: None,
                                chord_type: ChordType::Major,
                                extension: None,
                            },
                        }),
                    },
                    ChordInfo {
                        chord: ChordOrUnidentified::Chord(Chord {
                            plain: "G".to_string(),
                            detailed: ChordDetailed {
                                base: Base::G,
                                accidental: None,
                                chord_type: ChordType::Major,
                                extension: None,
                            },
                        }),
                        denominator: Some("Bb".to_string()),
                        meta_infos: Vec::new(),
                    },
                    ChordInfo {
                        chord: ChordOrUnidentified::Chord(Chord {
                            plain: "Am".to_string(),
                            detailed: ChordDetailed {
                                base: Base::A,
                                accidental: None,
                                chord_type: ChordType::Minor,
                                extension: None,
                            },
                        }),
                        denominator: None,
                        meta_infos: Vec::new(),
                    },
                    ChordInfo {
                        chord: ChordOrUnidentified::Chord(Chord {
                            plain: "Em".to_string(),
                            detailed: ChordDetailed {
                                base: Base::E,
                                accidental: None,
                                chord_type: ChordType::Minor,
                                extension: None,
                            },
                        }),
                        denominator: Some("G".to_string()),
                        meta_infos: Vec::new(),
                    },
                    ChordInfo {
                        chord: ChordOrUnidentified::Chord(Chord {
                            plain: "F#m7-5".to_string(),
                            detailed: ChordDetailed {
                                base: Base::F,
                                accidental: Some(Accidental::Sharp),
                                chord_type: ChordType::Minor,
                                extension: Some(Extension::SevenFlatFive),
                            },
                        }),
                        denominator: Some("F#m7-5".to_string()),
                        meta_infos: Vec::new(),
                    },
                    ChordInfo {
                        chord: ChordOrUnidentified::Chord(Chord {
                            plain: "Fbm13".to_string(),
                            detailed: ChordDetailed {
                                base: Base::F,
                                accidental: Some(Accidental::Flat),
                                chord_type: ChordType::Minor,
                                extension: Some(Extension::Thirteen),
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
        fn chord_blocks_with_unidentified() {
            let input = [
                Token::ChordBlockSeparator,
                Token::Chord("?".to_string()),
                Token::ChordBlockSeparator,
            ];

            assert_eq!(
                parse(&input),
                Ok([Section {
                    meta_infos: Vec::new(),
                    chord_blocks: vec![vec![ChordInfo {
                        chord: ChordOrUnidentified::Unidentified,
                        denominator: None,
                        meta_infos: Vec::new(),
                    },],],
                }]
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
                            chord: ChordOrUnidentified::Chord(Chord {
                                plain: "C".to_string(),
                                detailed: ChordDetailed {
                                    base: Base::C,
                                    accidental: None,
                                    chord_type: ChordType::Major,
                                    extension: None,
                                },
                            },),
                            denominator: None,
                            meta_infos: Vec::new(),
                        },],],
                    },
                    Section {
                        meta_infos: Vec::new(),
                        chord_blocks: vec![vec![ChordInfo {
                            chord: ChordOrUnidentified::Chord(Chord {
                                plain: "C".to_string(),
                                detailed: ChordDetailed {
                                    base: Base::C,
                                    accidental: None,
                                    chord_type: ChordType::Major,
                                    extension: None,
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
}
