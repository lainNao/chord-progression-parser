use crate::util::position::Position;

pub fn is_token_char(ch: char) -> bool {
    matches!(
        ch,
        '\n' | '\r' | '@' | '[' | ']' | '(' | ')' | '|' | '=' | '/' | ',' | '\t'
    )
}

pub fn next_char_with_position<I>(
    iter: &mut std::iter::Peekable<I>,
    line: &mut usize,
    column: &mut usize,
) -> Option<(char, Position)>
where
    I: Iterator<Item = char>,
{
    match iter.next() {
        Some('\n') => {
            let position = Position {
                line_number: *line,
                column_number: *column,
            };
            *line += 1;
            *column = 1;
            Some(('\n', position))
        }
        Some(ch) => {
            let position = Position {
                line_number: *line,
                column_number: *column,
            };
            *column += 1;
            Some((ch, position))
        }
        None => None,
    }
}
