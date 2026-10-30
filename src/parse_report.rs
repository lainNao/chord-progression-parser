use crate::{Ast, ErrorInfoWithPosition, Position, WarningCode};

/** Describes a notation warning at its original source position. */
#[derive(Debug, PartialEq)]
pub struct ParseWarning {
    pub code: WarningCode,
    pub additional_info: Option<String>,
    pub position: Position,
}

/** Connects one AST metadata value to its exact position in the original input. */
#[derive(Debug, PartialEq)]
pub struct SectionMetaPosition {
    pub section_index: usize,
    pub meta_index: usize,
    pub position: Position,
}

/** Keeps warnings available whether parsing succeeds or produces errors. */
#[derive(Debug, PartialEq)]
pub struct ParseReport {
    pub result: Result<Ast, Vec<ErrorInfoWithPosition>>,
    pub warnings: Vec<ParseWarning>,
    pub section_meta_positions: Vec<SectionMetaPosition>,
}
