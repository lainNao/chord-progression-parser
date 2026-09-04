#[derive(Debug, PartialEq, Clone)]
pub struct Position {
    pub line_number: usize,
    pub column_number: usize,
    pub length: usize,
    /// Zero-based UTF-16 offset used by JavaScript text editors.
    pub start_offset: usize,
    /// Exclusive zero-based UTF-16 offset used by JavaScript text editors.
    pub end_offset: usize,
}
