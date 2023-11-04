use crate::errors;

pub mod types;
pub mod util;

use types::token::Token;
use types::value_token::ValueToken;
use util::is_token_char;

pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '@' => tokens.push(Token::SectionMetaInfoStart),
            '[' => tokens.push(Token::MetaInfoStart),
            ']' => tokens.push(Token::MetaInfoEnd),
            '(' => tokens.push(Token::ExtensionStart),
            ')' => tokens.push(Token::ExtensionEnd),
            '|' => tokens.push(Token::ChordBlockSeparator),
            '=' => tokens.push(Token::Equal),
            ',' => tokens.push(Token::Comma),
            '/' => tokens.push(Token::Slash),
            ' ' | '　' | '\t' => {}
            '\n' | '\r' => match tokens.last() {
                Some(Token::SectionMetaInfoKey(_)) => {
                    return Err(
                        errors::SECTION_META_INFO_KEY_SHOULD_NOT_CONTAINS_LINE_BREAK.to_string()
                    );
                }
                Some(Token::MetaInfoKey(_)) => {
                    return Err(errors::META_INFO_KEY_SHOULD_NOT_CONTAINS_LINE_BREAK.to_string());
                }
                Some(Token::MetaInfoValue(_)) => {
                    return Err(errors::META_INFO_VALUE_SHOULD_NOT_CONTAINS_LINE_BREAK.to_string());
                }
                Some(Token::Comma) => {
                    return Err(errors::CHORD_BLOCK_SHOULD_NOT_CONTAINS_LINE_BREAK.to_string());
                }
                _ => tokens.push(Token::LineBreak),
            },
            non_functional_char => {
                let mut token = String::new();
                token.push(non_functional_char);

                // get token type by using previous token
                let get_token_type_result = match tokens.last() {
                    Some(Token::SectionMetaInfoStart) => Ok(Some(ValueToken::SectionMetaInfoKey)),
                    Some(Token::ExtensionStart) => Ok(Some(ValueToken::Extension)),
                    Some(Token::MetaInfoStart) => Ok(Some(ValueToken::MetaInfoKey)),
                    Some(Token::Equal) => {
                        // NOTE: イコールの場合、イコールの前の文字によってはセクションメタなのかコードメタなのかが変わるので、取得しておく
                        let token_before_equal: Option<&Token> = if tokens.len() >= 2 {
                            tokens.get(tokens.len() - 2)
                        } else {
                            None
                        };

                        match token_before_equal {
                            Some(Token::SectionMetaInfoKey(_)) => {
                                Ok(Some(ValueToken::SectionMetaInfoValue))
                            }
                            Some(Token::MetaInfoKey(_)) => Ok(Some(ValueToken::MetaInfoValue)),
                            _ => {
                                return Err([
                                    errors::INVALID_TOKEN_TYPE.to_string(),
                                    token_before_equal.unwrap().to_string(),
                                ]
                                .join(": "));
                            }
                        }
                    }
                    Some(Token::Slash) => Ok(Some(ValueToken::Denominator)),
                    _ => {
                        // 前を辿った結果"]"か"|"があればコード、"("があればExtension
                        let mut is_code = false;
                        let mut is_extension = false;
                        let mut prev_token_index = tokens.len() - 1;
                        while prev_token_index > 0 {
                            let prev_token = tokens.get(prev_token_index).unwrap();
                            match prev_token {
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
                                _ => {}
                            };
                            prev_token_index -= 1;
                        }

                        if is_code {
                            Ok(Some(ValueToken::Chord))
                        } else if is_extension {
                            Ok(Some(ValueToken::Extension))
                        } else {
                            Err(errors::INVALID_TOKEN_TYPE.to_string())
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
                    chars.next();
                }

                // push token
                match token_type {
                    Some(ValueToken::SectionMetaInfoKey) => {
                        tokens.push(Token::SectionMetaInfoKey(token))
                    }
                    Some(ValueToken::SectionMetaInfoValue) => {
                        tokens.push(Token::SectionMetaInfoValue(token))
                    }
                    Some(ValueToken::Extension) => tokens.push(Token::Extension(token)),
                    Some(ValueToken::MetaInfoKey) => tokens.push(Token::MetaInfoKey(token)),
                    Some(ValueToken::MetaInfoValue) => tokens.push(Token::MetaInfoValue(token)),
                    Some(ValueToken::Chord) => {
                        // 不正なコード（何らかの数字またはoが入っている）な場合エラー
                        if token.chars().any(|c| c.is_numeric() || c == 'o') {
                            return Err([errors::INVALID_CHORD.to_string(), token].join(": "));
                        }
                        tokens.push(Token::Chord(token))
                    }
                    Some(ValueToken::Denominator) => tokens.push(Token::Denominator(token)),
                    None => {
                        return Err([
                            errors::INVALID_TOKEN_TYPE.to_string(),
                            token_type.unwrap().to_string(),
                        ]
                        .join(": "));
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
        use super::*;

        #[test]
        fn section_meta_info() {
            let input = "@section=A";
            let expected = vec![
                Token::SectionMetaInfoStart,
                Token::SectionMetaInfoKey("section".to_string()),
                Token::Equal,
                Token::SectionMetaInfoValue("A".to_string()),
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
                Token::LineBreak,
                Token::SectionMetaInfoStart,
                Token::SectionMetaInfoKey("section".to_string()),
                Token::Equal,
                Token::SectionMetaInfoValue("A".to_string()),
                Token::LineBreak,
                Token::SectionMetaInfoStart,
                Token::SectionMetaInfoKey("sample".to_string()),
                Token::Equal,
                Token::SectionMetaInfoValue("aaa".to_string()),
                Token::LineBreak,
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
                Token::LineBreak,
                Token::ChordBlockSeparator,
                Token::Chord("C".to_string()),
                Token::ChordBlockSeparator,
                Token::Chord("F".to_string()),
                Token::ChordBlockSeparator,
                Token::Chord("Fm".to_string()),
                Token::ChordBlockSeparator,
                Token::Chord("C".to_string()),
                Token::ChordBlockSeparator,
                Token::LineBreak,
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
                Token::Denominator("F#m".to_string()),
                Token::ExtensionStart,
                Token::Extension("7".to_string()),
                Token::Comma,
                Token::Extension("b5".to_string()),
                Token::ExtensionEnd,
                Token::ChordBlockSeparator,
                Token::Chord("Fbm".to_string()),
                Token::ExtensionStart,
                Token::Extension("13".to_string()),
                Token::ExtensionEnd,
                Token::Slash,
                Token::Denominator("G".to_string()),
                Token::ExtensionStart,
                Token::Extension("7".to_string()),
                Token::ExtensionEnd,
                Token::ChordBlockSeparator,
                Token::LineBreak,
            ];

            let lex_result = tokenize(input);
            assert!(lex_result.is_ok());
            let tokens = lex_result.unwrap();
            assert_eq!(tokens, expected);
        }

        #[test]
        fn chord_block_with_multiple_meta_info() {
            let input = "
                |[key=C][sample=aaa]C|F|[key=Eb]Fm,[sample=bbb]Bb|C|
                ";

            let expected = vec![
                Token::LineBreak,
                Token::ChordBlockSeparator,
                Token::MetaInfoStart,
                Token::MetaInfoKey("key".to_string()),
                Token::Equal,
                Token::MetaInfoValue("C".to_string()),
                Token::MetaInfoEnd,
                Token::MetaInfoStart,
                Token::MetaInfoKey("sample".to_string()),
                Token::Equal,
                Token::MetaInfoValue("aaa".to_string()),
                Token::MetaInfoEnd,
                Token::Chord("C".to_string()),
                Token::ChordBlockSeparator,
                Token::Chord("F".to_string()),
                Token::ChordBlockSeparator,
                Token::MetaInfoStart,
                Token::MetaInfoKey("key".to_string()),
                Token::Equal,
                Token::MetaInfoValue("Eb".to_string()),
                Token::MetaInfoEnd,
                Token::Chord("Fm".to_string()),
                Token::Comma,
                Token::MetaInfoStart,
                Token::MetaInfoKey("sample".to_string()),
                Token::Equal,
                Token::MetaInfoValue("bbb".to_string()),
                Token::MetaInfoEnd,
                Token::Chord("Bb".to_string()),
                Token::ChordBlockSeparator,
                Token::Chord("C".to_string()),
                Token::ChordBlockSeparator,
                Token::LineBreak,
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
                Token::LineBreak,
                // @section=A
                Token::SectionMetaInfoStart,
                Token::SectionMetaInfoKey("section".to_string()),
                Token::Equal,
                Token::SectionMetaInfoValue("A".to_string()),
                Token::LineBreak,
                // @sample=aaa
                Token::SectionMetaInfoStart,
                Token::SectionMetaInfoKey("sample".to_string()),
                Token::Equal,
                Token::SectionMetaInfoValue("aaa".to_string()),
                Token::LineBreak,
                // |C|C7|F|Fm7|
                Token::ChordBlockSeparator,
                Token::Chord("C".to_string()),
                Token::ChordBlockSeparator,
                Token::Chord("C".to_string()),
                Token::ExtensionStart,
                Token::Extension("7".to_string()),
                Token::ExtensionEnd,
                Token::ChordBlockSeparator,
                Token::Chord("F".to_string()),
                Token::ChordBlockSeparator,
                Token::Chord("Fm".to_string()),
                Token::ExtensionStart,
                Token::Extension("7".to_string()),
                Token::ExtensionEnd,
                Token::ChordBlockSeparator,
                Token::LineBreak,
                // |C|C7|F|Fm7|
                Token::ChordBlockSeparator,
                Token::Chord("C".to_string()),
                Token::ChordBlockSeparator,
                Token::Chord("C".to_string()),
                Token::ExtensionStart,
                Token::Extension("7".to_string()),
                Token::ExtensionEnd,
                Token::ChordBlockSeparator,
                Token::Chord("F".to_string()),
                Token::ChordBlockSeparator,
                Token::Chord("Fm".to_string()),
                Token::ExtensionStart,
                Token::Extension("7".to_string()),
                Token::ExtensionEnd,
                Token::ChordBlockSeparator,
                Token::LineBreak,
                Token::LineBreak,
                // @section=B
                Token::SectionMetaInfoStart,
                Token::SectionMetaInfoKey("section".to_string()),
                Token::Equal,
                Token::SectionMetaInfoValue("B".to_string()),
                Token::LineBreak,
                // |[key=F]Gm|Gm|F|F|
                Token::ChordBlockSeparator,
                Token::MetaInfoStart,
                Token::MetaInfoKey("key".to_string()),
                Token::Equal,
                Token::MetaInfoValue("F".to_string()),
                Token::MetaInfoEnd,
                Token::Chord("Gm".to_string()),
                Token::ChordBlockSeparator,
                Token::Chord("Gm".to_string()),
                Token::ChordBlockSeparator,
                Token::Chord("F".to_string()),
                Token::ChordBlockSeparator,
                Token::Chord("F".to_string()),
                Token::ChordBlockSeparator,
                Token::LineBreak,
                // |Gm|Gm|F|F|
                Token::ChordBlockSeparator,
                Token::Chord("Gm".to_string()),
                Token::ChordBlockSeparator,
                Token::Chord("Gm".to_string()),
                Token::ChordBlockSeparator,
                Token::Chord("F".to_string()),
                Token::ChordBlockSeparator,
                Token::Chord("F".to_string()),
                Token::ChordBlockSeparator,
                Token::LineBreak,
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
        fn section_meta_info_key_should_not_contains_line_break() {
            let input = "
            @sect
            ion=A
            ";

            let lex_result = tokenize(input);
            assert!(lex_result.is_err());
            assert_eq!(
                lex_result.unwrap_err(),
                errors::SECTION_META_INFO_KEY_SHOULD_NOT_CONTAINS_LINE_BREAK
            );
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
            assert_eq!(
                lex_result.unwrap_err(),
                errors::META_INFO_KEY_SHOULD_NOT_CONTAINS_LINE_BREAK
            );
        }

        #[test]
        fn meta_info_value_should_not_contains_line_break() {
            let input = "
                |[aaaa=bbbb
                    bbbb]C|
                ";

            let lex_result = tokenize(input);
            assert!(lex_result.is_err());
            assert_eq!(
                lex_result.unwrap_err(),
                errors::META_INFO_VALUE_SHOULD_NOT_CONTAINS_LINE_BREAK
            );
        }

        #[test]
        fn chord_block_should_not_contains_line_break() {
            let input = "
                |C,
                C7|F|Fm(7)|
                ";

            let lex_result = tokenize(input);
            assert!(lex_result.is_err());
            assert_eq!(
                lex_result.unwrap_err(),
                errors::CHORD_BLOCK_SHOULD_NOT_CONTAINS_LINE_BREAK
            );
        }
    }
}
