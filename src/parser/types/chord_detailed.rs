use super::{accidental::Accidental, base::Base, chord_type::ChordType, extension::Extension};

#[derive(Debug, PartialEq, Clone)]
pub struct ChordDetailed {
    pub base: Base,
    pub accidental: Option<Accidental>,
    pub chord_type: ChordType,
    pub extension: Option<Extension>,
}
impl ChordDetailed {
    pub fn from_str(s: &str) -> Result<Self, String> {
        let base = match s.chars().next() {
            Some('A') => Base::A,
            Some('B') => Base::B,
            Some('C') => Base::C,
            Some('D') => Base::D,
            Some('E') => Base::E,
            Some('F') => Base::F,
            Some('G') => Base::G,
            _ => return Err("Invalid base.".to_string()),
        };

        let mut idx = 1; // Start after the base note

        let accidental = match s.chars().nth(idx) {
            Some('#') => {
                idx += 1;
                Some(Accidental::Sharp)
            }
            Some('b') => {
                idx += 1;
                Some(Accidental::Flat)
            }
            _ => return Err("Invalid accidental.".to_string()),
        };

        let chord_str = &s[idx..];

        let chord_type = if chord_str.starts_with("m") {
            ChordType::Minor
        } else if chord_str.starts_with("M") {
            ChordType::Major
        } else if chord_str.starts_with("aug") {
            ChordType::Augmented
        } else if chord_str.starts_with("dim") {
            ChordType::Diminished
        } else if chord_str.starts_with("add") {
            ChordType::Add
        } else if chord_str.starts_with("sus2") {
            ChordType::Sus2
        } else if chord_str.starts_with("sus4") {
            ChordType::Sus4
        } else {
            return Err("Invalid chord type.".to_string());
        };

        let extension_str = chord_str.strip_prefix(match &chord_type {
            ChordType::Minor => "m",
            ChordType::Major => "M",
            ChordType::Augmented => "aug",
            ChordType::Diminished => "dim",
            ChordType::Add => "add",
            ChordType::Sus2 => "sus2",
            ChordType::Sus4 => "sus4",
        });

        let extension = match extension_str {
            Some("2") => Some(Extension::Two),
            Some("3") => Some(Extension::Three),
            Some("b3") => Some(Extension::FlatThree),
            Some("4") => Some(Extension::Four),
            Some("b5") => Some(Extension::FlatFive),
            Some("-5") => Some(Extension::MinusFive),
            Some("5") => Some(Extension::Five),
            Some("#5") => Some(Extension::SharpFive),
            Some("b6") => Some(Extension::FlatSix),
            Some("6") => Some(Extension::Six),
            Some("7") => Some(Extension::Seven),
            Some("b9") => Some(Extension::FlatNine),
            Some("9") => Some(Extension::Nine),
            Some("#9") => Some(Extension::SharpNine),
            Some("b11") => Some(Extension::FlatEleven),
            Some("11") => Some(Extension::Eleven),
            Some("#11") => Some(Extension::SharpEleven),
            Some("b13") => Some(Extension::FlatThirteen),
            Some("13") => Some(Extension::Thirteen),
            Some("#13") => Some(Extension::SharpThirteen),
            _ => return Err("Invalid extension.".to_string()),
        };

        Ok(ChordDetailed {
            base,
            accidental,
            chord_type,
            extension,
        })
    }
}
