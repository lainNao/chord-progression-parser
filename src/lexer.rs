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

/** The UTF-16 range and display position; text tokens already borrow their UTF-8 source. */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SourceSpan {
    pub(crate) start_offset: usize,
    pub(crate) end_offset: usize,
    pub(crate) line: usize,
    pub(crate) column: usize,
    pub(crate) length: usize,
}

impl From<SourceSpan> for crate::Position {
    /** Uses the same display position and UTF-16 offsets for errors and warnings. */
    fn from(span: SourceSpan) -> Self {
        Self {
            line_number: span.line,
            column_number: span.column,
            length: span.length,
            start_offset: span.start_offset,
            end_offset: span.end_offset,
        }
    }
}

/** A token paired with the exact source range from which it was read. */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Token<'src> {
    pub(crate) kind: TokenKind<'src>,
    pub(crate) span: SourceSpan,
}

/** Retains the final lexer coordinates so EOF does not require another source scan. */
pub(crate) struct LexedSource<'src> {
    pub(crate) tokens: Vec<Token<'src>>,
    pub(crate) eof_span: SourceSpan,
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
pub(crate) fn lex(input: &str) -> LexedSource<'_> {
    let mut tokens = Vec::new();
    let mut chars = input.char_indices().peekable();
    let mut line = 1;
    let mut column = 1;
    let mut offset = 0;

    while let Some((start_byte, ch)) = chars.next() {
        match ch {
            ' ' | '\t' => {
                column += 1;
                offset += ch.len_utf16();
            }
            '\r' | '\n' => {
                let start_line = line;
                let start_column = column;
                let mut length = 1;
                let start_offset = offset;
                offset += ch.len_utf16();

                if ch == '\r' && chars.peek().is_some_and(|(_, next)| *next == '\n') {
                    if let Some((_, next)) = chars.next() {
                        length += 1;
                        offset += next.len_utf16();
                    }
                }

                tokens.push(Token {
                    kind: TokenKind::Newline,
                    span: SourceSpan {
                        start_offset,
                        end_offset: offset,
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
                    let start_offset = offset;
                    offset += ch.len_utf16();
                    tokens.push(Token {
                        kind,
                        span: SourceSpan {
                            start_offset,
                            end_offset: offset,
                            line,
                            column,
                            length: 1,
                        },
                    });
                    column += 1;
                    continue;
                }

                let start_column = column;
                let start_offset = offset;
                let mut end_byte = start_byte + ch.len_utf8();
                let mut length = 1;
                column += 1;
                offset += ch.len_utf16();

                while let Some((next_byte, next)) = chars.peek().copied() {
                    if ends_text(next) {
                        break;
                    }

                    chars.next();
                    end_byte = next_byte + next.len_utf8();
                    length += 1;
                    column += 1;
                    offset += next.len_utf16();
                }

                tokens.push(Token {
                    kind: TokenKind::Text(&input[start_byte..end_byte]),
                    span: SourceSpan {
                        start_offset,
                        end_offset: offset,
                        line,
                        column: start_column,
                        length,
                    },
                });
            }
        }
    }

    LexedSource {
        tokens,
        eof_span: SourceSpan {
            start_offset: offset,
            end_offset: offset,
            line,
            column,
            length: 0,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{lex, SourceSpan, Token, TokenKind};

    /** Verifies structural tokens, text tokens, and skipped horizontal space. */
    #[test]
    fn lexes_structure_without_assigning_context() {
        let input = " @section = Verse\n[key=C]C(M9),G/B-D ";
        let kinds: Vec<TokenKind<'_>> = lex(input).tokens.iter().map(|token| token.kind).collect();

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

    /** Verifies Unicode display columns and UTF-16 offsets independently. */
    #[test]
    fn tracks_unicode_text_by_character_and_utf16() {
        let input = "Cあ\nD";

        assert_eq!(
            lex(input).tokens,
            vec![
                Token {
                    kind: TokenKind::Text("Cあ"),
                    span: SourceSpan {
                        start_offset: 0,
                        end_offset: 2,
                        line: 1,
                        column: 1,
                        length: 2,
                    },
                },
                Token {
                    kind: TokenKind::Newline,
                    span: SourceSpan {
                        start_offset: 2,
                        end_offset: 3,
                        line: 1,
                        column: 3,
                        length: 1,
                    },
                },
                Token {
                    kind: TokenKind::Text("D"),
                    span: SourceSpan {
                        start_offset: 3,
                        end_offset: 4,
                        line: 2,
                        column: 1,
                        length: 1,
                    },
                },
            ]
        );
    }

    /** Counts skipped horizontal whitespace in JavaScript editor offsets. */
    #[test]
    fn tracks_offsets_across_horizontal_whitespace() {
        let tokens = lex("C - G\t- Fあ").tokens;
        let last = tokens.last().expect("the final text token must exist");

        assert_eq!(last.kind, TokenKind::Text("Fあ"));
        assert_eq!(last.span.column, 9);
        assert_eq!(last.span.start_offset, 8);
        assert_eq!(last.span.end_offset, 10);
    }

    /** Treats CRLF as one newline while retaining both source characters. */
    #[test]
    fn treats_crlf_as_one_line_break() {
        let tokens = lex("C\r\nD").tokens;

        assert_eq!(tokens[1].kind, TokenKind::Newline);
        assert_eq!(tokens[1].span.start_offset, 1);
        assert_eq!(tokens[1].span.end_offset, 3);
        assert_eq!(tokens[1].span.length, 2);
        assert_eq!(tokens[2].span.line, 2);
        assert_eq!(tokens[2].span.column, 1);
    }

    /** Ensures every emitted span points back to the token's original text. */
    #[test]
    fn spans_reference_the_original_input() {
        let input = "[key=あ😀] C(9,#11)\r\nD/B";
        let utf16: Vec<u16> = input.encode_utf16().collect();

        for token in lex(input).tokens {
            let source = String::from_utf16(&utf16[token.span.start_offset..token.span.end_offset])
                .expect("token ranges must contain whole Unicode characters");
            match token.kind {
                TokenKind::Text(value) => assert_eq!(source, value),
                TokenKind::Newline => assert!(matches!(source.as_str(), "\n" | "\r" | "\r\n")),
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

    /** Places EOF after skipped space and CRLF using display coordinates. */
    #[test]
    fn tracks_the_end_of_the_source() {
        assert_eq!(
            lex("Cあ \r\nD ").eof_span,
            SourceSpan {
                start_offset: 7,
                end_offset: 7,
                line: 2,
                column: 3,
                length: 0,
            }
        );
    }

    /** Preserves EOF display coordinates and UTF-16 offsets after all newline forms. */
    #[test]
    fn tracks_eof_after_unicode_and_trailing_whitespace() {
        for (input, line, column, offset) in [
            ("", 1, 1, 0),
            (" \t", 1, 3, 2),
            ("😀 ", 1, 3, 3),
            ("C\r", 2, 1, 2),
            ("C\r\n😀\t", 2, 3, 6),
            ("\r\n\r\n", 3, 1, 4),
        ] {
            assert_eq!(
                lex(input).eof_span,
                SourceSpan {
                    start_offset: offset,
                    end_offset: offset,
                    line,
                    column,
                    length: 0,
                },
                "incorrect EOF position for {input:?}"
            );
        }
    }
}
