mod errors;
mod parser;
mod tokenizer;

use parser::parse;
use tokenizer::tokenize;

fn main() {
    let input = "
    @section=A
    |C|F|G|C|
    |C|F|G|A|
    
    @section=B
    |(key=F)Gm|Gm|F|F|
    |Gm|Gm|F|F|
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
