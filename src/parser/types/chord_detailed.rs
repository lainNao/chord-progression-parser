use super::{accidental::Accidental, base::Base, chord_type::ChordType, extension::Extension};
use strum::VariantNames;

#[derive(Debug, PartialEq, Clone)]
pub struct ChordDetailed {
    pub base: Base,
    pub accidental: Option<Accidental>,
    pub chord_type: ChordType,
    pub extension: Option<Extension>,
    pub additional_extension: Option<Extension>,
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
            // No accidental
            _ => None,
        };

        let chord_str = &s[idx..];

        let chord_type = if chord_str.starts_with('m') {
            ChordType::Minor
        } else if chord_str.starts_with('M') {
            ChordType::Major
        } else if chord_str.starts_with("aug") {
            ChordType::Augmented
        } else if chord_str.starts_with("dim") {
            ChordType::Diminished
        } else {
            ChordType::Major
        };

        let maybe_other_str = chord_str.strip_prefix(match &chord_type {
            ChordType::Minor => "m",
            ChordType::Major => "M",
            ChordType::Augmented => "aug",
            ChordType::Diminished => "dim",
        });

        let other_str = match maybe_other_str {
            None => {
                return Ok(ChordDetailed {
                    base,
                    accidental,
                    chord_type,
                    extension: None,
                    additional_extension: None,
                });
            }
            Some(s) => s,
        };
        // let a = Extension::from_str("b3");

        let mut sorted_extensions = (*Extension::VARIANTS.clone()).to_vec();
        sorted_extensions.sort_by(|a, b| b.len().cmp(&a.len()));
        // let first_extension_str = sorted_extensions.iter().find(|e| other_str.starts_with(*e));
        let first_extension_str_result =
            sorted_extensions.iter().find(|e| other_str.starts_with(*e));
        if first_extension_str_result.is_none() {
            return Ok(ChordDetailed {
                base,
                accidental,
                chord_type,
                extension: None,
                additional_extension: None,
            });
        }

        // TODO 繰り返してる＆重複定義になってるのでリファクタ
        let first_extension_str = first_extension_str_result.unwrap();
        let first_extension: Option<Extension> = match *first_extension_str {
            "2" => Some(Extension::Two),
            "3" => Some(Extension::Three),
            "b3" => Some(Extension::FlatThree),
            "4" => Some(Extension::Four),
            "b5" => Some(Extension::FlatFive),
            "-5" => Some(Extension::FlatFive),
            "5" => Some(Extension::Five),
            "#5" => Some(Extension::SharpFive),
            "b6" => Some(Extension::FlatSix),
            "6" => Some(Extension::Six),
            "7" => Some(Extension::Seven),
            "b9" => Some(Extension::FlatNine),
            "9" => Some(Extension::Nine),
            "#9" => Some(Extension::SharpNine),
            "b11" => Some(Extension::FlatEleven),
            "11" => Some(Extension::Eleven),
            "#11" => Some(Extension::SharpEleven),
            "b13" => Some(Extension::FlatThirteen),
            "13" => Some(Extension::Thirteen),
            "#13" => Some(Extension::SharpThirteen),
            "add9" => Some(Extension::Add9),
            "add11" => Some(Extension::Add11),
            "add13" => Some(Extension::Add13),
            "sus2" => Some(Extension::Sus2),
            "sus4" => Some(Extension::Sus4),
            "o" => Some(Extension::HalfDiminish),
            _ => None,
        };

        let additional_extension_str = other_str.strip_prefix(first_extension_str);
        let additional_extension: Option<Extension> = match additional_extension_str {
            Some("2") => Some(Extension::Two),
            Some("3") => Some(Extension::Three),
            Some("b3") => Some(Extension::FlatThree),
            Some("4") => Some(Extension::Four),
            Some("b5") => Some(Extension::FlatFive),
            Some("-5") => Some(Extension::FlatFive),
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
            Some("add9") => Some(Extension::Add9),
            Some("add11") => Some(Extension::Add11),
            Some("add13") => Some(Extension::Add13),
            Some("sus2") => Some(Extension::Sus2),
            Some("sus4") => Some(Extension::Sus4),
            Some("o") => Some(Extension::HalfDiminish),
            _ => None,
        };

        Ok(ChordDetailed {
            base,
            accidental,
            chord_type,
            extension: first_extension,
            additional_extension,
        })
    }
}
