/** A structural token produced without assigning parser-specific meaning. */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TokenKind<'src> {
    At,
    LeftBracket,
    RightBracket,
    LeftParen,
    RightParen,
    Equal,
    Comma,
    Slash,
    Dash,
    Newline,
    Text(&'src str),
}

/** The source range of a token and its one-based display position. */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SourceSpan {
    pub(crate) start_byte: usize,
    pub(crate) end_byte: usize,
    pub(crate) line: usize,
    pub(crate) column: usize,
    pub(crate) length: usize,
}

/** A token paired with the exact source range from which it was read. */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Token<'src> {
    pub(crate) kind: TokenKind<'src>,
    pub(crate) span: SourceSpan,
}

/** Converts a structural character into its context-free token kind. */
fn structural_kind(ch: char) -> Option<TokenKind<'static>> {
    match ch {
        '@' => Some(TokenKind::At),
        '[' => Some(TokenKind::LeftBracket),
        ']' => Some(TokenKind::RightBracket),
        '(' => Some(TokenKind::LeftParen),
        ')' => Some(TokenKind::RightParen),
        '=' => Some(TokenKind::Equal),
        ',' => Some(TokenKind::Comma),
        '/' => Some(TokenKind::Slash),
        '-' => Some(TokenKind::Dash),
        _ => None,
    }
}

/** Returns whether a character terminates an unclassified text token. */
fn ends_text(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\r' | '\n') || structural_kind(ch).is_some()
}

/** Splits source text into context-free tokens while retaining exact spans. */
pub(crate) fn lex(input: &str) -> Vec<Token<'_>> {
    let mut tokens = Vec::new();
    let mut chars = input.char_indices().peekable();
    let mut line = 1;
    let mut column = 1;

    while let Some((start_byte, ch)) = chars.next() {
        match ch {
            ' ' | '\t' => {
                column += 1;
            }
            '\r' | '\n' => {
                let start_line = line;
                let start_column = column;
                let mut end_byte = start_byte + ch.len_utf8();
                let mut length = 1;

                if ch == '\r' && chars.peek().is_some_and(|(_, next)| *next == '\n') {
                    if let Some((next_byte, next)) = chars.next() {
                        end_byte = next_byte + next.len_utf8();
                        length += 1;
                    }
                }

                tokens.push(Token {
                    kind: TokenKind::Newline,
                    span: SourceSpan {
                        start_byte,
                        end_byte,
                        line: start_line,
                        column: start_column,
                        length,
                    },
                });
                line += 1;
                column = 1;
            }
            _ => {
                if let Some(kind) = structural_kind(ch) {
                    tokens.push(Token {
                        kind,
                        span: SourceSpan {
                            start_byte,
                            end_byte: start_byte + ch.len_utf8(),
                            line,
                            column,
                            length: 1,
                        },
                    });
                    column += 1;
                    continue;
                }

                let start_column = column;
                let mut end_byte = start_byte + ch.len_utf8();
                let mut length = 1;
                column += 1;

                while let Some((next_byte, next)) = chars.peek().copied() {
                    if ends_text(next) {
                        break;
                    }

                    chars.next();
                    end_byte = next_byte + next.len_utf8();
                    length += 1;
                    column += 1;
                }

                tokens.push(Token {
                    kind: TokenKind::Text(&input[start_byte..end_byte]),
                    span: SourceSpan {
                        start_byte,
                        end_byte,
                        line,
                        column: start_column,
                        length,
                    },
                });
            }
        }
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::{lex, SourceSpan, Token, TokenKind};

    /** Verifies structural tokens, text tokens, and skipped horizontal space. */
    #[test]
    fn lexes_structure_without_assigning_context() {
        let input = " @section = Verse\n[key=C]C(M9),G/B-D ";
        let kinds: Vec<TokenKind<'_>> = lex(input).iter().map(|token| token.kind).collect();

        assert_eq!(
            kinds,
            vec![
                TokenKind::At,
                TokenKind::Text("section"),
                TokenKind::Equal,
                TokenKind::Text("Verse"),
                TokenKind::Newline,
                TokenKind::LeftBracket,
                TokenKind::Text("key"),
                TokenKind::Equal,
                TokenKind::Text("C"),
                TokenKind::RightBracket,
                TokenKind::Text("C"),
                TokenKind::LeftParen,
                TokenKind::Text("M9"),
                TokenKind::RightParen,
                TokenKind::Comma,
                TokenKind::Text("G"),
                TokenKind::Slash,
                TokenKind::Text("B"),
                TokenKind::Dash,
                TokenKind::Text("D"),
            ]
        );
    }

    /** Verifies Unicode columns and byte offsets independently. */
    #[test]
    fn tracks_unicode_text_by_character_and_byte() {
        let input = "Cあ\nD";

        assert_eq!(
            lex(input),
            vec![
                Token {
                    kind: TokenKind::Text("Cあ"),
                    span: SourceSpan {
                        start_byte: 0,
                        end_byte: 4,
                        line: 1,
                        column: 1,
                        length: 2,
                    },
                },
                Token {
                    kind: TokenKind::Newline,
                    span: SourceSpan {
                        start_byte: 4,
                        end_byte: 5,
                        line: 1,
                        column: 3,
                        length: 1,
                    },
                },
                Token {
                    kind: TokenKind::Text("D"),
                    span: SourceSpan {
                        start_byte: 5,
                        end_byte: 6,
                        line: 2,
                        column: 1,
                        length: 1,
                    },
                },
            ]
        );
    }

    /** Treats CRLF as one newline while retaining both source characters. */
    #[test]
    fn treats_crlf_as_one_line_break() {
        let tokens = lex("C\r\nD");

        assert_eq!(tokens[1].kind, TokenKind::Newline);
        assert_eq!(tokens[1].span.start_byte, 1);
        assert_eq!(tokens[1].span.end_byte, 3);
        assert_eq!(tokens[1].span.length, 2);
        assert_eq!(tokens[2].span.line, 2);
        assert_eq!(tokens[2].span.column, 1);
    }

    /** Ensures every emitted span points back to the token's original text. */
    #[test]
    fn spans_reference_the_original_input() {
        let input = "[key=あ] C(9,#11)\r\nD/B";

        for token in lex(input) {
            let source = &input[token.span.start_byte..token.span.end_byte];
            match token.kind {
                TokenKind::Text(value) => assert_eq!(source, value),
                TokenKind::Newline => assert!(matches!(source, "\n" | "\r" | "\r\n")),
                _ => assert_eq!(source.chars().count(), 1),
            }
        }
    }

    /** Exercises dense delimiters without requiring a valid grammar. */
    #[test]
    fn lexes_malformed_delimiter_sequences_without_panicking() {
        let input = "@[]((=,,//--))]\nあ\r[";
        let result = std::panic::catch_unwind(|| lex(input));

        assert!(result.is_ok());
    }
}
