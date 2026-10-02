mod error_code;
mod formatter;
mod lexer;
mod model;
mod parse_report;
mod parser;
mod util;
mod warning_code;
#[cfg(any(target_arch = "wasm32", test))]
mod wasm_ast;
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
pub use parse_report::{ParseReport, ParseWarning};
pub use util::position::Position;
pub use warning_code::WarningCode;

/** Successful JavaScript response serialized as a plain object. */
#[derive(Serialize)]
struct JsParseSuccess {
    success: bool,
    ast: Ast,
    warnings: Vec<JsParseDiagnostic>,
}

/** Failed JavaScript response serialized as a plain object. */
#[derive(Serialize)]
struct JsParseFailure {
    success: bool,
    errors: Vec<JsParseDiagnostic>,
    warnings: Vec<JsParseDiagnostic>,
}

/** JavaScript-facing error or warning with camel-case field names. */
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsParseDiagnostic {
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

impl From<Position> for JsPosition {
    /** Converts a source position without changing its UTF-16 range. */
    fn from(position: Position) -> Self {
        Self {
            line_number: position.line_number,
            column_number: position.column_number,
            length: position.length,
            start_offset: position.start_offset,
            end_offset: position.end_offset,
        }
    }
}

impl From<ErrorInfoWithPosition> for JsParseDiagnostic {
    /** Retains the existing JavaScript error shape and code. */
    fn from(error: ErrorInfoWithPosition) -> Self {
        Self {
            code: error.error.code.to_string(),
            additional_info: error.error.additional_info,
            position: error.position.into(),
        }
    }
}

impl From<ParseWarning> for JsParseDiagnostic {
    /** Serializes warning codes independently of the error-code domain. */
    fn from(warning: ParseWarning) -> Self {
        Self {
            code: warning.code.to_string(),
            additional_info: warning.additional_info,
            position: warning.position.into(),
        }
    }
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
import type { WarningCode } from "./warning_code_message_map.js";
export type { WarningCode } from "./warning_code_message_map.js";

/** Source coordinates for both errors and warnings; offsets can be passed to String.slice. */
export type ParsePosition = {
  /** One-based line number; CRLF counts as one line break. */
  lineNumber: number;
  /** One-based Unicode scalar column; tabs count as one scalar. */
  columnNumber: number;
  /** Number of Unicode scalar values in the range; zero at EOF. */
  length: number;
  /** Zero-based UTF-16 offset into the original input. */
  startOffset: number;
  /** Exclusive zero-based UTF-16 offset into the original input. */
  endOffset: number;
};

export type ParseError = {
  code: ErrorCode;
  additionalInfo: string | null;
  position: ParsePosition;
};

export type ParseWarning = {
  code: WarningCode;
  additionalInfo: string | null;
  position: ParsePosition;
};

export type ParsedResult =
  | {
      success: true;
      ast: Ast;
      warnings: ParseWarning[];
    }
  | {
      success: false;
      errors: ParseError[];
      warnings: ParseWarning[];
    };

/** Formats an AST returned by parseChordProgressionString. */
export function formatChordProgression(ast: Ast): string;
"#;

#[doc(hidden)]
/** Validates JavaScript arguments before allocating their UTF-8 representation. */
#[wasm_bindgen(
    js_name = "parseChordProgressionString",
    skip_jsdoc,
    unchecked_return_type = "ParsedResult"
)]
pub fn parse_chord_progression_string_export(
    #[wasm_bindgen(unchecked_param_type = "string")] input: JsValue,
) -> Result<JsValue, JsValue> {
    // The generated &str argument encoder can allocate before discovering that
    // an array-like argument is not a string, leaving that allocation unclaimed.
    let input = input
        .as_string()
        .ok_or_else(|| JsValue::from_str("invalid chord progression input: expected a string"))?;
    Ok(parse_chord_progression_string_js(&input))
}

#[doc(hidden)]
/** Parses a Rust string into the JavaScript response shape. */
pub fn parse_chord_progression_string_js(input: &str) -> JsValue {
    let ParseReport { result, warnings } = parse_chord_progression_string_with_warnings(input);
    let warnings = warnings.into_iter().map(JsParseDiagnostic::from).collect();
    let response = match result {
        Ok(ast) => JsParseResult::Success(JsParseSuccess {
            success: true,
            ast,
            warnings,
        }),
        Err(error_infos) => JsParseResult::Failure(JsParseFailure {
            success: false,
            warnings,
            errors: error_infos
                .into_iter()
                .map(JsParseDiagnostic::from)
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
    // Ast is a JavaScript array. Reject primitives before serde's iterable lookup:
    // caught Reflect.get exceptions currently retain slots in wasm-bindgen's table.
    if !js_sys::Array::is_array(&ast) {
        return Err(JsValue::from_str(
            "invalid chord progression AST: expected an array",
        ));
    }
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
    // Legacy callers and formatter validation have no use for allocated warning messages.
    parser::parse(input, false).result
}

/** Parses a progression and retains notation warnings on both success and failure. */
pub fn parse_chord_progression_string_with_warnings(input: &str) -> ParseReport {
    parser::parse(input, true)
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
