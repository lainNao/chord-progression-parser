mod chord;

use std::str::FromStr;

use crate::{
    error_code::{ErrorCode, ErrorInfo, ErrorInfoWithPosition},
    lexer::{eof_span, lex, SourceSpan, Token, TokenKind},
    util::position::Position,
};

use self::chord::{parse_chord_head, ChordHeadError};
use super::types::{
    ast::Ast, bar::Bar, chord::Chord, chord_block::ChordBlock, chord_expression::ChordExpression,
    chord_info::ChordInfo, chord_info_meta::ChordInfoMeta, extension::Extension, key::Key,
    section::Section, section_meta::SectionMeta,
};

/** Parses source text with the context-free lexer and the new parser. */
pub(crate) fn parse(input: &str) -> Result<Ast, ErrorInfoWithPosition> {
    let tokens = lex(input);
    Parser::new(&tokens, eof_span(input)).parse_document()
}

/** Owns the token cursor and converts one grammar production at a time. */
struct Parser<'tokens, 'src> {
    tokens: &'tokens [Token<'src>],
    cursor: usize,
    eof_span: SourceSpan,
}

impl<'tokens, 'src> Parser<'tokens, 'src> {
    /** Creates a parser positioned before the first token. */
    fn new(tokens: &'tokens [Token<'src>], eof_span: SourceSpan) -> Self {
        Self {
            tokens,
            cursor: 0,
            eof_span,
        }
    }

    /** Parses all sections while interpreting line-break runs at document level. */
    fn parse_document(mut self) -> Result<Ast, ErrorInfoWithPosition> {
        let mut sections = Vec::new();
        let mut current_section = empty_section();
        let mut has_prior_chord = false;

        loop {
            let newline_count = self.consume_newlines();
            if self.is_at_end() {
                break;
            }

            let starts_section_meta = self.at(TokenKind::At);
            if !current_section.chord_blocks.is_empty()
                && (newline_count >= 2 || starts_section_meta)
            {
                sections.push(current_section);
                current_section = empty_section();
                has_prior_chord = false;
            } else if newline_count == 1 && !current_section.chord_blocks.is_empty() {
                current_section.chord_blocks.push(ChordBlock::Br);
            }

            if starts_section_meta {
                current_section.meta_infos.push(self.parse_section_meta()?);
            } else {
                current_section
                    .chord_blocks
                    .extend(self.parse_chord_line(&mut has_prior_chord)?);
            }
        }

        if !current_section.meta_infos.is_empty() || !current_section.chord_blocks.is_empty() {
            sections.push(current_section);
        }

        Ok(sections)
    }

    /** Parses a section metadata line and leaves its terminating newline untouched. */
    fn parse_section_meta(&mut self) -> Result<SectionMeta, ErrorInfoWithPosition> {
        self.expect_symbol(TokenKind::At, ErrorCode::Smik1)?;
        let (key, key_span) = self.expect_text(ErrorCode::Smik2)?;
        self.expect_symbol(TokenKind::Equal, ErrorCode::Smik2)?;
        let (value, value_span) = self.expect_text(ErrorCode::Smiv1)?;

        if !self.is_at_end() && !self.at(TokenKind::Newline) {
            return Err(parse_error(ErrorCode::Smiv2, self.current_span(), None));
        }

        match key {
            "section" => Ok(SectionMeta::Section(value.to_string())),
            "repeat" => value
                .parse::<u32>()
                .map(SectionMeta::Repeat)
                .map_err(|_| parse_error(ErrorCode::Smiv3, value_span, None)),
            _ => Err(parse_error(
                ErrorCode::Smik1,
                key_span,
                Some(key.to_string()),
            )),
        }
    }

    /** Parses one chord line as a sequence of bars separated by dashes. */
    fn parse_chord_line(
        &mut self,
        has_prior_chord: &mut bool,
    ) -> Result<Vec<ChordBlock>, ErrorInfoWithPosition> {
        let mut blocks = vec![ChordBlock::Bar(self.parse_bar(has_prior_chord)?)];

        while self.at(TokenKind::Dash) {
            let separator = self
                .advance()
                .ok_or_else(|| parse_error(ErrorCode::Cho3, self.eof_span, None))?;
            if self.is_at_end() || self.at(TokenKind::Newline) {
                return Err(parse_error(
                    ErrorCode::Cho3,
                    separator.span,
                    Some("-".to_string()),
                ));
            }
            blocks.push(ChordBlock::Bar(self.parse_bar(has_prior_chord)?));
        }

        if !self.is_at_end() && !self.at(TokenKind::Newline) {
            let token = self
                .peek()
                .copied()
                .ok_or_else(|| parse_error(ErrorCode::Tkn1, self.eof_span, None))?;
            return Err(parse_error(
                ErrorCode::Tkn1,
                token.span,
                Some(token_label(token.kind)),
            ));
        }

        Ok(blocks)
    }

    /** Parses one bar and groups comma-separated chord information. */
    fn parse_bar(&mut self, has_prior_chord: &mut bool) -> Result<Bar, ErrorInfoWithPosition> {
        let mut bar = vec![self.parse_chord_info(*has_prior_chord)?];
        *has_prior_chord = true;

        while self.at(TokenKind::Comma) {
            self.advance();
            if self.is_at_end() || self.at(TokenKind::Newline) || self.at(TokenKind::Dash) {
                break;
            }
            bar.push(self.parse_chord_info(*has_prior_chord)?);
            *has_prior_chord = true;
        }

        Ok(bar)
    }

    /** Parses metadata, expression, extension, and denominator for one chord. */
    fn parse_chord_info(
        &mut self,
        has_prior_chord: bool,
    ) -> Result<ChordInfo, ErrorInfoWithPosition> {
        let mut meta_infos = Vec::new();
        while self.at(TokenKind::LeftBracket) {
            meta_infos.push(self.parse_chord_meta()?);
        }

        let (head, head_span) = self.expect_text(ErrorCode::Cho3)?;
        let mut chord_expression = match head {
            "?" => ChordExpression::UnIdentified,
            "_" => ChordExpression::NoChord,
            "%" if has_prior_chord => ChordExpression::Same,
            "%" => return Err(parse_error(ErrorCode::Chb1, head_span, None)),
            _ => {
                let detailed = parse_chord_head(head).map_err(|error| {
                    let detail = match error {
                        ChordHeadError::InvalidBase => format!("BS-1: {head}"),
                        ChordHeadError::InvalidSuffix => format!("CHO-1: {head}"),
                    };
                    parse_error(ErrorCode::Cho1, head_span, Some(detail))
                })?;
                ChordExpression::Chord(Chord {
                    plain: head.to_string(),
                    detailed,
                })
            }
        };

        if self.at(TokenKind::LeftParen) {
            let extensions = self.parse_extensions()?;
            match &mut chord_expression {
                ChordExpression::Chord(chord) => {
                    chord.plain.push('(');
                    chord.plain.push_str(
                        &extensions
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join(","),
                    );
                    chord.plain.push(')');
                    chord.detailed.extensions = extensions;
                }
                _ => {
                    return Err(parse_error(ErrorCode::Ext3, head_span, None));
                }
            }
        }

        if self.at(TokenKind::LeftParen) {
            return Err(parse_error(ErrorCode::Ext4, self.current_span(), None));
        }

        let denominator = if self.at(TokenKind::Slash) {
            Some(self.parse_denominator()?)
        } else {
            None
        };

        Ok(ChordInfo {
            meta_infos,
            chord_expression,
            denominator,
        })
    }

    /** Parses one chord metadata expression and validates its key and value. */
    fn parse_chord_meta(&mut self) -> Result<ChordInfoMeta, ErrorInfoWithPosition> {
        self.expect_symbol(TokenKind::LeftBracket, ErrorCode::Cimk2)?;
        let (key, key_span) = self.expect_text(ErrorCode::Cimk2)?;
        self.expect_symbol(TokenKind::Equal, ErrorCode::Cimk1)?;
        let (value, value_span) = self.expect_text(ErrorCode::Cimv2)?;
        self.expect_symbol(TokenKind::RightBracket, ErrorCode::Cimv3)?;

        if key != "key" {
            return Err(parse_error(ErrorCode::Cimk3, key_span, None));
        }

        Key::from_str(value)
            .map(ChordInfoMeta::Key)
            .map_err(|_| parse_error(ErrorCode::Cimv4, value_span, None))
    }

    /** Parses a non-empty comma-separated extension list with exact matches. */
    fn parse_extensions(&mut self) -> Result<Vec<Extension>, ErrorInfoWithPosition> {
        let opening = self.expect_symbol(TokenKind::LeftParen, ErrorCode::Ext3)?;
        if self.at(TokenKind::RightParen) || self.is_at_end() {
            return Err(parse_error(ErrorCode::Ext2, opening.span, None));
        }

        let mut extensions = Vec::new();
        loop {
            let (value, span) = self.expect_text(ErrorCode::Ext2)?;
            let extension = Extension::from_str(value)
                .map_err(|_| parse_error(ErrorCode::Ext1, span, Some(value.to_string())))?;
            extensions.push(extension);

            if !self.at(TokenKind::Comma) {
                break;
            }
            let comma = self
                .advance()
                .ok_or_else(|| parse_error(ErrorCode::Ext2, self.eof_span, None))?;
            if self.at(TokenKind::RightParen) || self.is_at_end() {
                return Err(parse_error(ErrorCode::Ext2, comma.span, None));
            }
        }

        self.expect_symbol(TokenKind::RightParen, ErrorCode::Ext3)?;
        Ok(extensions)
    }

    /** Collects a denominator until the enclosing bar or line ends. */
    fn parse_denominator(&mut self) -> Result<String, ErrorInfoWithPosition> {
        let slash = self.expect_symbol(TokenKind::Slash, ErrorCode::Den1)?;
        let mut value = String::new();
        let mut parenthesis_depth = 0;

        while let Some(token) = self.peek().copied() {
            match token.kind {
                TokenKind::Newline | TokenKind::At | TokenKind::LeftBracket
                    if parenthesis_depth == 0 =>
                {
                    break;
                }
                TokenKind::Dash | TokenKind::Comma if parenthesis_depth == 0 => break,
                TokenKind::Slash => {
                    self.advance();
                    return Err(parse_error(ErrorCode::Den1, self.current_span(), None));
                }
                TokenKind::LeftParen => {
                    parenthesis_depth += 1;
                    value.push('(');
                }
                TokenKind::RightParen if parenthesis_depth > 0 => {
                    parenthesis_depth -= 1;
                    value.push(')');
                }
                TokenKind::RightParen => break,
                kind => value.push_str(&token_label(kind)),
            }
            self.advance();
        }

        if value.is_empty() || parenthesis_depth != 0 {
            return Err(parse_error(ErrorCode::Den1, slash.span, None));
        }

        Ok(value)
    }

    /** Consumes all consecutive newline tokens and returns their count. */
    fn consume_newlines(&mut self) -> usize {
        let mut count = 0;
        while self.at(TokenKind::Newline) {
            self.advance();
            count += 1;
        }
        count
    }

    /** Reads a text token or returns the requested syntax error. */
    fn expect_text(
        &mut self,
        code: ErrorCode,
    ) -> Result<(&'src str, SourceSpan), ErrorInfoWithPosition> {
        let token = self
            .advance()
            .ok_or_else(|| parse_error(code, self.eof_span, None))?;
        match token.kind {
            TokenKind::Text(value) => Ok((value, token.span)),
            _ => Err(parse_error(code, token.span, None)),
        }
    }

    /** Reads one expected structural token without assuming input completeness. */
    fn expect_symbol(
        &mut self,
        expected: TokenKind<'static>,
        code: ErrorCode,
    ) -> Result<Token<'src>, ErrorInfoWithPosition> {
        let token = self
            .advance()
            .ok_or_else(|| parse_error(code, self.eof_span, None))?;
        if token.kind == expected {
            Ok(token)
        } else {
            Err(parse_error(code, token.span, None))
        }
    }

    /** Returns whether the current token exactly matches a structural kind. */
    fn at(&self, expected: TokenKind<'static>) -> bool {
        self.peek().is_some_and(|token| token.kind == expected)
    }

    /** Returns the current token without advancing the cursor. */
    fn peek(&self) -> Option<&Token<'src>> {
        self.tokens.get(self.cursor)
    }

    /** Advances by one token and returns the consumed token. */
    fn advance(&mut self) -> Option<Token<'src>> {
        let token = self.peek().copied();
        if token.is_some() {
            self.cursor += 1;
        }
        token
    }

    /** Returns whether all tokens have been consumed. */
    fn is_at_end(&self) -> bool {
        self.cursor >= self.tokens.len()
    }

    /** Returns the current token span or a zero-length EOF span. */
    fn current_span(&self) -> SourceSpan {
        self.peek().map_or(self.eof_span, |token| token.span)
    }
}

/** Creates an empty section without hiding mutation behind a wrapper type. */
fn empty_section() -> Section {
    Section {
        meta_infos: Vec::new(),
        chord_blocks: Vec::new(),
    }
}

/** Converts an internal source span into the existing public error shape. */
fn parse_error(
    code: ErrorCode,
    span: SourceSpan,
    additional_info: Option<String>,
) -> ErrorInfoWithPosition {
    ErrorInfoWithPosition {
        error: ErrorInfo {
            code,
            additional_info,
        },
        position: Position {
            line_number: span.line,
            column_number: span.column,
            length: span.length,
        },
    }
}

/** Returns the source spelling of a context-free token for errors and denominators. */
fn token_label(kind: TokenKind<'_>) -> String {
    match kind {
        TokenKind::At => "@",
        TokenKind::LeftBracket => "[",
        TokenKind::RightBracket => "]",
        TokenKind::LeftParen => "(",
        TokenKind::RightParen => ")",
        TokenKind::Equal => "=",
        TokenKind::Comma => ",",
        TokenKind::Slash => "/",
        TokenKind::Dash => "-",
        TokenKind::Newline => "\n",
        TokenKind::Text(value) => value,
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::parse;

    /** Matches the legacy AST for the checked-in public contract. */
    #[test]
    fn matches_the_public_contract_ast() {
        let input = include_str!("../../tests/fixtures/public_contract.chord");
        let legacy = crate::parse_chord_progression_string(input)
            .expect("the legacy parser must accept the contract fixture");
        let current = parse(input).expect("the new parser must accept the contract fixture");

        assert_eq!(json!(current), json!(legacy));
    }

    /** Supports full chord syntax in a denominator without parsing its semantics. */
    #[test]
    fn preserves_a_structured_looking_denominator_as_text() {
        let ast =
            parse("F#m(7,b5)/F#m(7,b5)").expect("a chord-shaped denominator remains supported");

        assert_eq!(
            json!(ast)[0]["chordBlocks"][0]["value"][0]["denominator"],
            json!("F#m(7,b5)")
        );
    }

    /** Accepts extra blank lines as a section boundary instead of BL-1. */
    #[test]
    fn accepts_multiple_blank_lines_between_sections() {
        let ast = parse("C\n\n\n\nD").expect("extra blank lines must be harmless");

        assert_eq!(ast.len(), 2);
    }

    /** Rejects metadata placed after the chord it would otherwise be detached from. */
    #[test]
    fn rejects_chord_metadata_after_a_chord() {
        let error = parse("C[key=A]").expect_err("postfix metadata must not be ignored");

        assert_eq!(error.error.code.to_string(), "TKN-1");
        assert_eq!(error.position.column_number, 2);
    }

    /** Converts all known truncated inputs into errors without panicking. */
    #[test]
    fn rejects_the_malformed_input_corpus_without_panicking() {
        for input in include_str!("../../tests/fixtures/malformed_inputs.txt").lines() {
            let result = std::panic::catch_unwind(|| parse(input));
            assert!(result.is_ok(), "parser panicked for {input:?}");
            assert!(
                result.expect("panic was checked").is_err(),
                "accepted {input:?}"
            );
        }
    }

    /** Reports an exact extension token rather than accepting its prefix. */
    #[test]
    fn rejects_partial_extension_matches() {
        let error = parse("C(9,111)").expect_err("111 is not a known extension");

        assert_eq!(error.error.code.to_string(), "EXT-1");
        assert_eq!(error.position.line_number, 1);
        assert_eq!(error.position.column_number, 5);
        assert_eq!(error.position.length, 3);
    }
}
