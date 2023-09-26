use crate::lexer::Token;

/*

type ParsedResult = Section[]

type Section = {
    meta: SectionMeta[]
    codeBlocks: CodeBlock[]
}

type SectionMeta = {
    key: "section"
    value: string
} | {
    key: "repeat"
    value: number
}

type CodeBlock = CodeInfo[]     //CSVで別れたコード情報達

type CodeInfo = {
    meta: CodeInfoMeta[]
    codes: Code[]
}

type Code = string;
    ここ、場合によってはコードを以下に分割してもいいかも
        type Code = {
            raw: A#m7b5            //全部統合したやつ。以下は分解したやつ
            base: A~G
            accidental: #|b
            type: m|M|aug|dim|add|sus2|sus4
            extension: 2|3|b3|4|b5|-5|5|#5|b6|6|7|b9|9|#9|b11|11|#11|b13|13|#13
        }

type CodeInfoMeta = {
    key: "key"
    value: Cb | Cbm | C | Cm | C# | C#m | Db | Dbm | D | Dm | D# | D#m | Eb | Ebm | E | Em | Fb | Fbm | F | Fm | F# | F#m | Gb | Gbm | G | Gm | G# | G#m | Ab | Abm | A | Am | A# | A#m | Bb | Bbm | B | Bm
}

*/

#[derive(Debug, PartialEq, Clone)]
pub enum Ast {
    Section {
        meta: Option<Meta>,
        code_blocks: Vec<CodeBlock>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct Meta {
    key: String,
    value: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct CodeBlock {
    meta: Option<Meta>,
    codes: Vec<String>,
}

pub fn parser(tokens: &[Token]) -> Result<Ast, String> {
    let mut tokens = tokens.iter().peekable();
    let mut section_meta: Option<Meta> = None;
    let mut code_blocks: Vec<CodeBlock> = Vec::new();

    while let Some(token) = tokens.next() {
        match token {
            Token::MetaInfoStart => {
                if let Some(Token::MetaInfoKey(key)) = tokens.next() {
                    if let Some(Token::Equal) = tokens.next() {
                        if let Some(Token::Code(value)) = tokens.next() {
                            section_meta = Some(Meta {
                                key: key.clone(),
                                value: value.clone(),
                            });
                            #[cfg(debug_assertions)]
                            println!("Parser: Parsed section meta: {:?}", section_meta);
                        } else {
                            return Err(
                                "Error: Expected Token::Value after Token::Equal".to_string()
                            );
                        }
                    } else {
                        return Err(
                            "Error: Expected Token::Equal after Token::MetaInfo".to_string()
                        );
                    }
                } else {
                    return Err(
                        "Error: Expected Token::MetaInfo after Token::SectionMetaStart".to_string(),
                    );
                }
            }
            Token::CodeBlockSeparator => {
                let mut meta: Option<Meta> = None;
                let mut codes: Vec<String> = Vec::new();

                loop {
                    match tokens.peek() {
                        Some(Token::MetaInfoKey(key)) => {
                            tokens.next(); // Consume the peeked token
                            if let Some(Token::Equal) = tokens.next() {
                                if let Some(Token::Code(value)) = tokens.next() {
                                    meta = Some(Meta {
                                        key: key.clone(),
                                        value: value.clone(),
                                    });
                                    #[cfg(debug_assertions)]
                                    println!("Parser: Parsed code block meta: {:?}", meta);
                                } else {
                                    return Err("Error: Expected Token::Value after Token::Equal"
                                        .to_string());
                                }
                            } else {
                                return Err("Error: Expected Token::Equal after Token::MetaInfo"
                                    .to_string());
                            }
                        }
                        Some(Token::Code(code)) => {
                            tokens.next(); // Consume the peeked token
                            codes.push(code.clone());
                        }
                        Some(Token::CodeBlockSeparator) => {
                            tokens.next(); // Consume the peeked token
                            break;
                        }
                        _ => {
                            return Err("Unexpected token inside a code block".to_string());
                        }
                    }
                }

                code_blocks.push(CodeBlock { meta, codes });
            }
            _ => {}
        }
    }

    Ok(Ast::Section {
        meta: section_meta,
        code_blocks,
    })
}
