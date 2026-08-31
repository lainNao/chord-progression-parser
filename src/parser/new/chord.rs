use super::super::types::{
    accidental::Accidental, base::Base, chord_detailed::ChordDetailed, chord_type::ChordType,
};

/** Describes why a chord head could not be interpreted. */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ChordHeadError {
    InvalidBase,
    InvalidSuffix,
}

/** Parses the portion of a chord before its optional extension list. */
pub(super) fn parse_chord_head(value: &str) -> Result<ChordDetailed, ChordHeadError> {
    let Some(base_character) = value.chars().next() else {
        return Err(ChordHeadError::InvalidBase);
    };

    let base = match base_character {
        'A' => Base::A,
        'B' => Base::B,
        'C' => Base::C,
        'D' => Base::D,
        'E' => Base::E,
        'F' => Base::F,
        'G' => Base::G,
        _ => return Err(ChordHeadError::InvalidBase),
    };

    let remainder = &value[base_character.len_utf8()..];
    let (accidental, remainder) = match remainder.chars().next() {
        Some('#') => (Some(Accidental::Sharp), &remainder[1..]),
        Some('b') => (Some(Accidental::Flat), &remainder[1..]),
        _ => (None, remainder),
    };

    let chord_type = match remainder {
        "" | "M" => ChordType::Major,
        "m" => ChordType::Minor,
        "aug" => ChordType::Augmented,
        "dim" => ChordType::Diminished,
        _ => return Err(ChordHeadError::InvalidSuffix),
    };

    Ok(ChordDetailed {
        base,
        accidental,
        chord_type,
        extensions: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::{parse_chord_head, ChordHeadError};
    use crate::parser::types::{accidental::Accidental, base::Base, chord_type::ChordType};

    /** Parses every supported chord type without relying on prefix matches. */
    #[test]
    fn parses_supported_chord_heads() {
        let cases = [
            ("C", Base::C, None, ChordType::Major),
            ("CM", Base::C, None, ChordType::Major),
            ("C#m", Base::C, Some(Accidental::Sharp), ChordType::Minor),
            (
                "Dbaug",
                Base::D,
                Some(Accidental::Flat),
                ChordType::Augmented,
            ),
            ("Bdim", Base::B, None, ChordType::Diminished),
        ];

        for (input, base, accidental, chord_type) in cases {
            let detailed = parse_chord_head(input).expect("the chord head must parse");
            assert_eq!(detailed.base, base);
            assert_eq!(detailed.accidental, accidental);
            assert_eq!(detailed.chord_type, chord_type);
            assert!(detailed.extensions.is_empty());
        }
    }

    /** Rejects unknown bases and trailing chord text completely. */
    #[test]
    fn rejects_partial_or_unknown_chord_heads() {
        assert_eq!(parse_chord_head("H"), Err(ChordHeadError::InvalidBase));
        assert_eq!(
            parse_chord_head("Cminor"),
            Err(ChordHeadError::InvalidSuffix)
        );
    }
}
