mod errors;
mod lexer;
mod parser;

fn main() {
    let input = "
    @section=A
    @sample=aaa
    |C|F|G|C|
    |C|F|G|A|
    
    @section=B
    |(key=F)Gm|Gm|F|F|
    |Gm|Gm|F|F|
    ";

    println!("Input: {}", input);

    let lex_result = lexer::lexer(input);
    if let Err(e) = lex_result {
        println!("Error: {}", e);
        return;
    }
    let tokens = lex_result.unwrap();

    println!("Tokens:");
    for token in &tokens {
        println!("{:?}", token);
    }

    println!("Starting parsing");
    match parser::parser(&tokens) {
        Ok(ast) => println!("{:?}", ast),
        Err(e) => println!("Error: {}", e),
    }
}
