mod error_code;
mod parser;
mod tokenizer;
mod util;
use error_code::ErrorInfoWithPosition;
use parser::{parse, Ast};
use serde_json::json;
use tokenizer::tokenize;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

// FIXME:
//  "serde_wasm_bindgen::to_value(&json_result).unwrap()" makes Map.
//  But I want to generate JSON.
//  So currently use deprecated from_serde() instead.
#[doc(hidden)]
/// @param {string} input - The chord progression string to parse.
/// @returns {ParsedResult} - The parsed result.
/// @throws {string} - The error information.
#[wasm_bindgen(js_name = "parseChordProgressionString", skip_jsdoc)]
pub fn parse_chord_progression_string_js(input: &str) -> JsValue {
    let result = parse_chord_progression_string(input);

    let json_result = if let Err(error_info) = result {
        json!({
            "success": false,
            "error": {
                "code": error_info.error.code.to_string(),
                "additionalInfo": error_info.error.additional_info,
                "position": {
                    "lineNumber": error_info.position.line_number,
                    "columnNumber": error_info.position.column_number,
                    "length": error_info.position.length,
                },
            }
        })
    } else {
        json!({
            "success": true,
            "ast": result.unwrap(),
        })
    };

    JsValue::from_serde(&json_result).unwrap()
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

        // if C/D, is input, comma is ignored
        #[test]
        fn comma_is_ignored_in_dominator_last_char() {
            let input: &str = "C/D,";
            let result_json = json!(parse_chord_progression_string(input).unwrap());
            let expected = json!([
                {
                    "chordBlocks": [
                        {
                            "type": "bar",
                            "value": [
                                {
                                    "chordExpression": {
                                        "type": "chord",
                                        "value": {
                                            "detailed": {
                                                "accidental": null,
                                                "base": "C",
                                                "chordType": "M",
                                                "extensions": []
                                            },
                                            "plain": "C"
                                        }
                                    },
                                    "denominator": Some("D".to_string()),
                                    "metaInfos": []
                                }
                            ]
                        }
                    ],
                    "metaInfos": []
                }
            ]);

            assert_eq!(result_json, expected);
        }

        // if C/D,E is input, C/D and E are separated
        #[test]
        fn comma_separated_chords_with_denominator() {
            let input: &str = "C/D,E";
            let result_json = json!(parse_chord_progression_string(input).unwrap());
            let expected = json!([
                {
                    "chordBlocks": [
                        {
                            "type": "bar",
                            "value": [
                                {
                                    "chordExpression": {
                                        "type": "chord",
                                        "value": {
                                            "detailed": {
                                                "accidental": null,
                                                "base": "C",
                                                "chordType": "M",
                                                "extensions": []
                                            },
                                            "plain": "C"
                                        }
                                    },
                                    "denominator": "D",
                                    "metaInfos": []
                                },
                                {
                                    "chordExpression": {
                                        "type": "chord",
                                        "value": {
                                            "detailed": {
                                                "accidental": null,
                                                "base": "E",
                                                "chordType": "M",
                                                "extensions": []
                                            },
                                            "plain": "E"
                                        }
                                    },
                                    "denominator": null,
                                    "metaInfos": []
                                }
                            ]
                        }
                    ],
                    "metaInfos": []
                }
            ]);

            assert_eq!(result_json, expected);
        }

        #[test]
        fn only_section_meta() {
            let input: &str = "@section=A";

            let result_json = json!(parse_chord_progression_string(input).unwrap());
            let expected = json!([
                {
                    "chordBlocks": [],
                    "metaInfos": [
                        {
                            "type": "section",
                            "value": "A"
                        }
                    ]
                }
            ]);

            assert_eq!(result_json, expected);
        }

        #[test]
        fn only_tension() {
            let input: &str = "C(9,11,13,o)";

            let result_json = json!(parse_chord_progression_string(input).unwrap());
            let expected = json!([
                {
                    "chordBlocks": [
                        {
                            "type": "bar",
                            "value": [{
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
                            }]
                        }
                    ],
                    "metaInfos": []
                }
            ]);

            assert_eq!(result_json, expected);
        }

        #[test]
        fn complex_input_snapshot() {
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

            insta::assert_debug_snapshot!(parse_chord_progression_string(input));
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
                        {
                            "type": "bar",
                            "value": [
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
                                            "value": "C",
                                        }
                                    ]
                                },
                            ]
                        },
                        {
                            "type": "bar",
                            "value": [
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
                        },
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
