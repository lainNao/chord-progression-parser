mod types;

use crate::errors;
use crate::tokenizer::types::token::Token;

use types::ast::Ast;
use types::chord_info_meta::ChordInfoMeta;
use types::section::Section;
use types::section_meta::SectionMeta;
use types::chord::Chord;
use types::chord_detailed::ChordDetailed;

pub fn parse(tokens: &[Token]) -> Result<Ast, String> {
    let mut sections: Vec<Section> = Vec::new();
    let mut tokens = tokens.iter().peekable();

    while let Some(token) = tokens.next() {
        match token {
            // section meta info
            Token::SectionMetaInfoStart => {
                let is_new_section = 
                    // no section
                    sections.is_empty() ||
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
                    _ => return Err(errors::SECTION_META_INFO_KEY_SHOULD_NOT_CONTAINS_LINE_BREAK.to_string()),
                };

                // if next token is not Token::Equal, return error
                match tokens.next() {
                    Some(Token::Equal) => {}
                    _ => return Err(errors::SECTION_META_INFO_KEY_SHOULD_NOT_CONTAINS_LINE_BREAK.to_string())
                }

                // if next token is not Token::SectionMetaInfoValue, return error
                let section_meta_info_value = match tokens.next() {
                    Some(Token::SectionMetaInfoValue(value)) => value,
                    _ => return Err(errors::SECTION_META_INFO_VALUE_SHOULD_NOT_BE_EMPTY.to_string()),
                };

                // add section meta info to last section
                match section_meta_info_key.as_str() {
                    "section" => {
                        sections.last_mut().unwrap().meta_infos.push(SectionMeta::Section {
                            value: section_meta_info_value.clone(),
                        });
                    }
                    "repeat" => {
                        // if section_meta_info_value cannot parse as u32, return error
                        if section_meta_info_value.parse::<u32>().is_err() {
                            return Err(errors::SECTION_META_INFO_VALUE_OF_REPEAT_NEEDS_TO_BE_NUMBER.to_string());
                        }

                        sections.last_mut().unwrap().meta_infos.push(SectionMeta::Repeat {
                            value: section_meta_info_value.parse::<u32>().unwrap(),
                        });
                    }
                    _ => {
                        return Err(errors::SECTION_META_INFO_KEY_IS_INVALID.to_string());
                    }
                }
            
                match tokens.next() {
                    Some(Token::LineBreak) => {}
                    _ => {
                        return Err(errors::SECTION_META_INFO_VALUE_NEEDS_LINE_BREAK_AFTER.to_string());
                    }
                }
            },
            // meta info
            Token::MetaInfoStart => {//(
                
                // if next token is not Token::MetaInfoKey, return error
                let meta_info_key = match tokens.next() {
                    Some(Token::MetaInfoKey(value)) => value,
                    _ => return Err(errors::META_INFO_KEY_SHOULD_NOT_BE_EMPTY.to_string()),
                };
        
                // if next token is not Token::Equal, return error
                match tokens.next() {
                    Some(Token::Equal) => {}
                    _ => return Err(errors::META_INFO_KEY_SHOULD_NOT_CONTAINS_LINE_BREAK.to_string())
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

                        // add ChordInfoMeta to last ChordInfo
                        sections.last_mut().unwrap().chord_blocks.last_mut().unwrap().last_mut().unwrap().meta_infos.push(ChordInfoMeta::Key {
                            value: key_name, 
                        });
                    }
                    _ => {
                        return Err(errors::META_INFO_KEY_IS_INVALID.to_string());
                    }
                }

                // if next token is not Token::MetaInfoEnd, return error
                match tokens.next() {
                    Some(Token::MetaInfoEnd) => {}
                    _ => return Err(errors::META_INFO_VALUE_NEEDS_CLOSE_PARENTHESIS_AFTER.to_string())
                }
            }
            // chord
            Token::Chord(chord_string) => {
                // if next token is denominator, read denominator
                let denominator: Option<String> = match tokens.peek() {
                    Some(Token::Denominator(denominator)) => {
                        tokens.next();
                        Some(denominator.clone())
                    }
                    _ => None,
                };

                let chord_detailed  = ChordDetailed::from_str(chord_string);
                if let Ok(detailed) = chord_detailed {
                    let chord = Chord {
                        plain: chord_string.clone(),
                        detailed,
                    };
    
                    sections.last_mut().unwrap()
                        .chord_blocks.last_mut().unwrap().last_mut().unwrap()
                        .chord = chord;
                    
                    sections.last_mut().unwrap()
                        .chord_blocks.last_mut().unwrap().last_mut().unwrap()
                        .denominator = denominator;

                } else {
                    return Err([
                        errors::CHORD_IS_INVALID.to_string(),
                        chord_string.to_string()
                    ].join(": ")
                    );
                }
            }
            Token::LineBreak => {}
            Token::Denominator(_) => { /* Nothing */ }
            Token::Comma => { /* Nothing */ }
            Token::ChordBlockSeparator => { /* Nothing */ } // |
            Token::Equal => { /* Nothing */ }
            _ => {
                // invalid token
                return Err(errors::INVALID_TOKEN_TYPE.to_string())
            }
        }
    }

    Ok(sections)
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::chord_info::ChordInfo;
    use types::base::Base;
    use types::chord_type::ChordType;

    #[cfg(test)]
    mod success {
        use super::*;

        #[test]
        fn sample() {
            let input = [
                Token::SectionMetaInfoStart,
                Token::SectionMetaInfoKey("section".to_string()),
                Token::Equal,
                Token::SectionMetaInfoValue("A".to_string()),
                Token::LineBreak,
                Token::ChordBlockSeparator,
                Token::Chord("C".to_string()),
                Token::ChordBlockSeparator,
                Token::Chord("F".to_string()),
                Token::ChordBlockSeparator,
                Token::Chord("G".to_string()),
                Token::ChordBlockSeparator,
                Token::Chord("C".to_string()),
                Token::ChordBlockSeparator,
            ];
            let expected = [
                Section {
                    meta_infos: vec![
                        SectionMeta::Section {
                            value: "A".to_string(),
                        }
                    ],
                    chord_blocks: vec![
                        vec![
                            ChordInfo {
                                chord: Chord {
                                    plain: "C".to_string(),
                                    detailed: ChordDetailed {
                                        base: Base::C,
                                        accidental: None,
                                        chord_type: ChordType::Major,
                                        extension: None,
                                    },
                                },
                                denominator: None,
                                meta_infos: Vec::new(),
                            },
                            ChordInfo {
                                chord: Chord {
                                    plain: "F".to_string(),
                                    detailed: ChordDetailed {
                                        base: Base::F,
                                        accidental: None,
                                        chord_type: ChordType::Major,
                                        extension: None,
                                    },
                                },
                                denominator: None,
                                meta_infos: Vec::new(),
                            },
                            ChordInfo {
                                chord: Chord {
                                    plain: "G".to_string(),
                                    detailed: ChordDetailed {
                                        base: Base::G,
                                        accidental: None,
                                        chord_type: ChordType::Major,
                                        extension: None,
                                    },
                                },
                                denominator: None,
                                meta_infos: Vec::new(),
                            },
                            ChordInfo {
                                chord: Chord {
                                    plain: "C".to_string(),
                                    detailed: ChordDetailed {
                                        base: Base::C,
                                        accidental: None,
                                        chord_type: ChordType::Major,
                                        extension: None,
                                    },
                                },
                                denominator: None,
                                meta_infos: Vec::new(),
                            },
                        ]
                    ],
                }
            ];
            let parsed_result = parse(&input);
            assert_eq!(parsed_result, Ok(expected.to_vec()));
        }
    }
}
