use crate::errors;

pub mod types;
pub mod util;

use types::{Token, ValueToken};
use util::is_token_char;

pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '@' => tokens.push(Token::SectionMetaInfoStart),
            '(' => tokens.push(Token::MetaInfoStart),
            ')' => tokens.push(Token::MetaInfoEnd),
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
                let second_token_from_last = if tokens.len() >= 2 {
                    tokens.get(tokens.len() - 2)
                } else {
                    None
                };

                // get token type
                let get_token_type_result = match tokens.last() {
                    Some(Token::SectionMetaInfoStart) => Ok(Some(ValueToken::SectionMetaInfoKey)),
                    Some(Token::MetaInfoStart) => Ok(Some(ValueToken::MetaInfoKey)),
                    Some(Token::Equal) => match second_token_from_last {
                        Some(Token::SectionMetaInfoKey(_)) => {
                            Ok(Some(ValueToken::SectionMetaInfoValue))
                        }
                        Some(Token::MetaInfoKey(_)) => Ok(Some(ValueToken::MetaInfoValue)),
                        _ => {
                            return Err([
                                errors::INVALID_TOKEN_TYPE.to_string(),
                                second_token_from_last.unwrap().to_string(),
                            ]
                            .join(": "));
                        }
                    },
                    Some(Token::Slash) => Ok(Some(ValueToken::Denominator)),
                    _ => Ok(Some(ValueToken::Chord)),
                };

                // if error, return
                let token_type = if let Ok(token_type) = get_token_type_result {
                    token_type
                } else {
                    return Err(get_token_type_result.unwrap_err());
                };

                // get token
                while let Some(&next_ch) = chars.peek() {
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
                    Some(ValueToken::MetaInfoKey) => tokens.push(Token::MetaInfoKey(token)),
                    Some(ValueToken::MetaInfoValue) => tokens.push(Token::MetaInfoValue(token)),
                    Some(ValueToken::Chord) => tokens.push(Token::Chord(token)),
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
            println!("111 {:?}", lex_result);
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
                |F#m7-5/F#m7-5|Fbm13/G7|
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

            let lex_result = tokenize(input);
            assert!(lex_result.is_ok());
            let tokens = lex_result.unwrap();
            assert_eq!(tokens, expected);
        }

        #[test]
        fn chord_block_with_multiple_meta_info() {
            let input = "
                |(key=C)(sample=aaa)C|F|(key=Eb)Fm,(sample=bbb)Bb|C|
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
            println!("222 {:?}", lex_result);
            assert!(lex_result.is_ok());
            let tokens = lex_result.unwrap();
            assert_eq!(tokens, expected);
        }

        #[test]
        fn complicated() {
            let input = "
                @section=A
                @sample=aaa
                |C|C7|F|Fm7|
                |C|C7|F|Fm7|

                @section=B
                |(key=F)Gm|Gm|F|F|
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
                Token::Chord("C7".to_string()),
                Token::ChordBlockSeparator,
                Token::Chord("F".to_string()),
                Token::ChordBlockSeparator,
                Token::Chord("Fm7".to_string()),
                Token::ChordBlockSeparator,
                Token::LineBreak,
                // |C|C7|F|Fm7|
                Token::ChordBlockSeparator,
                Token::Chord("C".to_string()),
                Token::ChordBlockSeparator,
                Token::Chord("C7".to_string()),
                Token::ChordBlockSeparator,
                Token::Chord("F".to_string()),
                Token::ChordBlockSeparator,
                Token::Chord("Fm7".to_string()),
                Token::ChordBlockSeparator,
                Token::LineBreak,
                Token::LineBreak,
                // @section=B
                Token::SectionMetaInfoStart,
                Token::SectionMetaInfoKey("section".to_string()),
                Token::Equal,
                Token::SectionMetaInfoValue("B".to_string()),
                Token::LineBreak,
                // |(key=F)Gm|Gm|F|F|
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
                |(aaaa
                    aaaa=bbbb
                )C|
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
                |(aaaa=bbbb
                    bbbb)C|
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
                C7|F|Fm7|
                ";

            let lex_result = tokenize(input);
            println!("222 {:?}", lex_result);
            assert!(lex_result.is_err());
            assert_eq!(
                lex_result.unwrap_err(),
                errors::CHORD_BLOCK_SHOULD_NOT_CONTAINS_LINE_BREAK
            );
        }
    }
}
