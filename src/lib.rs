mod error_code;
mod parser;
mod tokenizer;

use error_code::ErrorInfo;
use parser::{parse, Ast};
use tokenizer::tokenize;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen]
pub fn run(input: &str) -> Result<JsValue, String> {
    let result = parse_chord_progression_string(input);

    if result.is_err() {
        return Err(result.err().unwrap().to_string());
    }

    serde_wasm_bindgen::to_value(&result.unwrap())
        .map_err(|e| format!("Serialization Error: {}", e))
}

pub fn parse_chord_progression_string(input: &str) -> Result<Ast, ErrorInfo> {
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
        use crate::{parser::parse, tokenizer::tokenize};
        use serde_json::json;

        #[test]
        fn complex_input_can_be_parsed() {
            let input: &str = "
            @section=Intro
            |[key=E]E|C#m(7)|Bm(7)|C#(7)|
            |F#m(7)|Am(7)|F#(7)|B|
            
            @section=Verse
            |E|C#m(7)|Bm(7)|C#(7)|
            |F#m(7)|Am(7)|F#(7)|B|
            
            @section=Chorus
            |[key=C]C|C(7)|FM(7)|Fm(7)|
            |C|C(7)|FM(7)|Dm(7)|
            |Em(7)|E(7)|
        
            @section=Interlude
            |C|A,B|

            |[key=C]C(M9)|CM(9)|
            ";

            let result = parse(&tokenize(input).unwrap());
            assert!(result.is_ok());
        }

        #[test]
        fn differ_major_9_vs_9_of_major() {
            let input: &str = "
            @section=Intro
            |[key=C]C(M9)|CM(9)|
            ";

            let result_json = json!(parse(&tokenize(input).unwrap()).unwrap());
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
                                        "value": {
                                            "value":"C_M"
                                        }
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
                            "value": {
                                "value":"Intro"
                            }
                        }
                    ]
                }
            ]);

            assert_eq!(result_json, expected);
        }
    }
}
