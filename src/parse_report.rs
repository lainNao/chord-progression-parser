use strum_macros::Display;

use crate::{Ast, ErrorInfoWithPosition, Position};

/** Identifies non-blocking notation warnings separately from error codes. */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum WarningCode {
    #[strum(serialize = "DUPLICATE_EXTENSION")]
    DuplicateExtension,
}

/** Describes a notation warning at its original source position. */
#[derive(Debug, PartialEq)]
pub struct ParseWarning {
    pub code: WarningCode,
    pub additional_info: Option<String>,
    pub position: Position,
}

/** Keeps warnings available whether parsing succeeds or produces errors. */
#[derive(Debug, PartialEq)]
pub struct ParseReport {
    pub result: Result<Ast, Vec<ErrorInfoWithPosition>>,
    pub warnings: Vec<ParseWarning>,
}
