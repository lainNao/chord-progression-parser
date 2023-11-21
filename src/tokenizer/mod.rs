use crate::error_code::ErrorInfoWithPosition;
use crate::error_code::{ErrorCode, ErrorInfo};

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
                position: pos.clone(),
            }),
            '[' => tokens.push(TokenWithPosition {
                token: Token::MetaInfoStart,
                position: pos.clone(),
            }),
            ']' => tokens.push(TokenWithPosition {
                token: Token::MetaInfoEnd,
                position: pos.clone(),
            }),
            '(' => tokens.push(TokenWithPosition {
                token: Token::ExtensionStart,
                position: pos.clone(),
            }),
            ')' => tokens.push(TokenWithPosition {
                token: Token::ExtensionEnd,
                position: pos.clone(),
            }),
            '|' => tokens.push(TokenWithPosition {
                token: Token::ChordBlockSeparator,
                position: pos.clone(),
            }),
            '=' => tokens.push(TokenWithPosition {
                token: Token::Equal,
                position: pos.clone(),
            }),
            ',' => tokens.push(TokenWithPosition {
                token: Token::Comma,
                position: pos.clone(),
            }),
            '/' => tokens.push(TokenWithPosition {
                token: Token::Slash,
                position: pos.clone(),
            }),
            ' ' | '　' | '\t' => {}
            '\n' | '\r' => {
                if tokens.is_empty() {
                    tokens.push(TokenWithPosition {
                        token: Token::LineBreak,
                        position: pos.clone(),
                    });
                    continue;
                }

                match tokens.last().unwrap().token {
                    Token::SectionMetaInfoKey(_) => {
                        return Err(ErrorInfoWithPosition {
                            error: ErrorInfo {
                                code: ErrorCode::Smik2,
                                additional_info: None,
                            },
                            position: pos.clone(),
                        });
                    }
                    Token::MetaInfoKey(_) => {
                        return Err(ErrorInfoWithPosition {
                            error: ErrorInfo {
                                code: ErrorCode::Cimk1,
                                additional_info: None,
                            },
                            position: pos.clone(),
                        });
                    }
                    Token::MetaInfoValue(_) => {
                        return Err(ErrorInfoWithPosition {
                            error: ErrorInfo {
                                code: ErrorCode::Cimv1,
                                additional_info: None,
                            },
                            position: pos.clone(),
                        });
                    }
                    Token::Comma => {
                        return Err(ErrorInfoWithPosition {
                            error: ErrorInfo {
                                code: ErrorCode::Chb2,
                                additional_info: None,
                            },
                            position: pos.clone(),
                        });
                    }
                    _ => tokens.push(TokenWithPosition {
                        token: Token::LineBreak,
                        position: pos.clone(),
                    }),
                }
            }
            non_functional_char => {
                let mut token = String::new();
                token.push(non_functional_char);

                // get token type by using previous token
                let get_token_type_result = match tokens.last().unwrap().token {
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
                                position: pos.clone(),
                            });
                        }

                        match token_before_equal.unwrap().token {
                            Token::SectionMetaInfoKey(_) => {
                                Ok(Some(ValueToken::SectionMetaInfoValue))
                            }
                            Token::MetaInfoKey(_) => Ok(Some(ValueToken::MetaInfoValue)),
                            _ => {
                                return Err(ErrorInfoWithPosition {
                                    error: ErrorInfo {
                                        code: ErrorCode::Tkn1,
                                        additional_info: Some(
                                            token_before_equal.unwrap().token.to_string(),
                                        ),
                                    },
                                    position: pos.clone(),
                                });
                            }
                        }
                    }
                    Token::Slash => Ok(Some(ValueToken::Denominator)),
                    _ => {
                        // If the result of tracing back is "]" or "|", it is a code, and if it is "(", it is an Extension.
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
                                _ => {}
                            };
                        }

                        if is_code {
                            Ok(Some(ValueToken::Chord))
                        } else if is_extension {
                            Ok(Some(ValueToken::Extension))
                        } else {
                            Err(ErrorInfoWithPosition {
                                error: ErrorInfo {
                                    code: ErrorCode::Tkn1,
                                    additional_info: None,
                                },
                                position: pos.clone(),
                            })
                        }
                    }
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

                    token.push(next_ch);
                    next_char_with_position(
                        &mut chars,
                        &mut origin_line_number,
                        &mut origin_column_number,
                    );
                }

                // push token
                match token_type {
                    Some(ValueToken::SectionMetaInfoKey) => tokens.push(TokenWithPosition {
                        token: Token::SectionMetaInfoKey(token),
                        position: pos.clone(),
                    }),
                    Some(ValueToken::SectionMetaInfoValue) => tokens.push(TokenWithPosition {
                        token: Token::SectionMetaInfoValue(token),
                        position: pos.clone(),
                    }),
                    Some(ValueToken::Extension) => tokens.push(TokenWithPosition {
                        token: Token::Extension(token),
                        position: pos.clone(),
                    }),
                    Some(ValueToken::MetaInfoKey) => tokens.push(TokenWithPosition {
                        token: Token::MetaInfoKey(token),
                        position: pos.clone(),
                    }),
                    Some(ValueToken::MetaInfoValue) => tokens.push(TokenWithPosition {
                        token: Token::MetaInfoValue(token),
                        position: pos.clone(),
                    }),
                    Some(ValueToken::Chord) => {
                        // If the chord is invalid (contains some number or o), an error occurs.
                        if token.chars().any(|c| c.is_numeric() || c == 'o') {
                            return Err(ErrorInfoWithPosition {
                                error: ErrorInfo {
                                    code: ErrorCode::Cho1,
                                    additional_info: Some(token),
                                },
                                position: pos.clone(),
                            });
                        }
                        tokens.push(TokenWithPosition {
                            token: Token::Chord(token),
                            position: pos.clone(),
                        })
                    }
                    Some(ValueToken::Denominator) => tokens.push(TokenWithPosition {
                        token: Token::Denominator(token),
                        position: pos.clone(),
                    }),
                    None => {
                        return Err(ErrorInfoWithPosition {
                            error: ErrorInfo {
                                code: ErrorCode::Tkn1,
                                additional_info: Some(token_type.unwrap().to_string()),
                            },
                            position: pos.clone(),
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
        fn without_any_line_break() {
            let input = "|C|";
            let expected = vec![
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 1,
                        column_number: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 1,
                        column_number: 3,
                    },
                },
            ];

            let lex_result = tokenize(input);
            assert!(lex_result.is_ok());
            let tokens = lex_result.unwrap();
            assert_eq!(tokens, expected);
        }

        #[test]
        fn chord_after_extension_and_comma() {
            let input = "|C(9),C|";
            let expected = vec![
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 1,
                        column_number: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionStart,
                    position: Position {
                        line_number: 1,
                        column_number: 3,
                    },
                },
                TokenWithPosition {
                    token: Token::Extension("9".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 4,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionEnd,
                    position: Position {
                        line_number: 1,
                        column_number: 5,
                    },
                },
                TokenWithPosition {
                    token: Token::Comma,
                    position: Position {
                        line_number: 1,
                        column_number: 6,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 7,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 1,
                        column_number: 8,
                    },
                },
            ];

            let lex_result = tokenize(input);
            assert!(lex_result.is_ok());
            let tokens = lex_result.unwrap();
            assert_eq!(tokens, expected);
        }

        #[test]
        fn section_meta_info3() {
            let input = "@section=A";
            let expected = vec![
                TokenWithPosition {
                    token: Token::SectionMetaInfoStart,
                    position: Position {
                        line_number: 1,
                        column_number: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoKey("section".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::Equal,
                    position: Position {
                        line_number: 1,
                        column_number: 9,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoValue("A".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 10,
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
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoStart,
                    position: Position {
                        line_number: 2,
                        column_number: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoKey("section".to_string()),
                    position: Position {
                        line_number: 2,
                        column_number: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::Equal,
                    position: Position {
                        line_number: 2,
                        column_number: 9,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoValue("A".to_string()),
                    position: Position {
                        line_number: 2,
                        column_number: 10,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 2,
                        column_number: 11,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoStart,
                    position: Position {
                        line_number: 3,
                        column_number: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoKey("sample".to_string()),
                    position: Position {
                        line_number: 3,
                        column_number: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::Equal,
                    position: Position {
                        line_number: 3,
                        column_number: 8,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoValue("aaa".to_string()),
                    position: Position {
                        line_number: 3,
                        column_number: 9,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 3,
                        column_number: 12,
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
            let input = "
|C|F|Fm|C|
";

            let expected = vec![
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 1,
                        column_number: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 2,
                        column_number: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 2,
                        column_number: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 2,
                        column_number: 3,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("F".to_string()),
                    position: Position {
                        line_number: 2,
                        column_number: 4,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 2,
                        column_number: 5,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("Fm".to_string()),
                    position: Position {
                        line_number: 2,
                        column_number: 6,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 2,
                        column_number: 8,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 2,
                        column_number: 9,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 2,
                        column_number: 10,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 2,
                        column_number: 11,
                    },
                },
            ];

            let lex_result = tokenize(input);
            assert!(lex_result.is_ok());
            let tokens = lex_result.unwrap();
            assert_eq!(tokens, expected);
        }

        #[test]
        fn chord_block_with_fraction_chord() {
            let input = "
|C|G/Bb|Am|Em/G|
|F#m(7,b5)/F#m(7,b5)|Fbm(13)/G(7)|
";

            let expected = vec![
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 1,
                        column_number: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 2,
                        column_number: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 2,
                        column_number: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 2,
                        column_number: 3,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("G".to_string()),
                    position: Position {
                        line_number: 2,
                        column_number: 4,
                    },
                },
                TokenWithPosition {
                    token: Token::Slash,
                    position: Position {
                        line_number: 2,
                        column_number: 5,
                    },
                },
                TokenWithPosition {
                    token: Token::Denominator("Bb".to_string()),
                    position: Position {
                        line_number: 2,
                        column_number: 6,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 2,
                        column_number: 8,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("Am".to_string()),
                    position: Position {
                        line_number: 2,
                        column_number: 9,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 2,
                        column_number: 11,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("Em".to_string()),
                    position: Position {
                        line_number: 2,
                        column_number: 12,
                    },
                },
                TokenWithPosition {
                    token: Token::Slash,
                    position: Position {
                        line_number: 2,
                        column_number: 14,
                    },
                },
                TokenWithPosition {
                    token: Token::Denominator("G".to_string()),
                    position: Position {
                        line_number: 2,
                        column_number: 15,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 2,
                        column_number: 16,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 2,
                        column_number: 17,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 3,
                        column_number: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("F#m".to_string()),
                    position: Position {
                        line_number: 3,
                        column_number: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionStart,
                    position: Position {
                        line_number: 3,
                        column_number: 5,
                    },
                },
                TokenWithPosition {
                    token: Token::Extension("7".to_string()),
                    position: Position {
                        line_number: 3,
                        column_number: 6,
                    },
                },
                TokenWithPosition {
                    token: Token::Comma,
                    position: Position {
                        line_number: 3,
                        column_number: 7,
                    },
                },
                TokenWithPosition {
                    token: Token::Extension("b5".to_string()),
                    position: Position {
                        line_number: 3,
                        column_number: 8,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionEnd,
                    position: Position {
                        line_number: 3,
                        column_number: 10,
                    },
                },
                TokenWithPosition {
                    token: Token::Slash,
                    position: Position {
                        line_number: 3,
                        column_number: 11,
                    },
                },
                TokenWithPosition {
                    token: Token::Denominator("F#m".to_string()),
                    position: Position {
                        line_number: 3,
                        column_number: 12,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionStart,
                    position: Position {
                        line_number: 3,
                        column_number: 15,
                    },
                },
                TokenWithPosition {
                    token: Token::Extension("7".to_string()),
                    position: Position {
                        line_number: 3,
                        column_number: 16,
                    },
                },
                TokenWithPosition {
                    token: Token::Comma,
                    position: Position {
                        line_number: 3,
                        column_number: 17,
                    },
                },
                TokenWithPosition {
                    token: Token::Extension("b5".to_string()),
                    position: Position {
                        line_number: 3,
                        column_number: 18,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionEnd,
                    position: Position {
                        line_number: 3,
                        column_number: 20,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 3,
                        column_number: 21,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("Fbm".to_string()),
                    position: Position {
                        line_number: 3,
                        column_number: 22,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionStart,
                    position: Position {
                        line_number: 3,
                        column_number: 25,
                    },
                },
                TokenWithPosition {
                    token: Token::Extension("13".to_string()),
                    position: Position {
                        line_number: 3,
                        column_number: 26,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionEnd,
                    position: Position {
                        line_number: 3,
                        column_number: 28,
                    },
                },
                TokenWithPosition {
                    token: Token::Slash,
                    position: Position {
                        line_number: 3,
                        column_number: 29,
                    },
                },
                TokenWithPosition {
                    token: Token::Denominator("G".to_string()),
                    position: Position {
                        line_number: 3,
                        column_number: 30,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionStart,
                    position: Position {
                        line_number: 3,
                        column_number: 31,
                    },
                },
                TokenWithPosition {
                    token: Token::Extension("7".to_string()),
                    position: Position {
                        line_number: 3,
                        column_number: 32,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionEnd,
                    position: Position {
                        line_number: 3,
                        column_number: 33,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 3,
                        column_number: 34,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 3,
                        column_number: 35,
                    },
                },
            ];

            let lex_result = tokenize(input);
            assert!(lex_result.is_ok());
            let tokens = lex_result.unwrap();
            assert_eq!(tokens, expected);
        }

        #[test]
        fn chord_block_with_multiple_meta_info() {
            let input = "|[key=C][sample=aaa]C|F|[key=Eb]Fm,[sample=bbb]Bb|C|";

            let expected = vec![
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 1,
                        column_number: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoStart,
                    position: Position {
                        line_number: 1,
                        column_number: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoKey("key".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 3,
                    },
                },
                TokenWithPosition {
                    token: Token::Equal,
                    position: Position {
                        line_number: 1,
                        column_number: 6,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoValue("C".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 7,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoEnd,
                    position: Position {
                        line_number: 1,
                        column_number: 8,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoStart,
                    position: Position {
                        line_number: 1,
                        column_number: 9,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoKey("sample".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 10,
                    },
                },
                TokenWithPosition {
                    token: Token::Equal,
                    position: Position {
                        line_number: 1,
                        column_number: 16,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoValue("aaa".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 17,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoEnd,
                    position: Position {
                        line_number: 1,
                        column_number: 20,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 21,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 1,
                        column_number: 22,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("F".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 23,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 1,
                        column_number: 24,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoStart,
                    position: Position {
                        line_number: 1,
                        column_number: 25,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoKey("key".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 26,
                    },
                },
                TokenWithPosition {
                    token: Token::Equal,
                    position: Position {
                        line_number: 1,
                        column_number: 29,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoValue("Eb".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 30,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoEnd,
                    position: Position {
                        line_number: 1,
                        column_number: 32,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("Fm".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 33,
                    },
                },
                TokenWithPosition {
                    token: Token::Comma,
                    position: Position {
                        line_number: 1,
                        column_number: 35,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoStart,
                    position: Position {
                        line_number: 1,
                        column_number: 36,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoKey("sample".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 37,
                    },
                },
                TokenWithPosition {
                    token: Token::Equal,
                    position: Position {
                        line_number: 1,
                        column_number: 43,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoValue("bbb".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 44,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoEnd,
                    position: Position {
                        line_number: 1,
                        column_number: 47,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("Bb".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 48,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 1,
                        column_number: 50,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 1,
                        column_number: 51,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 1,
                        column_number: 52,
                    },
                },
            ];

            let lex_result = tokenize(input);
            assert!(lex_result.is_ok());
            let tokens = lex_result.unwrap();
            assert_eq!(tokens, expected);
        }

        #[test]
        fn complicated() {
            let input = "
@section=A
@sample=aaa
|C|C(7)|F|Fm(7)|
|C|C(7)|F|Fm(7)|

@section=B
|[key=F]Gm|Gm|F|F|
|Gm|Gm|F|F|
";

            let expected = vec![
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 1,
                        column_number: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoStart,
                    position: Position {
                        line_number: 2,
                        column_number: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoKey("section".to_string()),
                    position: Position {
                        line_number: 2,
                        column_number: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::Equal,
                    position: Position {
                        line_number: 2,
                        column_number: 9,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoValue("A".to_string()),
                    position: Position {
                        line_number: 2,
                        column_number: 10,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 2,
                        column_number: 11,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoStart,
                    position: Position {
                        line_number: 3,
                        column_number: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoKey("sample".to_string()),
                    position: Position {
                        line_number: 3,
                        column_number: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::Equal,
                    position: Position {
                        line_number: 3,
                        column_number: 8,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoValue("aaa".to_string()),
                    position: Position {
                        line_number: 3,
                        column_number: 9,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 3,
                        column_number: 12,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 4,
                        column_number: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 4,
                        column_number: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 4,
                        column_number: 3,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 4,
                        column_number: 4,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionStart,
                    position: Position {
                        line_number: 4,
                        column_number: 5,
                    },
                },
                TokenWithPosition {
                    token: Token::Extension("7".to_string()),
                    position: Position {
                        line_number: 4,
                        column_number: 6,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionEnd,
                    position: Position {
                        line_number: 4,
                        column_number: 7,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 4,
                        column_number: 8,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("F".to_string()),
                    position: Position {
                        line_number: 4,
                        column_number: 9,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 4,
                        column_number: 10,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("Fm".to_string()),
                    position: Position {
                        line_number: 4,
                        column_number: 11,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionStart,
                    position: Position {
                        line_number: 4,
                        column_number: 13,
                    },
                },
                TokenWithPosition {
                    token: Token::Extension("7".to_string()),
                    position: Position {
                        line_number: 4,
                        column_number: 14,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionEnd,
                    position: Position {
                        line_number: 4,
                        column_number: 15,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 4,
                        column_number: 16,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 4,
                        column_number: 17,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 5,
                        column_number: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 5,
                        column_number: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 5,
                        column_number: 3,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("C".to_string()),
                    position: Position {
                        line_number: 5,
                        column_number: 4,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionStart,
                    position: Position {
                        line_number: 5,
                        column_number: 5,
                    },
                },
                TokenWithPosition {
                    token: Token::Extension("7".to_string()),
                    position: Position {
                        line_number: 5,
                        column_number: 6,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionEnd,
                    position: Position {
                        line_number: 5,
                        column_number: 7,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 5,
                        column_number: 8,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("F".to_string()),
                    position: Position {
                        line_number: 5,
                        column_number: 9,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 5,
                        column_number: 10,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("Fm".to_string()),
                    position: Position {
                        line_number: 5,
                        column_number: 11,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionStart,
                    position: Position {
                        line_number: 5,
                        column_number: 13,
                    },
                },
                TokenWithPosition {
                    token: Token::Extension("7".to_string()),
                    position: Position {
                        line_number: 5,
                        column_number: 14,
                    },
                },
                TokenWithPosition {
                    token: Token::ExtensionEnd,
                    position: Position {
                        line_number: 5,
                        column_number: 15,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 5,
                        column_number: 16,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 5,
                        column_number: 17,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 6,
                        column_number: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoStart,
                    position: Position {
                        line_number: 7,
                        column_number: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoKey("section".to_string()),
                    position: Position {
                        line_number: 7,
                        column_number: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::Equal,
                    position: Position {
                        line_number: 7,
                        column_number: 9,
                    },
                },
                TokenWithPosition {
                    token: Token::SectionMetaInfoValue("B".to_string()),
                    position: Position {
                        line_number: 7,
                        column_number: 10,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 7,
                        column_number: 11,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 8,
                        column_number: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoStart,
                    position: Position {
                        line_number: 8,
                        column_number: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoKey("key".to_string()),
                    position: Position {
                        line_number: 8,
                        column_number: 3,
                    },
                },
                TokenWithPosition {
                    token: Token::Equal,
                    position: Position {
                        line_number: 8,
                        column_number: 6,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoValue("F".to_string()),
                    position: Position {
                        line_number: 8,
                        column_number: 7,
                    },
                },
                TokenWithPosition {
                    token: Token::MetaInfoEnd,
                    position: Position {
                        line_number: 8,
                        column_number: 8,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("Gm".to_string()),
                    position: Position {
                        line_number: 8,
                        column_number: 9,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 8,
                        column_number: 11,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("Gm".to_string()),
                    position: Position {
                        line_number: 8,
                        column_number: 12,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 8,
                        column_number: 14,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("F".to_string()),
                    position: Position {
                        line_number: 8,
                        column_number: 15,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 8,
                        column_number: 16,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("F".to_string()),
                    position: Position {
                        line_number: 8,
                        column_number: 17,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 8,
                        column_number: 18,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 8,
                        column_number: 19,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 9,
                        column_number: 1,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("Gm".to_string()),
                    position: Position {
                        line_number: 9,
                        column_number: 2,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 9,
                        column_number: 4,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("Gm".to_string()),
                    position: Position {
                        line_number: 9,
                        column_number: 5,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 9,
                        column_number: 7,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("F".to_string()),
                    position: Position {
                        line_number: 9,
                        column_number: 8,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 9,
                        column_number: 9,
                    },
                },
                TokenWithPosition {
                    token: Token::Chord("F".to_string()),
                    position: Position {
                        line_number: 9,
                        column_number: 10,
                    },
                },
                TokenWithPosition {
                    token: Token::ChordBlockSeparator,
                    position: Position {
                        line_number: 9,
                        column_number: 11,
                    },
                },
                TokenWithPosition {
                    token: Token::LineBreak,
                    position: Position {
                        line_number: 9,
                        column_number: 12,
                    },
                },
            ];

            let lex_result = tokenize(input);
            assert!(lex_result.is_ok());
            let tokens = lex_result.unwrap();
            assert_eq!(tokens, expected);
        }
    }

    #[cfg(test)]
    mod failed {
        use super::*;

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
                |[aaaa
                    aaaa=bbbb
                ]C|
                ";

            let lex_result = tokenize(input);
            assert!(lex_result.is_err());
            assert_eq!(lex_result.unwrap_err().error.code, ErrorCode::Cimk1);
        }

        #[test]
        fn meta_info_value_should_not_contains_line_break() {
            let input = "
                |[aaaa=bbbb
                    bbbb]C|
                ";

            let lex_result = tokenize(input);
            assert!(lex_result.is_err());
            assert_eq!(lex_result.unwrap_err().error.code, ErrorCode::Cimv1);
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
