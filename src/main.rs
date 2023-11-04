mod errors;
mod parser;
mod tokenizer;

use parser::parse;
use serde_json::{json, Value};
use tokenizer::tokenize;

fn main() {
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
    ";
    println!("Input: {}", input);

    let tokens = match tokenize(input) {
        Ok(t) => t,
        Err(e) => {
            println!("Tokenization Error: {}", e);
            return;
        }
    };
    println!("Tokens: {:?}\n", tokens);

    let ast = match parse(&tokens) {
        Ok(ast) => ast,
        Err(e) => {
            println!("Parse Error: {}", e);
            return;
        }
    };
    println!("AST: {:?}\n", ast);

    let json = json!(ast);
    println!("JSON: {}", json);
}

// test
#[cfg(test)]
mod tests {
    #[cfg(test)]
    mod success {
        use serde_json::json;

        use crate::{parser::parse, tokenizer::tokenize};

        #[test]
        fn differ_major_9_vs_9_of_major() {
            let input: &str = "
            @section=Intro
            |[key=C]C(M9)|CM(9)|
            ";

            let result_json = json!(parse(&tokenize(input).unwrap()).unwrap());

            let expected = json!([
                {
                    "chord_blocks": [
                        [
                            {
                                "chord": {
                                    "Chord":{
                                        "detailed": {
                                            "accidental":null,
                                            "base":"C",
                                            "chord_type":"Major",
                                            "extensions": [
                                                "MajorNine"
                                            ]
                                        },
                                        "plain":"C(M9)"
                                    }
                                },
                                "denominator":null,
                                "meta_infos": [
                                    {
                                        "Key":{
                                            "value":"C_M"
                                        }
                                    }
                                ]
                            },
                            {
                                "chord": {
                                    "Chord":{
                                        "detailed": {
                                            "accidental":null,
                                            "base":"C",
                                            "chord_type":"Major",
                                            "extensions": [
                                                "Nine"
                                            ]
                                        },
                                        "plain":"CM(9)"
                                    }
                                },
                                "denominator":null,
                                "meta_infos": []
                            }
                        ]
                    ],
                    "meta_infos": [
                        {
                            "Section": {
                                "value":"Intro"
                            }
                        }
                    ]
                }
            ]);

            println!("result_json: {}", result_json);
            println!("expected: {}", expected);

            assert_eq!(result_json, expected);
        }
    }
}
