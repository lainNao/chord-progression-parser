#[derive(Debug, PartialEq, Clone)]
pub struct Position {
    pub line_number: usize,
    pub column_number: usize,
    pub length: usize,
}
