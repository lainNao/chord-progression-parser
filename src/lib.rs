mod error_code;
mod formatter;
mod lexer;
mod model;
mod parser;
mod util;
use serde::Serialize;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

pub use error_code::{ErrorCode, ErrorInfo, ErrorInfoWithPosition};
pub use formatter::{format_chord_progression, FormatError};
pub use model::{
    accidental::Accidental, ast::Ast, bar::Bar, base::Base, chord::Chord, chord_block::ChordBlock,
    chord_detailed::ChordDetailed, chord_expression::ChordExpression, chord_info::ChordInfo,
    chord_info_meta::ChordInfoMeta, chord_type::ChordType, extension::Extension, key::Key,
    section::Section, section_meta::SectionMeta,
};
pub use util::position::Position;

/** Successful JavaScript response serialized as a plain object. */
#[derive(Serialize)]
struct JsParseSuccess {
    success: bool,
    ast: Ast,
}

/** Failed JavaScript response serialized as a plain object. */
#[derive(Serialize)]
struct JsParseFailure {
    success: bool,
    errors: Vec<JsParseError>,
}

/** JavaScript-facing parse error with camel-case field names. */
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsParseError {
    code: String,
    additional_info: Option<String>,
    position: JsPosition,
}

/** JavaScript-facing source position with camel-case field names. */
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsPosition {
    line_number: usize,
    column_number: usize,
    length: usize,
    start_offset: usize,
    end_offset: usize,
}

/** Represents either JavaScript response shape without adding an enum tag. */
#[derive(Serialize)]
#[serde(untagged)]
enum JsParseResult {
    Success(JsParseSuccess),
    Failure(JsParseFailure),
}

#[wasm_bindgen(typescript_custom_section)]
const PARSED_RESULT_TYPES: &str = r#"
import type { Ast } from "./generatedTypes.js";
import type { ErrorCode } from "./error_code_message_map.js";

export type ParsedResult =
  | {
      success: true;
      ast: Ast;
    }
  | {
      success: false;
      errors: Array<{
        code: ErrorCode;
        additionalInfo: string | null;
        position: {
          lineNumber: number;
          columnNumber: number;
          length: number;
          startOffset: number;
          endOffset: number;
        };
      }>;
    };

/** Formats an AST returned by parseChordProgressionString. */
export function formatChordProgression(ast: Ast): string;
"#;

#[doc(hidden)]
/// @param {string} input - The chord progression string to parse.
/// @returns {ParsedResult} - The parsed result.
#[wasm_bindgen(
    js_name = "parseChordProgressionString",
    skip_jsdoc,
    unchecked_return_type = "ParsedResult"
)]
pub fn parse_chord_progression_string_js(input: &str) -> JsValue {
    let response = match parse_chord_progression_string(input) {
        Ok(ast) => JsParseResult::Success(JsParseSuccess { success: true, ast }),
        Err(error_infos) => JsParseResult::Failure(JsParseFailure {
            success: false,
            errors: error_infos
                .into_iter()
                .map(|error_info| JsParseError {
                    code: error_info.error.code.to_string(),
                    additional_info: error_info.error.additional_info,
                    position: JsPosition {
                        line_number: error_info.position.line_number,
                        column_number: error_info.position.column_number,
                        length: error_info.position.length,
                        start_offset: error_info.position.start_offset,
                        end_offset: error_info.position.end_offset,
                    },
                })
                .collect(),
        }),
    };

    response
        .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
        .expect("serializing the fixed JavaScript response types should not fail")
}

/** Formats a parser-produced JavaScript AST into stable chord-progression source text. */
#[wasm_bindgen(js_name = "formatChordProgression", skip_typescript)]
pub fn format_chord_progression_js(ast: JsValue) -> Result<String, JsValue> {
    let ast: Ast = serde_wasm_bindgen::from_value(ast)
        .map_err(|error| JsValue::from_str(&format!("invalid chord progression AST: {error}")))?;

    format_chord_progression(&ast)
        .map_err(|error| JsValue::from_str(&format!("invalid chord progression AST: {error}")))
}

/// Parse a chord progression string and return the AST
///
/// # Example
/// ```rust
/// use chord_progression_parser::parse_chord_progression_string;
///
/// let input: &str = "
/// @section=Intro
/// [key=E]E-C#m(7)-Bm(7)-C#(7)
/// F#m(7)-Am(7)-F#(7)-B
///
/// @section=Verse
/// E-C#m(7)-Bm(7)-C#(7)
/// F#m(7)-Am(7)-F#(7)-B
/// ";
///     
/// let result = parse_chord_progression_string(input);
/// println!("{:#?}", result);
/// ```
///
/// # Errors
///
/// Returns all recoverable errors and source ranges when the input does not follow the grammar.
pub fn parse_chord_progression_string(input: &str) -> Result<Ast, Vec<ErrorInfoWithPosition>> {
    parser::parse(input)
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
                result.unwrap_err()[0].position,
                Position {
                    line_number: 1,
                    column_number: 5,
                    length: 3,
                    start_offset: 4,
                    end_offset: 7,
                },
            );

            let errors = parse_chord_progression_string(input).expect_err("fixture must fail");
            assert_eq!(errors[0].error.to_string(), "EXT-1: 111");
        }
    }
}
