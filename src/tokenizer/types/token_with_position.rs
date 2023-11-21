use super::super::super::util::position::Position;
use super::token::Token;

#[derive(Debug, PartialEq, Clone)]
pub struct TokenWithPosition {
    pub token: Token,
    pub position: Position,
}
