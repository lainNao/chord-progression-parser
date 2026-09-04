use std::str::FromStr;

use crate::{
    error_code::{ErrorCode, ErrorInfo, ErrorInfoWithPosition},
    lexer::{eof_span, lex, SourceSpan, Token, TokenKind},
    model::{
        ast::Ast, bar::Bar, chord::Chord, chord_block::ChordBlock, chord_detailed::ChordDetailed,
        chord_expression::ChordExpression, chord_info::ChordInfo, chord_info_meta::ChordInfoMeta,
        extension::Extension, key::Key, section::Section, section_meta::SectionMeta,
    },
    util::position::Position,
};

/** Parses source text with the context-free lexer and the new parser. */
pub(crate) fn parse(input: &str) -> Result<Ast, Vec<ErrorInfoWithPosition>> {
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
    fn parse_document(mut self) -> Result<Ast, Vec<ErrorInfoWithPosition>> {
        let mut sections = Vec::new();
        let mut current_section = empty_section();
        let mut has_prior_chord = false;
        let mut errors = Vec::new();

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
                match self.parse_section_meta() {
                    Ok(meta) => current_section.meta_infos.push(meta),
                    Err(meta_errors) => {
                        errors.extend(meta_errors);
                        self.skip_to_line_end();
                    }
                }
            } else {
                loop {
                    match self.parse_chord_line(&mut has_prior_chord) {
                        Ok(blocks) => {
                            current_section.chord_blocks.extend(blocks);
                            break;
                        }
                        Err(line_errors) => errors.extend(line_errors),
                    }

                    self.skip_to_bar_or_line_end();
                    if !self.at(TokenKind::Dash) && !self.at(TokenKind::Comma) {
                        break;
                    }

                    let separator = self.advance().expect("separator was checked above");
                    if self.is_at_end() || self.at(TokenKind::Newline) {
                        if separator.kind == TokenKind::Dash {
                            errors.push(parse_error(
                                ErrorCode::Cho3,
                                separator.span,
                                Some("-".to_string()),
                            ));
                        }
                        break;
                    }
                }
            }
        }

        if !current_section.meta_infos.is_empty() || !current_section.chord_blocks.is_empty() {
            sections.push(current_section);
        }

        if errors.is_empty() {
            Ok(sections)
        } else {
            Err(errors)
        }
    }

    /** Parses a section metadata line and leaves its terminating newline untouched. */
    fn parse_section_meta(&mut self) -> Result<SectionMeta, Vec<ErrorInfoWithPosition>> {
        self.expect_symbol(TokenKind::At, ErrorCode::Smik1)
            .map_err(|error| vec![error])?;
        let (key, key_span) = self
            .expect_text(ErrorCode::Smik2)
            .map_err(|error| vec![error])?;
        self.expect_symbol(TokenKind::Equal, ErrorCode::Smik2)
            .map_err(|error| vec![error])?;

        let mut errors = Vec::new();
        if !matches!(key, "section" | "repeat") {
            errors.push(parse_error(
                ErrorCode::Smik1,
                key_span,
                Some(key.to_string()),
            ));
        }

        let value = if key == "repeat" && self.at(TokenKind::Dash) {
            let dash = self.advance().expect("dash was checked above");
            let span = match self.peek().copied() {
                Some(token) if matches!(token.kind, TokenKind::Text(_)) => {
                    self.advance();
                    span_through(dash.span, token.span)
                }
                _ => dash.span,
            };
            errors.push(parse_error(ErrorCode::Smiv3, span, None));
            None
        } else {
            match self.expect_text(ErrorCode::Smiv1) {
                Ok(value) => Some(value),
                Err(error) => {
                    errors.push(error);
                    None
                }
            }
        };

        let meta = match (key, value) {
            ("section", Some((value, _))) => Some(SectionMeta::Section(value.to_string())),
            ("repeat", Some((value, value_span))) => match value.parse::<u32>() {
                Ok(repeat) => Some(SectionMeta::Repeat(repeat)),
                Err(_) => {
                    errors.push(parse_error(ErrorCode::Smiv3, value_span, None));
                    None
                }
            },
            _ => None,
        };

        if value.is_some() && !self.is_at_end() && !self.at(TokenKind::Newline) {
            errors.push(parse_error(ErrorCode::Smiv2, self.current_span(), None));
        }

        if errors.is_empty() {
            Ok(meta.expect("known metadata with a parsed value must produce a result"))
        } else {
            Err(errors)
        }
    }

    /** Parses one chord line as a sequence of bars separated by dashes. */
    fn parse_chord_line(
        &mut self,
        has_prior_chord: &mut bool,
    ) -> Result<Vec<ChordBlock>, Vec<ErrorInfoWithPosition>> {
        let mut blocks = vec![ChordBlock::Bar(self.parse_bar(has_prior_chord)?)];

        while self.at(TokenKind::Dash) {
            let separator = self
                .advance()
                .ok_or_else(|| vec![parse_error(ErrorCode::Cho3, self.eof_span, None)])?;
            if self.is_at_end() || self.at(TokenKind::Newline) {
                return Err(vec![parse_error(
                    ErrorCode::Cho3,
                    separator.span,
                    Some("-".to_string()),
                )]);
            }
            blocks.push(ChordBlock::Bar(self.parse_bar(has_prior_chord)?));
        }

        if !self.is_at_end() && !self.at(TokenKind::Newline) {
            let token = self
                .peek()
                .copied()
                .ok_or_else(|| vec![parse_error(ErrorCode::Tkn1, self.eof_span, None)])?;
            return Err(vec![parse_error(
                ErrorCode::Tkn1,
                token.span,
                Some(token_label(token.kind)),
            )]);
        }

        Ok(blocks)
    }

    /** Parses one bar and groups comma-separated chord information. */
    fn parse_bar(&mut self, has_prior_chord: &mut bool) -> Result<Bar, Vec<ErrorInfoWithPosition>> {
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
    ) -> Result<ChordInfo, Vec<ErrorInfoWithPosition>> {
        let mut meta_infos = Vec::new();
        while self.at(TokenKind::LeftBracket) {
            meta_infos.push(self.parse_chord_meta().map_err(|error| vec![error])?);
        }

        let (head, head_span) = self
            .expect_text(ErrorCode::Cho3)
            .map_err(|error| vec![error])?;
        let mut chord_expression = match head {
            "?" => ChordExpression::UnIdentified,
            "_" => ChordExpression::NoChord,
            "%" if has_prior_chord => ChordExpression::Same,
            "%" => return Err(vec![parse_error(ErrorCode::Chb1, head_span, None)]),
            _ => {
                let detailed = ChordDetailed::from_head(head).map_err(|error| {
                    vec![parse_error(
                        ErrorCode::Cho1,
                        head_span,
                        Some(format!("{}: {head}", error.code)),
                    )]
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
                    return Err(vec![parse_error(ErrorCode::Ext3, head_span, None)]);
                }
            }
        }

        if self.at(TokenKind::LeftParen) {
            return Err(vec![parse_error(
                ErrorCode::Ext4,
                self.current_span(),
                None,
            )]);
        }

        let denominator = if self.at(TokenKind::Slash) {
            Some(self.parse_denominator().map_err(|error| vec![error])?)
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
    fn parse_extensions(&mut self) -> Result<Vec<Extension>, Vec<ErrorInfoWithPosition>> {
        let opening = self
            .expect_symbol(TokenKind::LeftParen, ErrorCode::Ext3)
            .map_err(|error| vec![error])?;
        let mut extensions = Vec::new();
        let mut errors = Vec::new();
        let mut is_first_value = true;

        loop {
            if self.at(TokenKind::RightParen) || self.is_at_end() {
                if is_first_value {
                    errors.push(parse_error(ErrorCode::Ext2, opening.span, None));
                }
                break;
            }

            if self.at(TokenKind::Comma) {
                let comma = self.advance().expect("comma was checked above");
                errors.push(parse_error(ErrorCode::Ext2, comma.span, None));
                is_first_value = false;
                continue;
            }

            match self.expect_text(ErrorCode::Ext2) {
                Ok((value, span)) => match Extension::from_str(value) {
                    Ok(extension) => extensions.push(extension),
                    Err(_) => {
                        errors.push(parse_error(ErrorCode::Ext1, span, Some(value.to_string())))
                    }
                },
                Err(error) => {
                    errors.push(error);
                    self.skip_to_extension_delimiter();
                }
            }
            is_first_value = false;

            if !self.at(TokenKind::Comma) {
                break;
            }

            let comma = self.advance().expect("comma was checked above");
            if self.at(TokenKind::RightParen) || self.is_at_end() {
                errors.push(parse_error(ErrorCode::Ext2, comma.span, None));
                break;
            }
        }

        if let Err(error) = self.expect_symbol(TokenKind::RightParen, ErrorCode::Ext3) {
            errors.push(error);
        }

        if errors.is_empty() {
            Ok(extensions)
        } else {
            Err(errors)
        }
    }

    /** Collects a denominator until the enclosing bar or line ends. */
    fn parse_denominator(&mut self) -> Result<String, ErrorInfoWithPosition> {
        let slash = self.expect_symbol(TokenKind::Slash, ErrorCode::Den1)?;
        let mut value = String::new();
        let mut parenthesis_depth = 0;
        let mut last_span = slash.span;

        while let Some(token) = self.peek().copied() {
            match token.kind {
                TokenKind::Newline => break,
                TokenKind::At | TokenKind::LeftBracket if parenthesis_depth == 0 => {
                    break;
                }
                TokenKind::Dash | TokenKind::Comma if parenthesis_depth == 0 => break,
                TokenKind::Slash => {
                    self.advance();
                    return Err(parse_error(ErrorCode::Den2, token.span, None));
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
            last_span = token.span;
            self.advance();
        }

        if value.is_empty() {
            return Err(parse_error(ErrorCode::Den1, slash.span, None));
        }
        if parenthesis_depth != 0 {
            return Err(parse_error(
                ErrorCode::Den1,
                span_through(slash.span, last_span),
                None,
            ));
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

    /** Skips invalid input until another chord or bar can be parsed, or the line ends. */
    fn skip_to_bar_or_line_end(&mut self) {
        while !self.is_at_end()
            && !self.at(TokenKind::Comma)
            && !self.at(TokenKind::Dash)
            && !self.at(TokenKind::Newline)
        {
            self.advance();
        }
    }

    /** Skips the remainder of a metadata line after a diagnostic. */
    fn skip_to_line_end(&mut self) {
        while !self.is_at_end() && !self.at(TokenKind::Newline) {
            self.advance();
        }
    }

    /** Skips an invalid extension fragment without leaving its parenthesized list. */
    fn skip_to_extension_delimiter(&mut self) {
        while !self.is_at_end()
            && !self.at(TokenKind::Comma)
            && !self.at(TokenKind::RightParen)
            && !self.at(TokenKind::Newline)
        {
            self.advance();
        }
    }

    /** Reads a text token or returns the requested syntax error. */
    fn expect_text(
        &mut self,
        code: ErrorCode,
    ) -> Result<(&'src str, SourceSpan), ErrorInfoWithPosition> {
        let token = self
            .peek()
            .copied()
            .ok_or_else(|| parse_error(code, self.eof_span, None))?;
        match token.kind {
            TokenKind::Text(value) => {
                self.advance();
                Ok((value, token.span))
            }
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
            .peek()
            .copied()
            .ok_or_else(|| parse_error(code, self.eof_span, None))?;
        if token.kind == expected {
            self.advance();
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

/** Joins two same-line spans so a compound invalid value is highlighted together. */
fn span_through(start: SourceSpan, end: SourceSpan) -> SourceSpan {
    SourceSpan {
        start_byte: start.start_byte,
        end_byte: end.end_byte,
        start_offset: start.start_offset,
        end_offset: end.end_offset,
        line: start.line,
        column: start.column,
        length: end
            .column
            .saturating_add(end.length)
            .saturating_sub(start.column),
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
            start_offset: span.start_offset,
            end_offset: span.end_offset,
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
    use serde_json::{json, Value};

    use super::parse;

    /** Advances a deterministic pseudo-random state without adding a runtime dependency. */
    fn next_random(state: &mut u64) -> u64 {
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;
        *state
    }

    /** Counts display characters on each line using the lexer's CRLF rules. */
    fn source_line_lengths(input: &str) -> Vec<usize> {
        let mut lengths = vec![0];
        let mut characters = input.chars().peekable();

        while let Some(character) = characters.next() {
            match character {
                '\r' => {
                    if characters.peek().is_some_and(|next| *next == '\n') {
                        characters.next();
                    }
                    lengths.push(0);
                }
                '\n' => lengths.push(0),
                _ => {
                    if let Some(length) = lengths.last_mut() {
                        *length += 1;
                    }
                }
            }
        }

        lengths
    }

    /** Matches the checked-in public JSON contract. */
    #[test]
    fn matches_the_public_contract_ast() {
        let input = include_str!("../../tests/fixtures/public_contract.chord");
        let expected: Value =
            serde_json::from_str(include_str!("../../tests/fixtures/public_contract.json"))
                .expect("the checked-in contract fixture must be valid JSON");
        let ast = parse(input).expect("the public contract fixture must parse");

        assert_eq!(json!({ "success": true, "ast": ast }), expected);
    }

    /** Parses documented syntax and representative line structures. */
    #[test]
    fn parses_representative_valid_documents() {
        let inputs = [
            "",
            "C",
            " C \t-\t Dm(7) ",
            "C,D-E\nF",
            "C/D,",
            "C/D,E",
            "C\n%",
            "[key=C][key=G]C",
            "@section=A",
            "@section=A\n@repeat=3\n\nC-D",
            "C\n\nD",
            "C(9, 11, #13)",
            "F#m(7,b5)/F#m(7,b5)-Fbm/G7",
            "?-_-%",
        ];

        for input in inputs {
            parse(input).unwrap_or_else(|error| {
                panic!("parser rejected representative input {input:?}: {error:?}")
            });
        }
    }

    /** Parses supported chord heads and extensions as a generated valid corpus. */
    #[test]
    fn parses_generated_chord_combinations() {
        let bases = ["A", "B", "C", "D", "E", "F", "G"];
        let accidentals = ["", "#", "b"];
        let chord_types = ["", "M", "m", "aug", "dim"];
        let extensions = ["2", "b5", "7", "M9", "#11", "add13", "sus4", "o"];

        for base in bases {
            for accidental in accidentals {
                for chord_type in chord_types {
                    for extension in extensions {
                        let input = format!("{base}{accidental}{chord_type}({extension})");
                        parse(&input).unwrap_or_else(|error| {
                            panic!("parser rejected generated chord {input:?}: {error:?}")
                        });
                    }
                }
            }
        }
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
        let errors = parse("C[key=A]").expect_err("postfix metadata must not be ignored");
        let error = &errors[0];

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

    /** Keeps representative source inputs mapped to every parser-reachable error code. */
    #[test]
    fn maps_representative_failures_to_specific_error_codes() {
        let cases = [
            ("@unknown=x", "SMIK-1"),
            ("@section", "SMIK-2"),
            ("@section=", "SMIV-1"),
            ("@section=A extra", "SMIV-2"),
            ("@repeat=nope", "SMIV-3"),
            ("[key C]C", "CIMK-1"),
            ("[]C", "CIMK-2"),
            ("[bad=C]C", "CIMK-3"),
            ("[key=]C", "CIMV-2"),
            ("[key=C", "CIMV-3"),
            ("[key=H]C", "CIMV-4"),
            ("%", "CHB-1"),
            ("H", "CHO-1"),
            ("-", "CHO-3"),
            ("C/", "DEN-1"),
            ("C//G", "DEN-2"),
            ("C(111)", "EXT-1"),
            ("C()", "EXT-2"),
            ("C(7", "EXT-3"),
            ("C(7)(9)", "EXT-4"),
            ("C]", "TKN-1"),
        ];

        for (input, expected_code) in cases {
            let errors = parse(input).expect_err("the representative input must be invalid");
            assert_eq!(
                errors[0].error.code.to_string(),
                expected_code,
                "unexpected error code for {input:?}"
            );
        }
    }

    /** Reports an exact extension token rather than accepting its prefix. */
    #[test]
    fn rejects_partial_extension_matches() {
        let errors = parse("C(9,111)").expect_err("111 is not a known extension");
        let error = &errors[0];

        assert_eq!(error.error.code.to_string(), "EXT-1");
        assert_eq!(error.position.line_number, 1);
        assert_eq!(error.position.column_number, 5);
        assert_eq!(error.position.length, 3);
        assert_eq!(error.position.start_offset, 4);
        assert_eq!(error.position.end_offset, 7);
    }

    /** Recovers at bar and line boundaries to report independent diagnostics together. */
    #[test]
    fn reports_multiple_errors_in_source_order() {
        let errors = parse("H-C(111)-I\n@repeat=nope\nJ")
            .expect_err("each invalid source region must be reported");
        let diagnostics: Vec<_> = errors
            .iter()
            .map(|error| {
                (
                    error.error.code.to_string(),
                    error.position.line_number,
                    error.position.column_number,
                    error.position.length,
                )
            })
            .collect();

        assert_eq!(
            diagnostics,
            vec![
                ("CHO-1".to_string(), 1, 1, 1),
                ("EXT-1".to_string(), 1, 5, 3),
                ("CHO-1".to_string(), 1, 10, 1),
                ("SMIV-3".to_string(), 2, 9, 4),
                ("CHO-1".to_string(), 3, 1, 1),
            ]
        );
    }

    /** Keeps collecting diagnostics inside comma-separated chords and extensions. */
    #[test]
    fn reports_multiple_errors_within_a_bar_and_extension_list() {
        let errors = parse("H,I-C(111,222)")
            .expect_err("invalid comma-separated values must all be reported");
        let diagnostics: Vec<_> = errors
            .iter()
            .map(|error| {
                (
                    error.error.code.to_string(),
                    error.position.start_offset,
                    error.position.end_offset,
                )
            })
            .collect();

        assert_eq!(
            diagnostics,
            vec![
                ("CHO-1".to_string(), 0, 1),
                ("CHO-1".to_string(), 2, 3),
                ("EXT-1".to_string(), 6, 9),
                ("EXT-1".to_string(), 10, 13),
            ]
        );
    }

    /** Keeps empty and invalid extension slots within the extension diagnostic domain. */
    #[test]
    fn reports_multiple_malformed_extension_slots() {
        let cases = [
            ("C(,111)", vec![("EXT-2", 3, 1), ("EXT-1", 4, 3)]),
            (
                "C(111,,222)",
                vec![("EXT-1", 3, 3), ("EXT-2", 7, 1), ("EXT-1", 8, 3)],
            ),
            ("C(,,)", vec![("EXT-2", 3, 1), ("EXT-2", 4, 1)]),
            ("C(/,111)", vec![("EXT-2", 3, 1), ("EXT-1", 5, 3)]),
        ];

        for (input, expected) in cases {
            let errors = parse(input).expect_err("malformed extensions must be rejected");
            let diagnostics: Vec<_> = errors
                .iter()
                .map(|error| {
                    (
                        error.error.code.to_string(),
                        error.position.column_number,
                        error.position.length,
                    )
                })
                .collect();
            let expected: Vec<_> = expected
                .into_iter()
                .map(|(code, column, length)| (code.to_string(), column, length))
                .collect();

            assert_eq!(
                diagnostics, expected,
                "unexpected diagnostics for {input:?}"
            );
        }
    }

    /** Continues with the next line when truncated section metadata ends at a newline. */
    #[test]
    fn reports_errors_after_truncated_section_metadata() {
        let errors = parse("@section\nH\n@repeat=nope\nI")
            .expect_err("metadata and chord errors on later lines must all be reported");
        let diagnostics: Vec<_> = errors
            .iter()
            .map(|error| {
                (
                    error.error.code.to_string(),
                    error.position.line_number,
                    error.position.column_number,
                    error.position.start_offset,
                    error.position.end_offset,
                )
            })
            .collect();

        assert_eq!(
            diagnostics,
            vec![
                ("SMIK-2".to_string(), 1, 9, 8, 9),
                ("CHO-1".to_string(), 2, 1, 9, 10),
                ("SMIV-3".to_string(), 3, 9, 19, 23),
                ("CHO-1".to_string(), 4, 1, 24, 25),
            ]
        );
    }

    /** Reports every supported section metadata failure without swallowing later lines. */
    #[test]
    fn reports_section_metadata_boundary_errors() {
        let errors = parse("@unknown=x\n@section=\n@repeat=nope\n@section=Ok extra\nH")
            .expect_err("each malformed metadata line must produce a diagnostic");
        let diagnostics: Vec<_> = errors
            .iter()
            .map(|error| {
                (
                    error.error.code.to_string(),
                    error.position.line_number,
                    error.position.column_number,
                    error.position.length,
                )
            })
            .collect();

        assert_eq!(
            diagnostics,
            vec![
                ("SMIK-1".to_string(), 1, 2, 7),
                ("SMIV-1".to_string(), 2, 10, 1),
                ("SMIV-3".to_string(), 3, 9, 4),
                ("SMIV-2".to_string(), 4, 13, 5),
                ("CHO-1".to_string(), 5, 1, 1),
            ]
        );
    }

    /** Reports independent key, value, and trailing-token errors on metadata lines. */
    #[test]
    fn reports_multiple_errors_within_section_metadata() {
        let errors = parse("@unknown=x extra\n@repeat=nope extra\nH")
            .expect_err("each independent metadata issue must be reported");
        let diagnostics: Vec<_> = errors
            .iter()
            .map(|error| {
                (
                    error.error.code.to_string(),
                    error.position.line_number,
                    error.position.column_number,
                    error.position.length,
                )
            })
            .collect();

        assert_eq!(
            diagnostics,
            vec![
                ("SMIK-1".to_string(), 1, 2, 7),
                ("SMIV-2".to_string(), 1, 12, 5),
                ("SMIV-3".to_string(), 2, 9, 4),
                ("SMIV-2".to_string(), 2, 14, 5),
                ("CHO-1".to_string(), 3, 1, 1),
            ]
        );
    }

    /** Accepts the inclusive unsigned repeat boundaries. */
    #[test]
    fn accepts_repeat_integer_boundaries() {
        parse("@repeat=0\nC").expect("zero repeats must remain valid");
        parse("@repeat=4294967295\nC").expect("u32::MAX repeats must remain valid");
    }

    /** Rejects negative and overflowing repeat values with their complete ranges. */
    #[test]
    fn reports_repeat_integer_boundary_errors() {
        let errors = parse("@repeat=-1\nH\n@repeat=4294967296")
            .expect_err("out-of-range repeat values must be rejected");
        let diagnostics: Vec<_> = errors
            .iter()
            .map(|error| {
                (
                    error.error.code.to_string(),
                    error.position.line_number,
                    error.position.column_number,
                    error.position.length,
                )
            })
            .collect();

        assert_eq!(
            diagnostics,
            vec![
                ("SMIV-3".to_string(), 1, 9, 2),
                ("CHO-1".to_string(), 2, 1, 1),
                ("SMIV-3".to_string(), 3, 9, 10),
            ]
        );

        let trailing_minus = parse("@repeat=-").expect_err("a bare minus must be rejected");
        assert_eq!(trailing_minus[0].error.code.to_string(), "SMIV-3");
        assert_eq!(trailing_minus[0].position.column_number, 9);
        assert_eq!(trailing_minus[0].position.length, 1);
    }

    /** Rejects extension lists attached to non-chord expressions. */
    #[test]
    fn rejects_extensions_on_non_chord_expressions() {
        for input in ["?(7)", "_(7)"] {
            let errors = parse(input).expect_err("non-chords cannot have extensions");

            assert_eq!(errors.len(), 1, "unexpected diagnostics for {input:?}");
            assert_eq!(errors[0].error.code.to_string(), "EXT-3");
            assert_eq!(errors[0].position.column_number, 1);
        }
    }

    /** Recovers from chord metadata errors at both bar and line boundaries. */
    #[test]
    fn reports_chord_metadata_errors_across_lines() {
        let errors = parse("[key=H]C-[bad=C]D\n[key=G]I")
            .expect_err("invalid chord metadata and later chords must all be reported");
        let diagnostics: Vec<_> = errors
            .iter()
            .map(|error| {
                (
                    error.error.code.to_string(),
                    error.position.line_number,
                    error.position.column_number,
                    error.position.length,
                )
            })
            .collect();

        assert_eq!(
            diagnostics,
            vec![
                ("CIMV-4".to_string(), 1, 6, 1),
                ("CIMK-3".to_string(), 1, 11, 3),
                ("CHO-1".to_string(), 2, 8, 1),
            ]
        );
    }

    /** Treats CRLF as one line break while retaining both UTF-16 code units. */
    #[test]
    fn reports_multiple_errors_with_crlf_offsets() {
        let errors = parse("H\r\nI\r\n@repeat=x")
            .expect_err("CRLF-separated invalid lines must all be reported");
        let positions: Vec<_> = errors
            .iter()
            .map(|error| {
                (
                    error.position.line_number,
                    error.position.start_offset,
                    error.position.end_offset,
                )
            })
            .collect();

        assert_eq!(positions, vec![(1, 0, 1), (2, 3, 4), (3, 14, 15)]);
    }

    /** Reports leading, repeated, and trailing bar separators independently. */
    #[test]
    fn reports_errors_around_repeated_bar_separators() {
        let errors = parse("-H--I-").expect_err("every malformed bar region must be reported");
        let diagnostics: Vec<_> = errors
            .iter()
            .map(|error| (error.error.code.to_string(), error.position.column_number))
            .collect();

        assert_eq!(
            diagnostics,
            vec![
                ("CHO-3".to_string(), 1),
                ("CHO-1".to_string(), 2),
                ("CHO-3".to_string(), 4),
                ("CHO-1".to_string(), 5),
                ("CHO-3".to_string(), 6),
            ]
        );
    }

    /** Collects empty extensions, invalid extensions, and denominator failures by line. */
    #[test]
    fn reports_delimiter_and_eof_boundary_errors() {
        let errors = parse("C()\nD(111,)\nE//G\nF/\n[key=")
            .expect_err("delimiter and EOF failures must all be reported");
        let diagnostics: Vec<_> = errors
            .iter()
            .map(|error| {
                (
                    error.error.code.to_string(),
                    error.position.line_number,
                    error.position.column_number,
                    error.position.length,
                )
            })
            .collect();

        assert_eq!(
            diagnostics,
            vec![
                ("EXT-2".to_string(), 1, 2, 1),
                ("EXT-1".to_string(), 2, 3, 3),
                ("EXT-2".to_string(), 2, 6, 1),
                ("DEN-2".to_string(), 3, 3, 1),
                ("DEN-1".to_string(), 4, 2, 1),
                ("CIMV-2".to_string(), 5, 6, 0),
            ]
        );

        let eof = errors.last().expect("EOF error was asserted above");
        assert_eq!(eof.position.start_offset, eof.position.end_offset);
    }

    /** Highlights the complete unclosed denominator instead of only its slash. */
    #[test]
    fn reports_unclosed_denominator_ranges() {
        let errors = parse("C/F#m(7\nD/(G")
            .expect_err("each denominator with an unclosed parenthesis must be rejected");
        let positions: Vec<_> = errors
            .iter()
            .map(|error| {
                (
                    error.error.code.to_string(),
                    error.position.line_number,
                    error.position.column_number,
                    error.position.length,
                )
            })
            .collect();

        assert_eq!(
            positions,
            vec![
                ("DEN-1".to_string(), 1, 2, 6),
                ("DEN-1".to_string(), 2, 2, 3),
            ]
        );
    }

    /** Keeps reporting after multiple blank lines that create a section boundary. */
    #[test]
    fn reports_errors_across_blank_section_boundaries() {
        let errors = parse("H\n\n\nI").expect_err("both invalid sections must be reported");

        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].position.line_number, 1);
        assert_eq!(errors[1].position.line_number, 4);
        assert_eq!(errors[1].position.start_offset, 4);
    }

    /** Retains the full range of a large invalid token without truncation. */
    #[test]
    fn reports_a_large_invalid_token_range() {
        let input = "H".repeat(4096);
        let errors = parse(&input).expect_err("a large invalid chord must be rejected");

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].position.length, 4096);
        assert_eq!(errors[0].position.start_offset, 0);
        assert_eq!(errors[0].position.end_offset, 4096);
    }

    /** Returns every diagnostic from a document with many malformed lines. */
    #[test]
    fn reports_many_invalid_lines_without_stopping_early() {
        const LINE_COUNT: usize = 512;
        let input = vec!["H"; LINE_COUNT].join("\n");
        let errors = parse(&input).expect_err("every generated line is invalid");

        assert_eq!(errors.len(), LINE_COUNT);
        assert_eq!(errors[0].position.line_number, 1);
        assert_eq!(errors[LINE_COUNT - 1].position.line_number, LINE_COUNT);
        assert_eq!(errors[LINE_COUNT - 1].position.end_offset, input.len());
    }

    /** Reports editor offsets as UTF-16 code units rather than UTF-8 bytes. */
    #[test]
    fn reports_utf16_offsets_for_javascript_editors() {
        let errors = parse("😀\nH").expect_err("the emoji and invalid chord must be rejected");

        assert_eq!(errors[0].position.start_offset, 0);
        assert_eq!(errors[0].position.end_offset, 2);
        assert_eq!(errors[1].position.start_offset, 3);
        assert_eq!(errors[1].position.end_offset, 4);
    }

    /** Exercises arbitrary delimiter and Unicode mixtures without panics or invalid positions. */
    #[test]
    fn handles_deterministic_adversarial_inputs() {
        let alphabet = [
            'A', 'C', 'm', '9', '#', 'b', '@', '[', ']', '(', ')', '=', ',', '/', '-', '?', '%',
            '_', ' ', '\t', '\n', '\r', 'あ', '♭', '|',
        ];
        let mut state = 0x4d59_5df4_d0f3_3173;

        for _ in 0..10_000 {
            let length = (next_random(&mut state) % 40) as usize;
            let mut input = String::new();
            for _ in 0..length {
                let index = (next_random(&mut state) % alphabet.len() as u64) as usize;
                input.push(alphabet[index]);
            }

            let result = std::panic::catch_unwind(|| parse(&input));
            let parsed = result.unwrap_or_else(|_| panic!("parser panicked for {input:?}"));
            if let Err(errors) = parsed {
                let line_lengths = source_line_lengths(&input);
                let source_length = input.encode_utf16().count();
                let mut previous_offset = 0;
                assert!(!errors.is_empty(), "empty error list for {input:?}");

                for error in errors {
                    assert!(
                        (1..=line_lengths.len()).contains(&error.position.line_number),
                        "invalid error line for {input:?}: {error:?}"
                    );

                    let line_length = line_lengths[error.position.line_number - 1];
                    assert!(
                        (1..=line_length + 1).contains(&error.position.column_number),
                        "invalid error column for {input:?}: {error:?}"
                    );
                    assert!(
                        error.position.start_offset <= error.position.end_offset,
                        "reversed error range for {input:?}: {error:?}"
                    );
                    assert!(
                        error.position.end_offset <= source_length,
                        "out-of-bounds editor range for {input:?}: {error:?}"
                    );
                    assert!(
                        previous_offset <= error.position.start_offset,
                        "out-of-order error range for {input:?}: {error:?}"
                    );
                    previous_offset = error.position.start_offset;
                }
            }
        }
    }
}
