use std::ops::Range;

use crate::{Accidental, Base, ChordType, ErrorCode, Extension};

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

/** Matches the longest shorthand extension prefix in O(1) without re-parsing its spelling. */
fn match_shorthand_extension(suffix: &str) -> Option<(Extension, usize)> {
    match suffix.as_bytes() {
        [b'1', b'1', ..] => Some((Extension::Eleven, 2)),
        [b'1', b'3', ..] => Some((Extension::Thirteen, 2)),
        [b'1', ..] => Some((Extension::One, 1)),
        [b'2', ..] => Some((Extension::Two, 1)),
        [b'3', ..] => Some((Extension::Three, 1)),
        [b'4', ..] => Some((Extension::Four, 1)),
        [b'5', ..] => Some((Extension::Five, 1)),
        [b'6', ..] => Some((Extension::Six, 1)),
        [b'7', ..] => Some((Extension::Seven, 1)),
        [b'9', ..] => Some((Extension::Nine, 1)),
        [b'b', b'1', b'1', ..] => Some((Extension::FlatEleven, 3)),
        [b'b', b'1', b'3', ..] => Some((Extension::FlatThirteen, 3)),
        [b'b', b'3', ..] => Some((Extension::FlatThree, 2)),
        [b'b', b'5', ..] => Some((Extension::FlatFive, 2)),
        [b'b', b'6', ..] => Some((Extension::FlatSix, 2)),
        [b'b', b'9', ..] => Some((Extension::FlatNine, 2)),
        [b'#', b'1', b'1', ..] => Some((Extension::SharpEleven, 3)),
        [b'#', b'1', b'3', ..] => Some((Extension::SharpThirteen, 3)),
        [b'#', b'5', ..] => Some((Extension::SharpFive, 2)),
        [b'#', b'9', ..] => Some((Extension::SharpNine, 2)),
        [b'M', b'1', b'1', ..] => Some((Extension::MajorEleven, 3)),
        [b'M', b'1', b'3', ..] => Some((Extension::MajorThirteen, 3)),
        [b'M', b'7', ..] => Some((Extension::MajorSeven, 2)),
        [b'M', b'9', ..] => Some((Extension::MajorNine, 2)),
        [b'a', b'd', b'd', b'1', b'1', ..] => Some((Extension::Add11, 5)),
        [b'a', b'd', b'd', b'1', b'3', ..] => Some((Extension::Add13, 5)),
        [b'a', b'd', b'd', b'9', ..] => Some((Extension::Add9, 4)),
        [b's', b'u', b's', b'2', ..] => Some((Extension::Sus2, 4)),
        [b's', b'u', b's', b'4', ..] => Some((Extension::Sus4, 4)),
        [b'o', b'm', b'i', b't', b'5', ..] => Some((Extension::Omit5, 5)),
        _ => None,
    }
}

/** Reads an ASCII chord atom; parentheses and document separators belong to the outer parser. */
pub(super) fn parse_head(source: &str) -> Result<ParsedHead, ErrorCode> {
    let base = match source.as_bytes().first() {
        Some(b'A') => Base::A,
        Some(b'B') => Base::B,
        Some(b'C') => Base::C,
        Some(b'D') => Base::D,
        Some(b'E') => Base::E,
        Some(b'F') => Base::F,
        Some(b'G') => Base::G,
        _ => return Err(ErrorCode::Bs1),
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
    let (mut chord_type, type_length) = match source.as_bytes()[cursor..] {
        [b'm', ..] => (ChordType::Minor, 1),
        [b'a', b'u', b'g', ..] => (ChordType::Augmented, 3),
        [b'd', b'i', b'm', ..] => (ChordType::Diminished, 3),
        [b'M', b'7' | b'9', ..] | [b'M', b'1', b'1' | b'3', ..] => (ChordType::Major, 0),
        [b'M', ..] => (ChordType::Major, 1),
        _ => (ChordType::Major, 0),
    };
    cursor += type_length;

    let mut extensions = Vec::new();
    while cursor < source.len() {
        // Longest complete tokens distinguish 13 from 1 and leave unknown suffixes rejected.
        let (value, spelling_len) =
            match_shorthand_extension(&source[cursor..]).ok_or(ErrorCode::Cho1)?;
        let is_primary = value.is_primary();
        if is_primary && !extensions.is_empty() {
            return Err(ErrorCode::Cho1);
        }
        let end = cursor + spelling_len;
        extensions.push(HeadExtension {
            value,
            range: cursor..end,
        });
        cursor = end;
        // C7aug and Caug7 share the same quality and extensions. Only an untyped
        // head may place aug directly after its primary token; m/dim/aug cannot mix.
        if is_primary && type_length == 0 && source[cursor..].starts_with("aug") {
            chord_type = ChordType::Augmented;
            cursor += 3;
        }
    }

    Ok(ParsedHead {
        base,
        accidental,
        chord_type,
        extensions,
    })
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use strum::VariantNames;

    use super::match_shorthand_extension;
    use crate::Extension;

    /** Guards the O(1) prefix matcher against drifting from Extension::VARIANTS. */
    #[test]
    fn shorthand_matcher_agrees_with_extension_variants() {
        let reference = |suffix: &str| {
            Extension::VARIANTS
                .iter()
                .filter(|value| **value != "o" && suffix.starts_with(**value))
                .max_by_key(|value| value.len())
                .map(|spelling| {
                    (
                        Extension::from_str(spelling).expect("declared extension must parse"),
                        spelling.len(),
                    )
                })
        };

        for variant in Extension::VARIANTS {
            for trailing in ["", "1", "3", "5", "sus4", "x"] {
                let candidate = format!("{variant}{trailing}");
                assert_eq!(
                    match_shorthand_extension(&candidate),
                    reference(&candidate),
                    "shorthand matcher disagreed for {candidate:?}"
                );
            }
        }
        assert_eq!(match_shorthand_extension(""), None);
        assert_eq!(match_shorthand_extension("o"), None);
        assert_eq!(match_shorthand_extension("omit"), None);
        assert_eq!(match_shorthand_extension("sus"), None);
        assert_eq!(match_shorthand_extension("add7"), None);
    }
}
