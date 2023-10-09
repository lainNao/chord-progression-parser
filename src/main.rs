mod errors;
mod parser;
mod tokenizer;

use parser::parse;
use tokenizer::tokenize;

fn main() {
    let input = "
    @section=Intro
    |(key=E)E|C#m7|Bm7|C#7|
    |F#m7|Am7|F#7|B|
    
    @section=Verse
    |E|C#m7|Bm7|C#7|
    |F#m7|Am7|F#7|B|
    
    @section=Chorus
    |(key=C)C|C7|FM7|Fm7|
    |C|C7|FM7|Dm7|
    |Em7|E7|

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
}
