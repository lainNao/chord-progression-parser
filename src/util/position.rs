/** Locates a diagnostic using display coordinates and JavaScript-compatible source offsets. */
#[derive(Debug, PartialEq, Clone)]
pub struct Position {
    /// One-based source line; CRLF counts as one line break.
    pub line_number: usize,
    /// One-based Unicode scalar column; a tab counts as one scalar, not a tab stop.
    pub column_number: usize,
    /// Number of Unicode scalar values covered by the diagnostic; zero at EOF.
    pub length: usize,
    /// Zero-based UTF-16 offset used by JavaScript text editors.
    pub start_offset: usize,
    /// Exclusive zero-based UTF-16 offset used by JavaScript text editors.
    pub end_offset: usize,
}
