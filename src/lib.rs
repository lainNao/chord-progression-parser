mod error_code;
mod parser;
mod tokenizer;
mod util;

use std::panic;

use error_code::ErrorInfoWithPosition;
use parser::{parse, Ast};
use serde_json::json;
use tokenizer::tokenize;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use crate::error_code::ErrorCode;

#[doc(hidden)]
/// @param {string} input - The chord progression string to parse.
/// @param {string} lang - The language to use for error messages.
/// @returns {Ast} - The parsed chord progression.
/// @throws {{code: string, additionalInfo: string} | string, position: {lineNumber: number, columnNumber: number}} - The error information.
#[wasm_bindgen(js_name = "parseChordProgressionString", skip_jsdoc)]
pub fn parse_chord_progression_string_js(input: &str) -> Result<JsValue, JsValue> {
    let result_of_result = panic::catch_unwind(|| parse_chord_progression_string(input));

    if result_of_result.is_err() {
        return Err(JsValue::from_str(
            &json!({
                "code": ErrorCode::Other1.to_string(),
                "additionalInfo": "",
                "position": {
                    "lineNumber": 0,
                    "columnNumber": 0,
                    "length": 0,
                },
            })
            .to_string(),
        ));
    }

    let result = result_of_result.unwrap();

    if result.is_err() {
        let error_info = result.err().unwrap();
        return Err(JsValue::from_str(
            &json!({
                "code": error_info.error.code.to_string(),
                "additionalInfo": error_info.error.additional_info,
                "position": {
                    "lineNumber": error_info.position.line_number,
                    "columnNumber": error_info.position.column_number,
                    "length": error_info.position.length,
                },
            })
            .to_string(),
        ));
    }

    serde_wasm_bindgen::to_value(&result.unwrap())
        .map_err(|err| JsValue::from_str(&format!("{}", err)))
}

/// Parse a chord progression string and return the AST
///
/// # Example
/// ```rust
/// use chord_progression_parser::parse_chord_progression_string;
///
/// let input: &str = "
/// @section=Intro
/// |[key=E]E|C#m(7)|Bm(7)|C#(7)|
/// |F#m(7)|Am(7)|F#(7)|B|
///
/// @section=Verse
/// |E|C#m(7)|Bm(7)|C#(7)|
/// |F#m(7)|Am(7)|F#(7)|B|
/// ";
///     
/// let result = parse_chord_progression_string(input);
/// println!("{:#?}", result);
/// ```
///
/// # Panics
///
/// Panics if unhandled error occurs.
pub fn parse_chord_progression_string(input: &str) -> Result<Ast, ErrorInfoWithPosition> {
    let tokenized_result = tokenize(input);
    if tokenized_result.is_err() {
        return Err(tokenized_result.err().unwrap());
    }

    let parsed_result = parse(&tokenized_result.unwrap());
    if parsed_result.is_err() {
        return Err(parsed_result.err().unwrap());
    }

    Ok(parsed_result.unwrap())
}

#[cfg(test)]
mod tests {
    #[cfg(test)]
    mod success {
        use crate::parse_chord_progression_string;
        use serde_json::json;

        #[test]
        fn only_tension() {
            let input: &str = "C(9,11,13,o)";

            let result_json = json!(parse_chord_progression_string(input).unwrap());
            let expected = json!([
                {
                    "chordBlocks": [
                        [
                            {
                                "chordExpression": {
                                    "type": "chord",
                                    "value": {
                                        "detailed": {
                                            "accidental": null,
                                            "base": "C",
                                            "chordType": "M",
                                            "extensions": [
                                                "9",
                                                "11",
                                                "13",
                                                "o"
                                            ]
                                        },
                                        "plain": "C(9,11,13,o)"
                                    }
                                },
                                "denominator": null,
                                "metaInfos": []
                            }
                        ]
                    ],
                    "metaInfos": []
                }
            ]);

            assert_eq!(result_json, expected);
        }

        #[test]
        fn complex_input_can_be_parsed() {
            let input: &str = "
@section=Intro
[key=E]E-C#m(7)-Bm(7)-C#(7)
F#m(7)-Am(7)-F#(7)-B

@section=Verse
E-C#m(7)-Bm(7)-C#(7)
F#m(7)-Am(7)-F#(7)-B

@section=Chorus
[key=C]C-C(7)-FM(7)-Fm(7)
C-C(7)-FM(7)-Dm(7)
Em(7)-E(7)
        
@section=Interlude
C-A,B

[key=C]C(M9)-CM(9)
";

            let result = parse_chord_progression_string(input);
            println!("111 {:#?}", result);
            assert!(result.is_ok());
        }

        #[test]
        fn differ_major_9_vs_9_of_major() {
            let input: &str = "
            @section=Intro
            [key=C]C(M9)-CM(9)
            ";

            let result_json = json!(parse_chord_progression_string(input).unwrap());
            let expected = json!([
                {
                    "chordBlocks": [
                        [
                            {
                                "chordExpression": {
                                    "type": "chord",
                                    "value": {
                                        "detailed": {
                                            "accidental": null,
                                            "base":"C",
                                            "chordType":"M",
                                            "extensions": [
                                                "M9"
                                            ]
                                        },
                                        "plain":"C(M9)"
                                    }
                                },
                                "denominator":null,
                                "metaInfos": [
                                    {
                                        "type": "key",
                                        "value": "C_M",
                                    }
                                ]
                            },
                            {
                                "chordExpression": {
                                    "type": "chord",
                                    "value": {
                                        "detailed": {
                                            "accidental": null,
                                            "base":"C",
                                            "chordType":"M",
                                            "extensions": [
                                                "9"
                                            ]
                                        },
                                        "plain":"CM(9)"
                                    }
                                },
                                "denominator":null,
                                "metaInfos": []
                            }
                        ]
                    ],
                    "metaInfos": [
                        {
                            "type": "section",
                            "value": "Intro"
                        }
                    ]
                }
            ]);

            assert_eq!(result_json, expected);
        }
    }

    mod failure {
        use crate::{parse_chord_progression_string, util::position::Position};

        #[test]
        fn tension_position_when_error() {
            let input: &str = "C(9,111)";

            let result = parse_chord_progression_string(input);
            assert_eq!(
                result.unwrap_err().position,
                Position {
                    line_number: 1,
                    column_number: 5,
                    length: 3,
                },
            )
        }
    }
}
