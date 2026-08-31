use std::str::FromStr;

use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::error_code::{ErrorCode, ErrorInfo};

use super::{accidental::Accidental, base::Base, chord_type::ChordType, extension::Extension};

#[typeshare]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChordDetailed {
    pub base: Base,
    pub accidental: Option<Accidental>,
    pub chord_type: ChordType,
    pub extensions: Vec<Extension>,
}

impl ChordDetailed {
    /** Parses a chord head without its optional parenthesized extension list. */
    pub(crate) fn from_head(value: &str) -> Result<Self, ErrorInfo> {
        let Some(base_character) = value.chars().next() else {
            return Err(chord_error(ErrorCode::Bs1, None));
        };

        let base = match base_character {
            'A' => Base::A,
            'B' => Base::B,
            'C' => Base::C,
            'D' => Base::D,
            'E' => Base::E,
            'F' => Base::F,
            'G' => Base::G,
            _ => return Err(chord_error(ErrorCode::Bs1, Some(value.to_string()))),
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
            _ => return Err(chord_error(ErrorCode::Cho1, Some(value.to_string()))),
        };

        Ok(Self {
            base,
            accidental,
            chord_type,
            extensions: Vec::new(),
        })
    }
}

impl FromStr for ChordDetailed {
    type Err = ErrorInfo;

    /** Parses a complete chord using exact chord-type and extension matches. */
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let Some(opening_index) = value.find('(') else {
            if value.contains(')') {
                return Err(chord_error(ErrorCode::Ext3, Some(value.to_string())));
            }
            return Self::from_head(value);
        };

        if !value.ends_with(')') {
            return Err(chord_error(ErrorCode::Ext3, Some(value.to_string())));
        }

        let head = &value[..opening_index];
        let extension_source = &value[opening_index + 1..value.len() - 1];
        if extension_source
            .chars()
            .any(|character| matches!(character, '(' | ')'))
        {
            return Err(chord_error(ErrorCode::Ext4, Some(value.to_string())));
        }
        if extension_source.is_empty() {
            return Err(chord_error(ErrorCode::Ext2, None));
        }

        let mut detailed = Self::from_head(head)?;
        detailed.extensions = extension_source
            .split(',')
            .map(|extension| {
                if extension.is_empty() {
                    return Err(chord_error(ErrorCode::Ext2, None));
                }
                Extension::from_str(extension)
                    .map_err(|_| chord_error(ErrorCode::Ext1, Some(extension.to_string())))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(detailed)
    }
}

/** Creates a chord parsing error without assigning a source position. */
fn chord_error(code: ErrorCode, additional_info: Option<String>) -> ErrorInfo {
    ErrorInfo {
        code,
        additional_info,
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use strum::VariantNames;

    use super::ChordDetailed;
    use crate::model::{base::Base, chord_type::ChordType, extension::Extension};

    /** Parses every supported extension using the public FromStr contract. */
    #[test]
    fn parses_every_extension_exactly() {
        for extension_source in Extension::VARIANTS {
            let chord = ChordDetailed::from_str(&format!("C({extension_source})"))
                .expect("a declared extension must parse");

            assert_eq!(chord.base, Base::C);
            assert_eq!(chord.chord_type, ChordType::Major);
            assert_eq!(chord.extensions.len(), 1);
            assert_eq!(chord.extensions[0].to_string(), *extension_source);
        }
    }

    /** Rejects extension prefixes, empty lists, and multiple parenthesis groups. */
    #[test]
    fn rejects_ambiguous_extension_text() {
        for input in ["C(111)", "C()", "C(7)(9)", "C(7,)"] {
            assert!(
                ChordDetailed::from_str(input).is_err(),
                "unexpectedly accepted {input:?}"
            );
        }
    }

    /** Parses supported chord heads and rejects trailing text. */
    #[test]
    fn parses_chord_heads_by_complete_match() {
        assert_eq!(
            ChordDetailed::from_str("C#m")
                .expect("minor chord must parse")
                .chord_type,
            ChordType::Minor
        );
        assert!(ChordDetailed::from_str("Cminor").is_err());
    }
}
