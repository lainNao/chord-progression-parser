use std::{ops::Range, str::FromStr};

use strum::VariantNames;

use crate::{Accidental, Base, ChordType, ErrorCode, ErrorInfo, Extension};

/** Retains atom-relative ranges until the document parser assigns source positions. */
pub(super) struct HeadExtension {
    pub value: Extension,
    pub range: Range<usize>,
}

/** Separates the chord head from its source-ordered shorthand extensions. */
pub(super) struct ParsedHead {
    pub base: Base,
    pub accidental: Option<Accidental>,
    pub chord_type: ChordType,
    pub extensions: Vec<HeadExtension>,
}

/** Reads an ASCII chord atom; parentheses and document separators belong to the outer parser. */
pub(super) fn parse_head(source: &str) -> Result<ParsedHead, ErrorInfo> {
    let base = match source.as_bytes().first() {
        Some(b'A') => Base::A,
        Some(b'B') => Base::B,
        Some(b'C') => Base::C,
        Some(b'D') => Base::D,
        Some(b'E') => Base::E,
        Some(b'F') => Base::F,
        Some(b'G') => Base::G,
        _ => return Err(head_error(source, ErrorCode::Bs1)),
    };
    let mut cursor = 1;
    let accidental = match source.as_bytes().get(cursor) {
        Some(b'#') => {
            cursor += 1;
            Some(Accidental::Sharp)
        }
        Some(b'b') => {
            cursor += 1;
            Some(Accidental::Flat)
        }
        _ => None,
    };

    // M7/M9/M11/M13 are whole extension tokens, never a major marker followed by a number.
    let suffix = &source[cursor..];
    let (chord_type, type_length) = if suffix.starts_with('m') {
        (ChordType::Minor, 1)
    } else if suffix.starts_with("aug") {
        (ChordType::Augmented, 3)
    } else if suffix.starts_with("dim") {
        (ChordType::Diminished, 3)
    } else if suffix.starts_with('M')
        && !["M7", "M9", "M11", "M13"]
            .iter()
            .any(|value| suffix.starts_with(value))
    {
        (ChordType::Major, 1)
    } else {
        (ChordType::Major, 0)
    };
    cursor += type_length;

    let mut extensions = Vec::new();
    while cursor < source.len() {
        let suffix = &source[cursor..];
        // Longest complete tokens distinguish 13 from 1 and leave unknown suffixes rejected.
        let spelling = Extension::VARIANTS
            .iter()
            .filter(|value| **value != "o" && suffix.starts_with(**value))
            .max_by_key(|value| value.len())
            .ok_or_else(|| head_error(source, ErrorCode::Cho1))?;
        let value = Extension::from_str(spelling).expect("declared extension must parse");
        if value.is_primary() && !extensions.is_empty() {
            return Err(head_error(source, ErrorCode::Cho1));
        }
        let end = cursor + spelling.len();
        extensions.push(HeadExtension {
            value,
            range: cursor..end,
        });
        cursor = end;
    }

    Ok(ParsedHead {
        base,
        accidental,
        chord_type,
        extensions,
    })
}

/** Attaches the complete atom to chord-head errors without changing document error positions. */
fn head_error(source: &str, code: ErrorCode) -> ErrorInfo {
    ErrorInfo {
        code,
        additional_info: Some(source.to_owned()),
    }
}
