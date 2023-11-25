use crate::error_code::ErrorInfoWithPosition;
use crate::error_code::{ErrorCode, ErrorInfo};
use crate::util::position::Position;

pub mod types;
pub mod util;

use types::token::Token;
use types::value_token::ValueToken;
use util::is_token_char;
use util::next_char_with_position;

use self::types::token_with_position::TokenWithPosition;

pub fn tokenize(input: &str) -> Result<Vec<TokenWithPosition>, ErrorInfoWithPosition> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    let mut origin_line_number = 1;
    let mut origin_column_number = 1;

    // while let Some(ch) = chars.next() {
    while let Some((ch, pos)) = next_char_with_position(
        &mut chars,
        &mut origin_line_number,
        &mut origin_column_number,
    ) {
        match ch {
            '@' => tokens.push(TokenWithPosition {
                token: Token::SectionMetaInfoStart,
                position: Position {
                    line_number: pos.line_number,
                    column_number: pos.column_number,
                    length: 1,
                },
            }),
            '[' => tokens.push(TokenWithPosition {
                token: Token::MetaInfoStart,
                position: Position {
                    line_number: pos.line_number,
                    column_number: pos.column_number,
                    length: 1,
                },
            }),
            ']' => tokens.push(TokenWithPosition {
                token: Token::MetaInfoEnd,
                position: Position {
                    line_number: pos.line_number,
                    column_number: pos.column_number,
                    length: 1,
                },
            }),
            '(' => tokens.push(TokenWithPosition {
                token: Token::ExtensionStart,
                position: Position {
                    line_number: pos.line_number,
                    column_number: pos.column_number,
                    length: 1,
                },
            }),
            ')' => tokens.push(TokenWithPosition {
                token: Token::ExtensionEnd,
                position: Position {
                    line_number: pos.line_number,
                    column_number: pos.column_number,
                    length: 1,
                },
            }),
            '-' => tokens.push(TokenWithPosition {
                token: Token::ChordBlockSeparator,
                position: Position {
                    line_number: pos.line_number,
                    column_number: pos.column_number,
                    length: 1,
                },
            }),
            '=' => tokens.push(TokenWithPosition {
                token: Token::Equal,
                position: Position {
                    line_number: pos.line_number,
                    column_number: pos.column_number,
                    length: 1,
                },
            }),
            ',' => tokens.push(TokenWithPosition {
                token: Token::Comma,
                position: Position {
                    line_number: pos.line_number,
                    column_number: pos.column_number,
                    length: 1,
                },
            }),
            '/' => tokens.push(TokenWithPosition {
                token: Token::Slash,
                position: Position {
                    line_number: pos.line_number,
                    column_number: pos.column_number,
                    length: 1,
                },
            }),
            ' ' | '　' | '\t' => {}
            '\n' | '\r' => {
                // first line line-break
                if tokens.is_empty() {
                    tokens.push(TokenWithPosition {
                        token: Token::LineBreak,
                        position: Position {
                            line_number: pos.line_number,
                            column_number: pos.column_number,
                            length: 1,
                        },
                    });
                    continue;
                }

                // validations for line break
                match tokens.last().unwrap().token {
                    Token::SectionMetaInfoKey(_) => {
                        return Err(ErrorInfoWithPosition {
                            error: ErrorInfo {
                                code: ErrorCode::Smik2,
                                additional_info: None,
                            },
                            position: Position {
                                line_number: pos.line_number,
                                column_number: pos.column_number,
                                length: 1,
                            },
                        });
                    }
                    Token::MetaInfoKey(_) => {
                        return Err(ErrorInfoWithPosition {
                            error: ErrorInfo {
                                code: ErrorCode::Cimk1,
                                additional_info: None,
                            },
                            position: Position {
                                line_number: pos.line_number,
                                column_number: pos.column_number,
                                length: 1,
                            },
                        });
                    }
                    Token::MetaInfoValue(_) => {
                        return Err(ErrorInfoWithPosition {
                            error: ErrorInfo {
                                code: ErrorCode::Cimv1,
                                additional_info: None,
                            },
                            position: Position {
                                line_number: pos.line_number,
                                column_number: pos.column_number,
                                length: 1,
                            },
                        });
                    }
                    Token::Comma => {
                        return Err(ErrorInfoWithPosition {
                            error: ErrorInfo {
                                code: ErrorCode::Chb2,
                                additional_info: None,
                            },
                            position: Position {
                                line_number: pos.line_number,
                                column_number: pos.column_number,
                                length: 1,
                            },
                        });
                    }
                    _ => tokens.push(TokenWithPosition {
                        token: Token::LineBreak,
                        position: Position {
                            line_number: pos.line_number,
                            column_number: pos.column_number,
                            length: 1,
                        },
                    }),
                }
            }
            non_functional_char => {
                let mut token = String::new();
                token.push(non_functional_char);

                let get_token_type_result = match tokens.last() {
                    None => Ok(Some(ValueToken::Chord)),
                    Some(token_with_position) => match token_with_position.token {
                        Token::SectionMetaInfoStart => Ok(Some(ValueToken::SectionMetaInfoKey)),
                        Token::ExtensionStart => Ok(Some(ValueToken::Extension)),
                        Token::MetaInfoStart => Ok(Some(ValueToken::MetaInfoKey)),
                        Token::Equal => {
                            // NOTE: In the case of equal, depending on the character before equal,
                            //       it may be a section meta or a code meta, so get it in advance.
                            let token_before_equal = if tokens.len() >= 2 {
                                tokens.get(tokens.len() - 2)
                            } else {
                                None
                            };

                            if token_before_equal.is_none() {
                                return Err(ErrorInfoWithPosition {
                                    error: ErrorInfo {
                                        code: ErrorCode::Tkn1,
                                        additional_info: None,
                                    },
                                    position: Position {
                                        line_number: pos.line_number,
                                        column_number: pos.column_number,
                                        length: 1,
                                    },
                                });
                            }

                            match token_before_equal.unwrap().token {
                                Token::SectionMetaInfoKey(_) => {
                                    Ok(Some(ValueToken::SectionMetaInfoValue))
                                }
                                Token::MetaInfoKey(_) => Ok(Some(ValueToken::MetaInfoValue)),
                                _ => {
                                    let position = pos.clone();
                                    return Err(ErrorInfoWithPosition {
                                        error: ErrorInfo {
                                            code: ErrorCode::Tkn1,
                                            additional_info: Some(
                                                token_before_equal.unwrap().token.to_string(),
                                            ),
                                        },
                                        position: Position {
                                            line_number: position.line_number,
                                            column_number: position.column_number - 1,
                                            length: token_before_equal
                                                .unwrap()
                                                .token
                                                .to_string()
                                                .len(),
                                        },
                                    });
                                }
                            }
                        }
                        Token::Slash => Ok(Some(ValueToken::Denominator)),
                        _ => {
                            // NOTE:
                            //   If the result of tracing back is "]" or "-", it is a code,
                            //   and if it is "(", it is an Extension.
                            let mut is_code = false;
                            let mut is_extension = false;
                            let mut prev_token_index = tokens.len();

                            // get previous token type
                            // To take into account line feed codes, use while
                            while prev_token_index > 0 {
                                prev_token_index -= 1;
                                let prev_token = tokens.get(prev_token_index).unwrap();
                                match prev_token.token {
                                    Token::MetaInfoEnd => {
                                        is_code = true;
                                        break;
                                    }
                                    Token::ChordBlockSeparator => {
                                        is_code = true;
                                        break;
                                    }
                                    Token::ExtensionStart => {
                                        is_extension = true;
                                        break;
                                    }
                                    Token::ExtensionEnd => {
                                        is_code = true;
                                        break;
                                    }
                                    Token::LineBreak => {
                                        is_code = true;
                                        break;
                                    }
                                    _ => {}
                                };
                            }

                            if is_code {
                                Ok(Some(ValueToken::Chord))
                            } else if is_extension {
                                Ok(Some(ValueToken::Extension))
                            } else {
                                println!("111 prev_token: {:?}", tokens.get(prev_token_index));
                                Err(ErrorInfoWithPosition {
                                    error: ErrorInfo {
                                        code: ErrorCode::Tkn1,
                                        additional_info: None,
                                    },
                                    position: Position {
                                        line_number: pos.line_number,
                                        column_number: pos.column_number,
                                        length: 1,
                                    },
                                })
                            }
                        }
                    },
                };

                // if error, return
                let token_type = if let Ok(token_type) = get_token_type_result {
                    token_type
                } else {
                    return Err(get_token_type_result.unwrap_err());
                };

                // get token
                while let Some(&next_ch) = chars.peek() {
                    // loop while next char is token char
                    if is_token_char(next_ch) {
                        break;
                    }

                    // if white space, break
                    if next_ch == ' ' || next_ch == '　' || next_ch == '\t' {
                        break;
                    }

                    token.push(next_ch);
                    next_char_with_position(
                        &mut chars,
                        &mut origin_line_number,
                        &mut origin_column_number,
                    );
                }

                // push token
                let borrowed_token = token.clone();
                match token_type {
                    Some(ValueToken::SectionMetaInfoKey) => tokens.push(TokenWithPosition {
                        token: Token::SectionMetaInfoKey(token),
                        position: Position {
                            line_number: pos.line_number,
                            column_number: pos.column_number,
                            length: borrowed_token.len(),
                        },
                    }),
                    Some(ValueToken::SectionMetaInfoValue) => tokens.push(TokenWithPosition {
                        token: Token::SectionMetaInfoValue(token),
                        position: Position {
                            line_number: pos.line_number,
                            column_number: pos.column_number,
                            length: borrowed_token.len(),
                        },
                    }),
                    Some(ValueToken::Extension) => tokens.push(TokenWithPosition {
                        token: Token::Extension(token),
                        position: Position {
                            line_number: pos.line_number,
                            column_number: pos.column_number,
                            length: borrowed_token.len(),
                        },
                    }),
                    Some(ValueToken::MetaInfoKey) => tokens.push(TokenWithPosition {
                        token: Token::MetaInfoKey(token),
                        position: Position {
                            line_number: pos.line_number,
                            column_number: pos.column_number,
                            length: borrowed_token.len(),
                        },
                    }),
                    Some(ValueToken::MetaInfoValue) => tokens.push(TokenWithPosition {
                        token: Token::MetaInfoValue(token),
                        position: Position {
                            line_number: pos.line_number,
                            column_number: pos.column_number,
                            length: borrowed_token.len(),
                        },
                    }),
                    Some(ValueToken::Chord) => {
                        // If the chord is invalid (contains some number or o), an error occurs.
                        if token.chars().any(|c| c.is_numeric() || c == 'o') {
                            return Err(ErrorInfoWithPosition {
                                error: ErrorInfo {
                                    code: ErrorCode::Cho1,
                                    additional_info: Some(token),
                                },
                                position: Position {
                                    line_number: pos.line_number,
                                    column_number: pos.column_number,
                                    length: borrowed_token.len(),
                                },
                            });
                        }
                        tokens.push(TokenWithPosition {
                            token: Token::Chord(token),
                            position: Position {
                                line_number: pos.line_number,
                                column_number: pos.column_number,
                                length: borrowed_token.len(),
                            },
                        })
                    }
                    Some(ValueToken::Denominator) => tokens.push(TokenWithPosition {
                        token: Token::Denominator(token),
                        position: Position {
                            line_number: pos.line_number,
                            column_number: pos.column_number,
                            length: borrowed_token.len(),
                        },
                    }),
                    None => {
                        return Err(ErrorInfoWithPosition {
                            error: ErrorInfo {
                                code: ErrorCode::Tkn1,
                                additional_info: Some(token_type.unwrap().to_string()),
                            },
                            position: Position {
                                line_number: pos.line_number,
                                column_number: pos.column_number,
                                length: borrowed_token.len(),
                            },
                        });
                    }
                }
            }
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod success {
        use crate::util::position::Position;

        use super::*;

        #[test]
        fn chord_can_surrounded_by_white_space() {
            let input = "C -   C -C(7) -C";
            let expected = vec![
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
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
                        column_number: 3,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 7,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 1,
                        column_number: 9,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 10,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionStart,
                    position: Position {
                        line_number: 1,
                        column_number: 11,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Extension("7".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 12,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionEnd,
                    position: Position {
                        line_number: 1,
                        column_number: 13,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 1,
                        column_number: 15,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 16,
                        length: 1,
                    },
                },
            ];

            let lex_result = tokenize(input);
            assert_eq!(lex_result.unwrap(), expected);
        }

        #[test]
        fn without_any_line_break() {
            let input = "C";
            let expected = vec![TokenWithPosition {
                token: Token::Chord("C".to_string()),
                position: Position {
                    line_number: 1,
                    column_number: 1,
                    length: 1,
                },
            }];

            let lex_result = tokenize(input);
            assert_eq!(lex_result.unwrap(), expected);
        }

        #[test]
        fn chord_after_extension_and_comma() {
            let input = "C(9),C";
            let expected = vec![
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
                    token: Token::Comma,
                    position: Position {
                        line_number: 1,
                        column_number: 5,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 6,
                        length: 1,
                    },
                },
            ];

            let lex_result = tokenize(input);
            assert_eq!(lex_result.unwrap(), expected);
        }

        #[test]
        fn section_meta_info() {
            let input = "@section=A";
            let expected = vec![
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
            ];

            let lex_result = tokenize(input);
            assert!(lex_result.is_ok());
            let tokens = lex_result.unwrap();
            assert_eq!(tokens, expected);
        }

        #[test]
        fn multiple_section_meta_info() {
            let input = "
@section=A
@sample=aaa
";
            let expected = vec![
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
                        length: 7,
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
                    token: Token::SectionMetaInfoKey("sample".to_string()),
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
                        column_number: 8,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoValue("aaa".to_string()),
                    position: Position {
                        line_number: 3,
                        column_number: 9,
                        length: 3,
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

            let lex_result = tokenize(input);
            assert!(lex_result.is_ok());
            let tokens = lex_result.unwrap();
            assert_eq!(tokens, expected);
        }

        #[test]
        fn chord_block() {
            let input = "C-F-Fm-C";

            let expected = vec![
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
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
                    token: Token::Chord("F".to_string()),
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
                TokenWithPosition {
                    token: Token::Chord("Fm".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 5,
                        length: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 1,
                        column_number: 7,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 8,
                        length: 1,
                    },
                },
            ];

            let lex_result = tokenize(input);
            assert_eq!(lex_result.unwrap(), expected);
        }

        #[test]
        fn chord_block_with_fraction_chord() {
            let input = "
C-G/Bb-Am-Em/G
F#m(7,b5)/F#m(7,b5)-Fbm(13)/G(7)
";

            let expected = vec![
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
                        column_number: 7,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("Am".to_string()),
                    position: Position {
                        line_number: 2,
                        column_number: 8,
                        length: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 2,
                        column_number: 10,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("Em".to_string()),
                    position: Position {
                        line_number: 2,
                        column_number: 11,
                        length: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::Slash,
                    position: Position {
                        line_number: 2,
                        column_number: 13,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Denominator("G".to_string()),
                    position: Position {
                        line_number: 2,
                        column_number: 14,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 2,
                        column_number: 15,
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
                        column_number: 9,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Slash,
                    position: Position {
                        line_number: 3,
                        column_number: 10,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Denominator("F#m".to_string()),
                    position: Position {
                        line_number: 3,
                        column_number: 11,
                        length: 3,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionStart,
                    position: Position {
                        line_number: 3,
                        column_number: 14,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Extension("7".to_string()),
                    position: Position {
                        line_number: 3,
                        column_number: 15,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Comma,
                    position: Position {
                        line_number: 3,
                        column_number: 16,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Extension("b5".to_string()),
                    position: Position {
                        line_number: 3,
                        column_number: 17,
                        length: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionEnd,
                    position: Position {
                        line_number: 3,
                        column_number: 19,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 3,
                        column_number: 20,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("Fbm".to_string()),
                    position: Position {
                        line_number: 3,
                        column_number: 21,
                        length: 3,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionStart,
                    position: Position {
                        line_number: 3,
                        column_number: 24,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Extension("13".to_string()),
                    position: Position {
                        line_number: 3,
                        column_number: 25,
                        length: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionEnd,
                    position: Position {
                        line_number: 3,
                        column_number: 27,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Slash,
                    position: Position {
                        line_number: 3,
                        column_number: 28,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Denominator("G".to_string()),
                    position: Position {
                        line_number: 3,
                        column_number: 29,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionStart,
                    position: Position {
                        line_number: 3,
                        column_number: 30,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Extension("7".to_string()),
                    position: Position {
                        line_number: 3,
                        column_number: 31,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionEnd,
                    position: Position {
                        line_number: 3,
                        column_number: 32,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 3,
                        column_number: 33,
                        length: 1,
                    },
                },
            ];

            let lex_result = tokenize(input);
            assert_eq!(lex_result.unwrap(), expected);
        }

        #[test]
        fn chord_block_with_multiple_meta_info() {
            let input = "[key=C][sample=aaa]C-F-[key=Eb]Fm,[sample=bbb]Bb-C";

            let expected = vec![
                TokenWithPosition {
                    token: Token::MetaInfoStart,
                    position: Position {
                        line_number: 1,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoKey("key".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 2,
                        length: 3,
                    },
                },
                TokenWithPosition {
                    token: Token::Equal,
                    position: Position {
                        line_number: 1,
                        column_number: 5,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoValue("C".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 6,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoEnd,
                    position: Position {
                        line_number: 1,
                        column_number: 7,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoStart,
                    position: Position {
                        line_number: 1,
                        column_number: 8,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoKey("sample".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 9,
                        length: 6,
                    },
                },
                TokenWithPosition {
                    token: Token::Equal,
                    position: Position {
                        line_number: 1,
                        column_number: 15,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoValue("aaa".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 16,
                        length: 3,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoEnd,
                    position: Position {
                        line_number: 1,
                        column_number: 19,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 20,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 1,
                        column_number: 21,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("F".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 22,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 1,
                        column_number: 23,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoStart,
                    position: Position {
                        line_number: 1,
                        column_number: 24,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoKey("key".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 25,
                        length: 3,
                    },
                },
                TokenWithPosition {
                    token: Token::Equal,
                    position: Position {
                        line_number: 1,
                        column_number: 28,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoValue("Eb".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 29,
                        length: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoEnd,
                    position: Position {
                        line_number: 1,
                        column_number: 31,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("Fm".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 32,
                        length: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::Comma,
                    position: Position {
                        line_number: 1,
                        column_number: 34,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoStart,
                    position: Position {
                        line_number: 1,
                        column_number: 35,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoKey("sample".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 36,
                        length: 6,
                    },
                },
                TokenWithPosition {
                    token: Token::Equal,
                    position: Position {
                        line_number: 1,
                        column_number: 42,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoValue("bbb".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 43,
                        length: 3,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoEnd,
                    position: Position {
                        line_number: 1,
                        column_number: 46,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("Bb".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 47,
                        length: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 1,
                        column_number: 49,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 50,
                        length: 1,
                    },
                },
            ];

            let lex_result = tokenize(input);
            assert_eq!(lex_result.unwrap(), expected);
        }

        #[test]
        fn complicated() {
            let input = "
@section=A
@sample=aaa
C-C(7)-F-Fm(7)
C

@section=B
[key=F]Gm-Gm-F-F
Gm-Gm
";

            let expected = vec![
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
                        length: 7,
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
                    token: Token::SectionMetaInfoKey("sample".to_string()),
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
                        column_number: 8,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoValue("aaa".to_string()),
                    position: Position {
                        line_number: 3,
                        column_number: 9,
                        length: 3,
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
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 4,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 4,
                        column_number: 2,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 4,
                        column_number: 3,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionStart,
                    position: Position {
                        line_number: 4,
                        column_number: 4,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Extension("7".to_string()),
                    position: Position {
                        line_number: 4,
                        column_number: 5,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionEnd,
                    position: Position {
                        line_number: 4,
                        column_number: 6,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 4,
                        column_number: 7,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("F".to_string()),
                    position: Position {
                        line_number: 4,
                        column_number: 8,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 4,
                        column_number: 9,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("Fm".to_string()),
                    position: Position {
                        line_number: 4,
                        column_number: 10,
                        length: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionStart,
                    position: Position {
                        line_number: 4,
                        column_number: 12,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Extension("7".to_string()),
                    position: Position {
                        line_number: 4,
                        column_number: 13,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionEnd,
                    position: Position {
                        line_number: 4,
                        column_number: 14,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 4,
                        column_number: 15,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 5,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 5,
                        column_number: 2,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 6,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoStart,
                    position: Position {
                        line_number: 7,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoKey("section".to_string()),
                    position: Position {
                        line_number: 7,
                        column_number: 2,
                        length: 7,
                    },
                },
                TokenWithPosition {
                    token: Token::Equal,
                    position: Position {
                        line_number: 7,
                        column_number: 9,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoValue("B".to_string()),
                    position: Position {
                        line_number: 7,
                        column_number: 10,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 7,
                        column_number: 11,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoStart,
                    position: Position {
                        line_number: 8,
                        column_number: 1,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoKey("key".to_string()),
                    position: Position {
                        line_number: 8,
                        column_number: 2,
                        length: 3,
                    },
                },
                TokenWithPosition {
                    token: Token::Equal,
                    position: Position {
                        line_number: 8,
                        column_number: 5,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoValue("F".to_string()),
                    position: Position {
                        line_number: 8,
                        column_number: 6,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoEnd,
                    position: Position {
                        line_number: 8,
                        column_number: 7,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("Gm".to_string()),
                    position: Position {
                        line_number: 8,
                        column_number: 8,
                        length: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 8,
                        column_number: 10,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("Gm".to_string()),
                    position: Position {
                        line_number: 8,
                        column_number: 11,
                        length: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 8,
                        column_number: 13,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("F".to_string()),
                    position: Position {
                        line_number: 8,
                        column_number: 14,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 8,
                        column_number: 15,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("F".to_string()),
                    position: Position {
                        line_number: 8,
                        column_number: 16,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 8,
                        column_number: 17,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("Gm".to_string()),
                    position: Position {
                        line_number: 9,
                        column_number: 1,

                        length: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 9,
                        column_number: 3,
                        length: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("Gm".to_string()),
                    position: Position {
                        line_number: 9,
                        column_number: 4,
                        length: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 9,
                        column_number: 6,
                        length: 1,
                    },
                },
            ];

            let lex_result = tokenize(input);
            assert_eq!(lex_result.unwrap(), expected);
        }
    }

    #[cfg(test)]
    mod failed {
        use super::*;

        // #[test]
        // fn only_non_meaning_char() {
        //     let non_meaning_chars = vec![
        //         'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', '!', '$', ':', ';',
        //     ];

        //     for c in non_meaning_chars {
        //         let input = format!("{}", c);
        //         let lex_result = tokenize(&input);
        //         assert_eq!(lex_result.unwrap_err().error.code, ErrorCode::Tkn1);
        //     }
        // }

        #[test]
        fn invalid_chord_o() {
            let input = "|o|";
            let lex_result = tokenize(input);
            assert!(lex_result.is_err());
            assert_eq!(lex_result.unwrap_err().error.code, ErrorCode::Cho1);
        }

        #[test]
        fn section_meta_info_key_should_not_contains_line_break() {
            let input = "
            @sect
            ion=A
            ";

            let lex_result = tokenize(input);
            assert!(lex_result.is_err());
            assert_eq!(lex_result.unwrap_err().error.code, ErrorCode::Smik2);
        }

        #[test]
        fn meta_info_key_should_not_contains_line_break() {
            let input = "
                [aaaa
                    aaaa=bbbb
                ]C
                ";

            assert_eq!(tokenize(input).unwrap_err().error.code, ErrorCode::Cimk1);
        }

        #[test]
        fn meta_info_value_should_not_contains_line_break() {
            let input = "
                [aaaa=bbbb
                    bbbb]C
                ";

            assert_eq!(tokenize(input).unwrap_err().error.code, ErrorCode::Cimv1);
        }

        #[test]
        fn chord_block_should_not_contains_line_break() {
            let input = "
                |C,
                C7|F|Fm(7)|
                ";

            let lex_result = tokenize(input);
            println!("111111 {:?}", lex_result);
            assert!(lex_result.is_err());
            assert_eq!(lex_result.unwrap_err().error.code, ErrorCode::Chb2,);
        }
    }
}
