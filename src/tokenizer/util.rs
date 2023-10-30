pub fn is_token_char(ch: char) -> bool {
    matches!(
        ch,
        '\n' | '\r' | '@' | '[' | ']' | '(' | ')' | '|' | '=' | '/' | ',' | '\t'
    )
}
