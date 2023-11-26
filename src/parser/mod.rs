mod types;

use std::str::FromStr;

use crate::error_code::{ErrorCode, ErrorInfo, ErrorInfoWithPosition};
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::token_with_position::TokenWithPosition;
use crate::util::position::Position;

pub use types::ast::Ast;
use types::chord::Chord;
use types::chord_detailed::ChordDetailed;
use types::chord_expression::ChordExpression;
use types::chord_info::ChordInfo;
use types::chord_info_meta::ChordInfoMeta;
use types::section::Section;
use types::section_meta::SectionMeta;

use self::types::extension::Extension;

pub fn parse(token_with_position_list: &[TokenWithPosition]) -> Result<Ast, ErrorInfoWithPosition> {
    // if no token_with_position_list, return empty Ast
    if token_with_position_list.is_empty() {
        return Ok(Vec::new());
    }

    let mut sections: Vec<Section> = vec![Section {
        meta_infos: Vec::new(),
        chord_blocks: Vec::new(),
    }];
    let mut token_with_position_list = token_with_position_list.iter().peekable();
    let mut tmp_chord_info_meta_infos: Vec<ChordInfoMeta> = Vec::new();

    // return previous args
    let mut get_previous_token_with_position = {
        let mut previous_token_with_position: Option<TokenWithPosition> = None;

        // クロージャは `token_with_position` を受け取り、前回の値を返します。
        move |token_with_position: Option<TokenWithPosition>| -> Option<TokenWithPosition> {
            // 現在の値を一時変数に保存します。
            let current = previous_token_with_position.clone();

            // 新しい値で更新します。
            previous_token_with_position = token_with_position;

            // 前回の値を返します。
            current
        }
    };

    while let Some(token_with_position) = token_with_position_list.next() {
        let previous = get_previous_token_with_position(Some(token_with_position.clone()));

        match token_with_position.token.clone() {
            // section meta info
            Token::SectionMetaInfoStart => {
                // last section's chord_blocks is not empty
                let is_new_section = !sections.last().unwrap().chord_blocks.is_empty();

                // if is_new_section, initialize new section
                if is_new_section {
                    sections.push(Section {
                        meta_infos: Vec::new(),
                        chord_blocks: Vec::new(),
                    });
                }

                // if next token is not Token::SectionMetaInfoKey, return error
                let section_meta_info_key = match &token_with_position_list.next().unwrap().token {
                    Token::SectionMetaInfoKey(value) => value,
                    _ => {
                        return Err(ErrorInfoWithPosition {
                            error: ErrorInfo {
                                code: ErrorCode::Smik2,
                                additional_info: None,
                            },
                            position: token_with_position.position.clone().clone(),
                        })
                    }
                };

                // if next token is not Token::Equal, return error
                match token_with_position_list.next().unwrap().token {
                    Token::Equal => {}
                    _ => {
                        return Err(ErrorInfoWithPosition {
                            error: ErrorInfo {
                                code: ErrorCode::Smik2,
                                additional_info: None,
                            },
                            position: token_with_position.position.clone(),
                        })
                    }
                }

                // if next token is not Token::SectionMetaInfoValue, return error
                let section_meta_info_value = match &token_with_position_list.next().unwrap().token
                {
                    Token::SectionMetaInfoValue(value) => value,
                    _ => {
                        return Err(ErrorInfoWithPosition {
                            error: ErrorInfo {
                                code: ErrorCode::Smiv1,
                                additional_info: None,
                            },
                            position: token_with_position.position.clone(),
                        })
                    }
                };

                // add section meta info to last section
                match section_meta_info_key.as_str() {
                    "section" => sections
                        .last_mut()
                        .unwrap()
                        .meta_infos
                        .push(SectionMeta::Section(section_meta_info_value.clone())),
                    "repeat" => {
                        // if section_meta_info_value cannot parse as u32, return error
                        if section_meta_info_value.parse::<u32>().is_err() {
                            let cloned_token_with_position = token_with_position.clone();
                            let error_section_meta_info_value_column_number =
                                cloned_token_with_position.position.column_number
                                    + section_meta_info_key.as_str().len()
                                    + 1
                                    + section_meta_info_value.len();

                            return Err(ErrorInfoWithPosition {
                                error: ErrorInfo {
                                    code: ErrorCode::Smiv3,
                                    additional_info: None,
                                },
                                position: Position {
                                    line_number: cloned_token_with_position.position.line_number,
                                    column_number: error_section_meta_info_value_column_number,
                                    length: cloned_token_with_position.position.length,
                                },
                            });
                        }

                        sections
                            .last_mut()
                            .unwrap()
                            .meta_infos
                            .push(SectionMeta::Repeat(
                                section_meta_info_value.parse::<u32>().unwrap(),
                            ));
                    }
                    _ => {
                        return Err(ErrorInfoWithPosition {
                            error: ErrorInfo {
                                code: ErrorCode::Smik1,
                                additional_info: Some(section_meta_info_key.to_string()),
                            },
                            position: Position {
                                line_number: token_with_position.position.line_number,
                                column_number: token_with_position.position.column_number + 1,
                                length: section_meta_info_key.len(),
                            },
                        });
                    }
                }

                if token_with_position_list.peek().is_none() {
                    continue;
                }

                match token_with_position_list.peek().unwrap().token {
                    Token::LineBreak => {
                        token_with_position_list.next();

                        if token_with_position_list.peek().is_none() {
                            continue;
                        }

                        match token_with_position_list.peek().unwrap().token {
                            Token::LineBreak => {
                                token_with_position_list.next();

                                match token_with_position_list.peek().unwrap().token {
                                    Token::LineBreak => {
                                        // if line break appears three times in a row, return error
                                        return Err(ErrorInfoWithPosition {
                                            error: ErrorInfo {
                                                code: ErrorCode::Bl1,
                                                additional_info: None,
                                            },
                                            position: token_with_position.position.clone(),
                                        });
                                    }
                                    _ => { /* Nothing */ }
                                }
                            }
                            _ => { /* Nothing */ }
                        }
                    }
                    _ => {
                        return Err(ErrorInfoWithPosition {
                            error: ErrorInfo {
                                code: ErrorCode::Smiv2,
                                additional_info: None,
                            },
                            position: token_with_position_list.peek().unwrap().position.clone(),
                        });
                    }
                }
            }
            // meta info
            Token::MetaInfoStart => {
                //(

                // if next token is not Token::MetaInfoKey, return error
                let meta_info_key = match &token_with_position_list.next().unwrap().token {
                    Token::MetaInfoKey(value) => value,
                    _ => {
                        return Err(ErrorInfoWithPosition {
                            error: ErrorInfo {
                                code: ErrorCode::Cimk2,
                                additional_info: None,
                            },
                            position: token_with_position.position.clone().clone(),
                        })
                    }
                };

                // if next token is not Token::Equal, return error
                match token_with_position_list.next().unwrap().token {
                    Token::Equal => {}
                    _ => {
                        return Err(ErrorInfoWithPosition {
                            error: ErrorInfo {
                                code: ErrorCode::Cimk1,
                                additional_info: None,
                            },
                            position: token_with_position.position.clone(),
                        })
                    }
                }

                // if next token is not Token::MetaInfoValue, return error
                let meta_info_value = match &token_with_position_list.next().unwrap().token {
                    Token::MetaInfoValue(value) => value,
                    _ => {
                        return Err(ErrorInfoWithPosition {
                            error: ErrorInfo {
                                code: ErrorCode::Cimv2,
                                additional_info: None,
                            },
                            position: token_with_position.position.clone(),
                        })
                    }
                };

                // add meta info to last chord block
                match meta_info_key.as_str() {
                    "key" => {
                        let key_name = match meta_info_value.parse() {
                            Ok(key) => key,
                            Err(_) => {
                                return Err(ErrorInfoWithPosition {
                                    error: ErrorInfo {
                                        code: ErrorCode::Cimv4,
                                        additional_info: None,
                                    },
                                    position: token_with_position.position.clone(),
                                })
                            }
                        };

                        // add ChordInfoMeta to temporary variable
                        tmp_chord_info_meta_infos.push(ChordInfoMeta::Key(key_name));
                    }
                    _ => {
                        return Err(ErrorInfoWithPosition {
                            error: ErrorInfo {
                                code: ErrorCode::Cimk3,
                                additional_info: None,
                            },
                            position: token_with_position.position.clone(),
                        });
                    }
                }

                // if next token is not Token::MetaInfoEnd, return error
                match token_with_position_list.next().unwrap().token {
                    Token::MetaInfoEnd => {}
                    _ => {
                        return Err(ErrorInfoWithPosition {
                            error: ErrorInfo {
                                code: ErrorCode::Cimv3,
                                additional_info: None,
                            },
                            position: token_with_position.position.clone(),
                        })
                    }
                }
            }
            // chord
            Token::Chord(chord_string) => {
                // chord expression of "-" or "?" or "%"
                if chord_string.eq("-") || chord_string.eq("?") || chord_string.eq("%") {
                    if sections.last_mut().unwrap().chord_blocks.is_empty() && chord_string == "%" {
                        return Err(ErrorInfoWithPosition {
                            error: ErrorInfo {
                                code: ErrorCode::Chb1,
                                additional_info: None,
                            },
                            position: token_with_position.position.clone(),
                        });
                    }

                    if previous.is_some() && previous.clone().unwrap().token == Token::Comma {
                        // add ChordInfo to last chord block
                        sections
                            .last_mut()
                            .unwrap()
                            .chord_blocks
                            .last_mut()
                            .unwrap()
                            .push(ChordInfo {
                                chord_expression: match chord_string.as_str() {
                                    "?" => ChordExpression::Unidentified,
                                    "%" => ChordExpression::Same,
                                    _ => {
                                        return Err(ErrorInfoWithPosition {
                                            error: ErrorInfo {
                                                code: ErrorCode::Cho1,
                                                additional_info: None,
                                            },
                                            position: token_with_position.position.clone(),
                                        });
                                    }
                                },
                                denominator: None,
                                meta_infos: tmp_chord_info_meta_infos.clone(),
                            });

                        // reset tmp_chord_info_meta_infos
                        tmp_chord_info_meta_infos = Vec::new();
                    } else {
                        println!("2322222{:?}", previous);

                        // add ChordInfo to last chord block
                        sections
                            .last_mut()
                            .unwrap()
                            .chord_blocks
                            .push(vec![ChordInfo {
                                chord_expression: match chord_string.as_str() {
                                    "?" => ChordExpression::Unidentified,
                                    "%" => ChordExpression::Same,
                                    _ => {
                                        return Err(ErrorInfoWithPosition {
                                            error: ErrorInfo {
                                                code: ErrorCode::Cho1,
                                                additional_info: None,
                                            },
                                            position: token_with_position.position.clone(),
                                        });
                                    }
                                },
                                denominator: None,
                                meta_infos: tmp_chord_info_meta_infos.clone(),
                            }]);

                        // reset tmp_chord_info_meta_infos
                        tmp_chord_info_meta_infos = Vec::new();
                    }

                    continue;
                }

                let chord_detailed_result = ChordDetailed::from_str(&chord_string);

                // mutate line_number and column_number
                if let Ok(detailed) = chord_detailed_result {
                    let chord = Chord {
                        plain: chord_string.clone(),
                        detailed,
                    };

                    if previous.is_some() && previous.clone().unwrap().token == Token::Comma {
                        sections
                            .last_mut()
                            .unwrap()
                            .chord_blocks
                            .last_mut()
                            .unwrap()
                            .push(ChordInfo {
                                chord_expression: ChordExpression::Chord(chord),
                                denominator: None,
                                meta_infos: tmp_chord_info_meta_infos.clone(),
                            });
                    } else {
                        //  make new chord_block
                        sections
                            .last_mut()
                            .unwrap()
                            .chord_blocks
                            .push(vec![ChordInfo {
                                chord_expression: ChordExpression::Chord(chord),
                                denominator: None,
                                meta_infos: tmp_chord_info_meta_infos.clone(),
                            }]);
                    }

                    // reset tmp_chord_info_meta_infos
                    tmp_chord_info_meta_infos = Vec::new();
                } else {
                    return Err(ErrorInfoWithPosition {
                        error: ErrorInfo {
                            code: ErrorCode::Cho1,
                            additional_info: Some(
                                [
                                    chord_detailed_result.err().unwrap().code.to_string(),
                                    chord_string.to_string(),
                                ]
                                .join(": "),
                            ),
                        },
                        position: token_with_position.position.clone(),
                    });
                }
            }
            Token::LineBreak => {
                let peeked_token_with_position_list = token_with_position_list.peek();
                if peeked_token_with_position_list.is_none() {
                    continue;
                }

                // if Token::LineBreak appears two or more times in a row, create new section
                match peeked_token_with_position_list.unwrap().token {
                    Token::LineBreak => {
                        token_with_position_list.next();

                        // if next is ChordBlockSeparator, create new section
                        match token_with_position_list.peek().unwrap().token {
                            Token::MetaInfoStart | Token::Chord(_) => {
                                // create new section
                                sections.push(Section {
                                    meta_infos: Vec::new(),
                                    chord_blocks: Vec::new(),
                                });
                            }
                            Token::LineBreak => {
                                // error
                                return Err(ErrorInfoWithPosition {
                                    error: ErrorInfo {
                                        code: ErrorCode::Bl1,
                                        additional_info: None,
                                    },
                                    position: token_with_position.position.clone(),
                                });
                            }
                            _ => { /* Nothing */ }
                        }
                    }
                    _ => { /* Nothing */ }
                }
            }
            Token::Extension(ext_str) => {
                // if ext_str doesn't in Extension enum, error
                if Extension::from_str(&ext_str).is_err() {
                    let cloned_token_with_position = token_with_position.clone();
                    return Err(ErrorInfoWithPosition {
                        error: ErrorInfo {
                            code: ErrorCode::Ext1,
                            additional_info: Some(ext_str.to_string()),
                        },
                        position: Position {
                            line_number: cloned_token_with_position.position.line_number,
                            column_number: cloned_token_with_position.position.column_number,
                            length: ext_str.len(),
                        },
                    });
                }

                match &sections
                    .last_mut()
                    .unwrap()
                    .chord_blocks
                    .last_mut()
                    .unwrap()
                    .last_mut()
                    .unwrap()
                    .chord_expression
                {
                    ChordExpression::Unidentified => {}
                    ChordExpression::Same => {}
                    ChordExpression::NoChord => {}
                    ChordExpression::Chord(c) => {
                        let mut parsed_extensions = vec![Extension::from_str(&ext_str).unwrap()];

                        // REFACTOR: please remove this flag variable for refactoring
                        let mut is_previous_token_is_comma = false;

                        for t in token_with_position_list.by_ref() {
                            // validation
                            match &t.token {
                                Token::Comma => {
                                    if is_previous_token_is_comma {
                                        return Err(ErrorInfoWithPosition {
                                            error: ErrorInfo {
                                                code: ErrorCode::Ext2,
                                                additional_info: None,
                                            },
                                            position: t.position.clone(),
                                        });
                                    }
                                    is_previous_token_is_comma = true
                                }
                                _ => is_previous_token_is_comma = false,
                            }

                            match &t.token {
                                Token::ExtensionEnd => {
                                    let peeked_token_with_position_list =
                                        token_with_position_list.peek();

                                    if peeked_token_with_position_list.is_none() {
                                        break;
                                    }

                                    // if next token is ExtensionStart, error
                                    if let Token::ExtensionStart =
                                        peeked_token_with_position_list.unwrap().token
                                    {
                                        return Err(ErrorInfoWithPosition {
                                            error: ErrorInfo {
                                                code: ErrorCode::Ext4,
                                                additional_info: None,
                                            },
                                            position: token_with_position.position.clone(),
                                        });
                                    }

                                    break;
                                }
                                Token::Comma => {
                                    continue;
                                }
                                Token::Extension(ext_str) => {
                                    if Extension::from_str(ext_str).is_err() {
                                        let cloned_token_with_position =
                                            token_with_position.clone();

                                        let extensions_before_current_length = parsed_extensions
                                            .iter()
                                            .map(|e| e.to_string() + ",")
                                            .collect::<Vec<String>>()
                                            .join("");

                                        return Err(ErrorInfoWithPosition {
                                            error: ErrorInfo {
                                                code: ErrorCode::Ext1,
                                                additional_info: Some(ext_str.to_string()),
                                            },
                                            position: Position {
                                                line_number: cloned_token_with_position
                                                    .position
                                                    .line_number,
                                                column_number: cloned_token_with_position
                                                    .position
                                                    .column_number
                                                    + extensions_before_current_length.len(),
                                                length: ext_str.len(),
                                            },
                                        });
                                    }
                                    parsed_extensions.push(Extension::from_str(ext_str).unwrap());
                                }
                                _ => {
                                    let cloned_token_with_position = token_with_position.clone();
                                    return Err(ErrorInfoWithPosition {
                                        error: ErrorInfo {
                                            code: ErrorCode::Ext1,
                                            additional_info: Some(t.token.to_string()),
                                        },
                                        position: Position {
                                            line_number: cloned_token_with_position
                                                .position
                                                .line_number,
                                            column_number: cloned_token_with_position
                                                .position
                                                .column_number,
                                            length: t.token.to_string().len(),
                                        },
                                    });
                                }
                            }
                        }
                        let extension_str_with_parenthesis = format!(
                            "({})",
                            parsed_extensions
                                .iter()
                                .map(|e| e.to_string())
                                .collect::<Vec<String>>()
                                .join(",")
                        );

                        sections
                            .last_mut() //last section
                            .unwrap()
                            .chord_blocks
                            .last_mut() // last chord block
                            .unwrap()
                            .last_mut() // last chord in comma separated chords
                            .unwrap()
                            .chord_expression = ChordExpression::Chord(Chord {
                            plain: [c.plain.clone(), extension_str_with_parenthesis.to_string()]
                                .concat(),
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
                if sections.last_mut().unwrap().chord_blocks.last().is_none() {
                    return Err(ErrorInfoWithPosition {
                        error: ErrorInfo {
                            code: ErrorCode::Cho3,
                            additional_info: None,
                        },
                        position: token_with_position.position.clone(),
                    });
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
                    .denominator
                    .is_some()
                {
                    return Err(ErrorInfoWithPosition {
                        error: ErrorInfo {
                            code: ErrorCode::Den1,
                            additional_info: None,
                        },
                        position: token_with_position.position.clone(),
                    });
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
            }
            Token::Comma => { /* Nothing */ }
            Token::ChordBlockSeparator => {
                // if previous is not Chord, error
                match previous {
                    None => {
                        return Err(ErrorInfoWithPosition {
                            error: ErrorInfo {
                                code: ErrorCode::Cho3,
                                additional_info: token_with_position.token.to_string().into(),
                            },
                            position: token_with_position.position.clone(),
                        });
                    }
                    _ => {
                        match previous.unwrap().token {
                            Token::Chord(_) | Token::Denominator(_) | Token::Extension(_) => { /* Nothing */
                            }
                            _ => {
                                return Err(ErrorInfoWithPosition {
                                    error: ErrorInfo {
                                        code: ErrorCode::Cho3,
                                        additional_info: token_with_position
                                            .token
                                            .to_string()
                                            .into(),
                                    },
                                    position: token_with_position.position.clone(),
                                });
                            }
                        }
                    }
                }

                // ?
                if token_with_position_list.peek().is_none() {
                    continue;
                }

                // if last and second last token is BreakLine, create new Section
                match token_with_position_list.peek().unwrap().token {
                    Token::ChordBlockSeparator => {
                        // TODO: ここにも例の（コードブロックなのかCSVなインフォの方か）のif分岐を作成？
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
                                chord_expression: ChordExpression::NoChord,
                                denominator: None,
                                meta_infos: tmp_chord_info_meta_infos.clone(),
                            });
                    }
                    _ => { /* Nothing */ }
                }
            }
            Token::Equal => { /* Nothing */ }
            Token::Slash => { /* Nothing */ }
            Token::ExtensionStart => {
                // if next token is not Extension, error
                match token_with_position_list.peek().unwrap().token {
                    Token::Extension(_) => { /* Nothing */ }
                    _ => {
                        return Err(ErrorInfoWithPosition {
                            error: ErrorInfo {
                                code: ErrorCode::Ext2,
                                additional_info: None,
                            },
                            position: token_with_position.position.clone(),
                        });
                    }
                }
            }
            Token::ExtensionEnd => { /* Nothing */ }
            _ => {
                // invalid token
                return Err(ErrorInfoWithPosition {
                    error: ErrorInfo {
                        code: ErrorCode::Tkn1,
                        additional_info: Some(token_with_position.token.to_string()),
                    },
                    position: token_with_position.position.clone(),
                });
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
        use crate::util::position::Position;

        #[test]
        fn multiple_break_line_under_section_meta_line() {
            let input = [
                TokenWithPosition {
                    token: Token::SectionMetaInfoStart,
                    position: Position {
                        line_number: 1,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoKey("section".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 2,
                        length: 6,
                    },
                },
                TokenWithPosition {
                    token: Token::Equal,
                    position: Position {
                        line_number: 1,
                        column_number: 9,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoValue("A".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 10,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 1,
                        column_number: 11,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 2,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 3,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 3,
                        column_number: 2,
                        length: 1,
                    },
                },
            ];
            let result = parse(&input);

            assert_eq!(
                result.unwrap(),
                [Section {
                    meta_infos: vec![SectionMeta::Section("A".to_string())],
                    chord_blocks: vec![vec![ChordInfo {
                        chord_expression: ChordExpression::Chord(Chord {
                            plain: "C".to_string(),
                            detailed: ChordDetailed {
                                base: Base::C,
                                accidental: None,
                                chord_type: ChordType::Major,
                                extensions: Vec::new(),
                            },
                        }),
                        denominator: None,
                        meta_infos: Vec::new(),
                    },],],
                },]
            );
        }

        #[test]
        fn empty() {
            let input = [];
            let result = parse(&input);

            assert_eq!(result.unwrap(), []);
        }

        #[test]
        fn section_meta_info() {
            let input = [
                TokenWithPosition {
                    token: Token::SectionMetaInfoStart,
                    position: Position {
                        line_number: 1,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoKey("section".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 2,
                        length: 6,
                    },
                },
                TokenWithPosition {
                    token: Token::Equal,
                    position: Position {
                        line_number: 1,
                        column_number: 9,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoValue("A".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 10,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 1,
                        column_number: 11,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoStart,
                    position: Position {
                        line_number: 2,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoKey("repeat".to_string()),
                    position: Position {
                        line_number: 2,
                        column_number: 2,
                        length: 5,
                    },
                },
                TokenWithPosition {
                    token: Token::Equal,
                    position: Position {
                        line_number: 2,
                        column_number: 9,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoValue("3".to_string()),
                    position: Position {
                        line_number: 2,
                        column_number: 10,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 2,
                        column_number: 11,
                        length: 1,
                    },
                },
            ];

            assert_eq!(
                parse(&input),
                Ok([Section {
                    meta_infos: vec![
                        SectionMeta::Section("A".to_string()),
                        SectionMeta::Repeat(3),
                    ],
                    chord_blocks: Vec::new(),
                }]
                .to_vec())
            );
        }

        #[test]
        fn multiple_section_meta_info() {
            let input = [
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 1,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoStart,
                    position: Position {
                        line_number: 2,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoKey("section".to_string()),
                    position: Position {
                        line_number: 2,
                        column_number: 2,
                        length: 6,
                    },
                },
                TokenWithPosition {
                    token: Token::Equal,
                    position: Position {
                        line_number: 2,
                        column_number: 9,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoValue("A".to_string()),
                    position: Position {
                        line_number: 2,
                        column_number: 10,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 2,
                        column_number: 11,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoStart,
                    position: Position {
                        line_number: 3,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoKey("section".to_string()),
                    position: Position {
                        line_number: 3,
                        column_number: 2,
                        length: 6,
                    },
                },
                TokenWithPosition {
                    token: Token::Equal,
                    position: Position {
                        line_number: 3,
                        column_number: 9,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoValue("AA".to_string()),
                    position: Position {
                        line_number: 3,
                        column_number: 10,
                        length: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 3,
                        column_number: 12,
                        length: 1,
                    },
                },
            ];

            assert_eq!(
                parse(&input),
                Ok([Section {
                    meta_infos: vec![
                        SectionMeta::Section("A".to_string()),
                        SectionMeta::Section("AA".to_string())
                    ],
                    chord_blocks: Vec::new(),
                }]
                .to_vec())
            );
        }

        #[test]
        fn chord_blocks_with_fraction_chord() {
            let input = [
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 1,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 2,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 2,
                        column_number: 2,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("G".to_string()),
                    position: Position {
                        line_number: 2,
                        column_number: 3,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Slash,
                    position: Position {
                        line_number: 2,
                        column_number: 4,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Denominator("Bb".to_string()),
                    position: Position {
                        line_number: 2,
                        column_number: 5,
                        length: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 2,
                        column_number: 6,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("Am".to_string()),
                    position: Position {
                        line_number: 2,
                        column_number: 7,
                        length: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 2,
                        column_number: 8,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("Em".to_string()),
                    position: Position {
                        line_number: 2,
                        column_number: 9,
                        length: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::Slash,
                    position: Position {
                        line_number: 2,
                        column_number: 11,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Denominator("G".to_string()),
                    position: Position {
                        line_number: 2,
                        column_number: 12,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 2,
                        column_number: 13,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("F#m".to_string()),
                    position: Position {
                        line_number: 3,
                        column_number: 1,
                        length: 3,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionStart,
                    position: Position {
                        line_number: 3,
                        column_number: 4,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Extension("7".to_string()),
                    position: Position {
                        line_number: 3,
                        column_number: 5,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Comma,
                    position: Position {
                        line_number: 3,
                        column_number: 6,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Extension("b5".to_string()),
                    position: Position {
                        line_number: 3,
                        column_number: 7,
                        length: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionEnd,
                    position: Position {
                        line_number: 3,
                        column_number: 8,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Slash,
                    position: Position {
                        line_number: 3,
                        column_number: 9,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Denominator("F#m(7,b5)".to_string()),
                    position: Position {
                        line_number: 3,
                        column_number: 10,
                        length: 8,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 3,
                        column_number: 19,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("Fbm".to_string()),
                    position: Position {
                        line_number: 3,
                        column_number: 20,
                        length: 3,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionStart,
                    position: Position {
                        line_number: 3,
                        column_number: 23,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Extension("13".to_string()),
                    position: Position {
                        line_number: 3,
                        column_number: 24,
                        length: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionEnd,
                    position: Position {
                        line_number: 3,
                        column_number: 26,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Slash,
                    position: Position {
                        line_number: 3,
                        column_number: 27,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Denominator("G7".to_string()),
                    position: Position {
                        line_number: 3,
                        column_number: 28,
                        length: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 3,
                        column_number: 30,
                        length: 1,
                    },
                },
            ];

            let expected = [Section {
                meta_infos: Vec::new(),
                chord_blocks: vec![
                    vec![ChordInfo {
                        meta_infos: Vec::new(),
                        denominator: None,
                        chord_expression: ChordExpression::Chord(Chord {
                            plain: "C".to_string(),
                            detailed: ChordDetailed {
                                base: Base::C,
                                accidental: None,
                                chord_type: ChordType::Major,
                                extensions: Vec::new(),
                            },
                        }),
                    }],
                    vec![ChordInfo {
                        chord_expression: ChordExpression::Chord(Chord {
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
                    }],
                    vec![ChordInfo {
                        chord_expression: ChordExpression::Chord(Chord {
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
                    }],
                    vec![ChordInfo {
                        chord_expression: ChordExpression::Chord(Chord {
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
                    }],
                    vec![ChordInfo {
                        chord_expression: ChordExpression::Chord(Chord {
                            plain: "F#m(7,b5)".to_string(),
                            detailed: ChordDetailed {
                                base: Base::F,
                                accidental: Some(Accidental::Sharp),
                                chord_type: ChordType::Minor,
                                extensions: vec![Extension::Seven, Extension::FlatFive],
                            },
                        }),
                        denominator: Some("F#m(7,b5)".to_string()),
                        meta_infos: Vec::new(),
                    }],
                    vec![ChordInfo {
                        chord_expression: ChordExpression::Chord(Chord {
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
                    }],
                ],
            }];
            let parsed_result = parse(&input);
            assert_eq!(parsed_result, Ok(expected.to_vec()));
        }

        #[test]
        fn same_chord_symbol_should_not_be_placed_first_of_chord_block() {
            let input = [TokenWithPosition {
                token: Token::Chord("%".to_string()),
                position: Position {
                    line_number: 2,
                    column_number: 1,
                    length: 1,
                },
            }];

            assert_eq!(
                parse(&input),
                Err(ErrorInfoWithPosition {
                    error: ErrorInfo {
                        code: ErrorCode::Chb1,
                        additional_info: None,
                    },
                    position: Position {
                        line_number: 2,
                        column_number: 1,
                        length: 1,
                    },
                })
            );
        }

        #[test]
        fn chord_blocks_with_expressions() {
            let input = [
                TokenWithPosition {
                    token: Token::Chord("?".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 1,
                        column_number: 2,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("%".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 3,
                        length: 1,
                    },
                },
            ];

            assert_eq!(
                parse(&input),
                Ok([Section {
                    meta_infos: Vec::new(),
                    chord_blocks: vec![
                        vec![ChordInfo {
                            chord_expression: ChordExpression::Unidentified,
                            denominator: None,
                            meta_infos: Vec::new(),
                        },],
                        vec![ChordInfo {
                            chord_expression: ChordExpression::Same,
                            denominator: None,
                            meta_infos: Vec::new(),
                        },]
                    ],
                },]
                .to_vec())
            );
        }

        #[test]
        fn multiple_section_without_section_meta() {
            let input = [
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 1,
                        column_number: 2,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 2,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 3,
                        column_number: 1,
                        length: 1,
                    },
                },
            ];

            assert_eq!(
                parse(&input),
                Ok([
                    Section {
                        meta_infos: Vec::new(),
                        chord_blocks: vec![vec![ChordInfo {
                            chord_expression: ChordExpression::Chord(Chord {
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
                            chord_expression: ChordExpression::Chord(Chord {
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
        use crate::{
            error_code::{ErrorCode, ErrorInfo, ErrorInfoWithPosition},
            parser::parse,
            tokenizer::types::{token::Token, token_with_position::TokenWithPosition},
            util::position::Position,
        };

        // C(9,,11) is error
        #[test]
        fn empty_extension() {
            let input = [
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionStart,
                    position: Position {
                        line_number: 1,
                        column_number: 2,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Extension("9".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 3,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Comma,
                    position: Position {
                        line_number: 1,
                        column_number: 4,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Comma,
                    position: Position {
                        line_number: 1,
                        column_number: 5,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Extension("11".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 6,
                        length: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionEnd,
                    position: Position {
                        line_number: 1,
                        column_number: 8,
                        length: 1,
                    },
                },
            ];

            assert_eq!(
                parse(&input),
                Err(ErrorInfoWithPosition {
                    error: ErrorInfo {
                        code: ErrorCode::Ext2,
                        additional_info: None,
                    },
                    position: Position {
                        line_number: 1,
                        column_number: 5,
                        length: 1,
                    },
                })
            );
        }

        #[test]
        fn only_chord_block_separator() {
            let input = [TokenWithPosition {
                token: Token::ChordBlockSeparator,
                position: Position {
                    line_number: 1,
                    column_number: 1,
                    length: 1,
                },
            }];

            let result = parse(&input).unwrap_err();

            assert_eq!(result.error.code, ErrorCode::Cho3);
            assert_eq!(result.position.line_number, 1);
            assert_eq!(result.position.column_number, 1);
            assert_eq!(result.position.length, 1);
        }

        #[test]
        fn chord_block_separator_after_line_break() {
            let input = [
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 1,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 2,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 2,
                        column_number: 1,
                        length: 1,
                    },
                },
            ];

            let result = parse(&input).unwrap_err();

            assert_eq!(result.error.code, ErrorCode::Cho3);
            assert_eq!(result.position.line_number, 2);
            assert_eq!(result.position.column_number, 1);
            assert_eq!(result.position.length, 1);
        }

        #[test]
        // if line break appears three times in a row, return error
        fn no_line_breaks_three_times_in_a_row() {
            let input = [
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 1,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 2,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 3,
                        column_number: 1,
                        length: 1,
                    },
                },
            ];

            assert_eq!(
                parse(&input),
                Err(ErrorInfoWithPosition {
                    error: ErrorInfo {
                        code: ErrorCode::Bl1,
                        additional_info: None,
                    },
                    position: Position {
                        line_number: 1,
                        column_number: 1,
                        length: 1,
                    },
                })
            );
        }

        #[test]
        fn invalid_extension_after_comma() {
            let input = [
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionStart,
                    position: Position {
                        line_number: 1,
                        column_number: 2,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Extension("9".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 3,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Comma,
                    position: Position {
                        line_number: 1,
                        column_number: 4,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Extension("1".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 5,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionEnd,
                    position: Position {
                        line_number: 1,
                        column_number: 6,
                        length: 1,
                    },
                },
            ];
            let result = parse(&input);

            assert_eq!(
                result,
                Err(ErrorInfoWithPosition {
                    error: ErrorInfo {
                        code: ErrorCode::Ext1,
                        additional_info: Some("1".to_string()),
                    },
                    position: Position {
                        line_number: 1,
                        column_number: 5,
                        length: 1,
                    },
                })
            );
        }

        #[test]
        fn no_multiple_extension_parenthesis() {
            let input = [
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionStart,
                    position: Position {
                        line_number: 1,
                        column_number: 2,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Extension("9".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 3,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionEnd,
                    position: Position {
                        line_number: 1,
                        column_number: 4,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionStart,
                    position: Position {
                        line_number: 1,
                        column_number: 5,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Extension("13".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 6,
                        length: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionEnd,
                    position: Position {
                        line_number: 1,
                        column_number: 8,
                        length: 1,
                    },
                },
            ];
            let result = parse(&input);

            assert_eq!(
                result,
                Err(ErrorInfoWithPosition {
                    error: ErrorInfo {
                        code: ErrorCode::Ext4,
                        additional_info: None,
                    },
                    position: Position {
                        line_number: 1,
                        column_number: 3, // TODO: I'd like to make it 6, but I've still handled all the extensions together, so once 6
                        length: 1,
                    },
                })
            );
        }

        #[test]
        fn empty_line_continue() {
            let input = [
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 1,
                        column_number: 2,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 2,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 3,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 4,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 4,
                        column_number: 2,
                        length: 1,
                    },
                },
            ];
            let result = parse(&input);

            assert_eq!(
                result,
                Err(ErrorInfoWithPosition {
                    error: ErrorInfo {
                        code: ErrorCode::Bl1,
                        additional_info: None,
                    },
                    position: Position {
                        line_number: 1,
                        column_number: 2,
                        length: 1,
                    },
                })
            );
        }

        #[test]
        fn invalid_extension() {
            let input = [
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionStart,
                    position: Position {
                        line_number: 1,
                        column_number: 2,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Extension("1".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 3,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionEnd,
                    position: Position {
                        line_number: 1,
                        column_number: 4,
                        length: 1,
                    },
                },
            ];
            let result = parse(&input);

            assert_eq!(
                result,
                Err(ErrorInfoWithPosition {
                    error: ErrorInfo {
                        code: ErrorCode::Ext1,
                        additional_info: Some("1".to_string()),
                    },
                    position: Position {
                        line_number: 1,
                        column_number: 3,
                        length: 1,
                    },
                })
            );
        }

        #[test]
        fn section_meta_info_value_of_repeat_needs_to_be_number() {
            let input = [
                TokenWithPosition {
                    token: Token::SectionMetaInfoStart,
                    position: Position {
                        line_number: 1,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoKey("repeat".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 2,
                        length: 6,
                    },
                },
                TokenWithPosition {
                    token: Token::Equal,
                    position: Position {
                        line_number: 1,
                        column_number: 8,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoValue("A".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 9,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 1,
                        column_number: 10,
                        length: 1,
                    },
                },
            ];

            assert_eq!(
                parse(&input),
                Err(ErrorInfoWithPosition {
                    error: ErrorInfo {
                        code: ErrorCode::Smiv3,
                        additional_info: None,
                    },
                    position: Position {
                        line_number: 1,
                        column_number: 9,
                        length: 1,
                    },
                })
            );
        }

        #[test]
        fn section_meta_info_key_is_invalid() {
            let input = [
                TokenWithPosition {
                    token: Token::SectionMetaInfoStart,
                    position: Position {
                        line_number: 1,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoKey("asdf".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 2,
                        length: 4,
                    },
                },
                TokenWithPosition {
                    token: Token::Equal,
                    position: Position {
                        line_number: 1,
                        column_number: 6,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoValue("A".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 7,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 1,
                        column_number: 8,
                        length: 1,
                    },
                },
            ];

            assert_eq!(
                parse(&input),
                Err(ErrorInfoWithPosition {
                    error: ErrorInfo {
                        code: ErrorCode::Smik1,
                        additional_info: Some("asdf".to_string()),
                    },
                    position: Position {
                        line_number: 1,
                        column_number: 2,
                        length: 4,
                    },
                })
            );
        }

        #[test]
        fn section_meta_info_value_needs_line_break_after() {
            let input = [
                TokenWithPosition {
                    token: Token::SectionMetaInfoStart,
                    position: Position {
                        line_number: 1,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoKey("section".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 2,
                        length: 7,
                    },
                },
                TokenWithPosition {
                    token: Token::Equal,
                    position: Position {
                        line_number: 1,
                        column_number: 9,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoValue("A".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 10,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 1,
                        column_number: 11,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 1,
                        column_number: 12,
                        length: 1,
                    },
                },
            ];

            assert_eq!(
                parse(&input),
                Err(ErrorInfoWithPosition {
                    error: ErrorInfo {
                        code: ErrorCode::Smiv2,
                        additional_info: None,
                    },
                    position: Position {
                        line_number: 1,
                        column_number: 11,
                        length: 1,
                    },
                })
            );
        }

        #[test]
        fn chord_should_not_be_empty() {
            let input = [
                TokenWithPosition {
                    token: Token::Slash,
                    position: Position {
                        line_number: 1,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Denominator("D".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 2,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 1,
                        column_number: 3,
                        length: 1,
                    },
                },
            ];

            assert_eq!(
                parse(&input),
                Err(ErrorInfoWithPosition {
                    error: ErrorInfo {
                        code: ErrorCode::Cho3,
                        additional_info: None,
                    },
                    position: Position {
                        line_number: 1,
                        column_number: 2,
                        length: 1,
                    },
                })
            );
        }

        #[test]
        fn denominator_is_limited_to_one_per_chord() {
            let input = [
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Slash,
                    position: Position {
                        line_number: 1,
                        column_number: 2,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Denominator("D".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 3,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Slash,
                    position: Position {
                        line_number: 1,
                        column_number: 4,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Denominator("E".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 5,
                        length: 1,
                    },
                },
            ];

            assert_eq!(
                parse(&input),
                Err(ErrorInfoWithPosition {
                    error: ErrorInfo {
                        code: ErrorCode::Den1,
                        additional_info: None,
                    },
                    position: Position {
                        line_number: 1,
                        column_number: 5,
                        length: 1,
                    },
                })
            );
        }

        #[test]
        fn extension_must_not_be_empty() {
            let input = [
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionStart,
                    position: Position {
                        line_number: 1,
                        column_number: 2,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionEnd,
                    position: Position {
                        line_number: 1,
                        column_number: 3,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 1,
                        column_number: 4,
                        length: 1,
                    },
                },
            ];

            assert_eq!(
                parse(&input),
                Err(ErrorInfoWithPosition {
                    error: ErrorInfo {
                        code: ErrorCode::Ext2,
                        additional_info: None,
                    },
                    position: Position {
                        line_number: 1,
                        column_number: 2,
                        length: 1,
                    },
                })
            );
        }
    }
}
