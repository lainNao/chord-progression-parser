use std::str::FromStr;

use crate::errors;
use typeshare::typeshare;

use super::{accidental::Accidental, base::Base, chord_type::ChordType, extension::Extension};
use serde::{Deserialize, Serialize};
use strum::VariantNames;

#[typeshare]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChordDetailed {
    pub base: Base,
    pub accidental: Option<Accidental>,
    pub chord_type: ChordType,
    pub extensions: Vec<Extension>,
}

fn try_remove_prefix(input: &str, prefix: &str) -> String {
    match input.strip_prefix(prefix) {
        Some(stripped) => String::from(stripped),
        None => String::from(input),
    }
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

        let chord_str_without_base = &s[idx..];

        let chord_type = if chord_str_without_base.starts_with('m') {
            ChordType::Minor
        } else if chord_str_without_base.starts_with('M') {
            ChordType::Major
        } else if chord_str_without_base.starts_with("aug") {
            ChordType::Augmented
        } else if chord_str_without_base.starts_with("dim") {
            ChordType::Diminished
        } else {
            ChordType::Major
        };

        let extensions_str_with_parenthesis = try_remove_prefix(
            chord_str_without_base,
            match chord_type {
                ChordType::Minor => "m",
                ChordType::Major => "M",
                ChordType::Augmented => "aug",
                ChordType::Diminished => "dim",
            },
        );

        if extensions_str_with_parenthesis.eq("") {
            return Ok(ChordDetailed {
                base,
                accidental,
                chord_type,
                extensions: vec![],
            });
        }

        if !extensions_str_with_parenthesis.starts_with('(')
            || !extensions_str_with_parenthesis.ends_with(')')
        {
            return Err([
                errors::EXTENSION_STRING_MUST_BE_SURROUNDED_BY_PARENTHESIS,
                &extensions_str_with_parenthesis.to_string(),
            ]
            .join(": "));
        }

        // strip surrounded parenthesis
        let extensions_str =
            &extensions_str_with_parenthesis[1..extensions_str_with_parenthesis.len() - 1];

        if extensions_str.eq("") {
            return Ok(ChordDetailed {
                base,
                accidental,
                chord_type,
                extensions: vec![],
            });
        }

        let mut sorted_extensions = (*Extension::VARIANTS.clone()).to_vec();
        sorted_extensions.sort_by_key(|b| std::cmp::Reverse(b.len()));

        let extensions_str_vec: Vec<&str> = extensions_str.split(',').collect();

        let mut parsed_extensions: Vec<Extension> = vec![];
        // loop extensions_str_vec
        for extension_str in extensions_str_vec.iter() {
            let extension_str_result = sorted_extensions
                .iter()
                .find(|e| extension_str.starts_with(**e));
            match extension_str_result {
                Some(extension_str_result) => {
                    parsed_extensions.push(Extension::from_str(extension_str_result).unwrap());
                }
                None => {
                    return Err([errors::INVALID_EXTENSION, extension_str].join(": "));
                }
            }
        }

        Ok(ChordDetailed {
            base,
            accidental,
            chord_type,
            extensions: parsed_extensions,
        })
    }
}

#[cfg(test)]
mod tests {

    #[cfg(test)]
    mod success {

        mod complex_chords {
            use std::str::FromStr;

            use strum::VariantNames;

            use crate::parser::types::{
                base::Base, chord_detailed::ChordDetailed, chord_type::ChordType,
                extension::Extension,
            };

            #[test]
            fn all_extension() {
                Extension::VARIANTS.iter().for_each(|extension_str| {
                    let chord_and_extension_str = format!("C({})", extension_str);
                    assert_eq!(
                        ChordDetailed::from_str(&chord_and_extension_str).unwrap(),
                        ChordDetailed {
                            base: Base::C,
                            accidental: None,
                            chord_type: ChordType::Major,
                            extensions: vec![Extension::from_str(extension_str).unwrap()],
                        }
                    );
                });
            }

            #[test]
            fn all_extension_pairs() {
                Extension::VARIANTS.iter().for_each(|extension_str1| {
                    Extension::VARIANTS.iter().for_each(|extension_str2| {
                        let chord_and_extension_str =
                            format!("C({},{})", extension_str1, extension_str2);
                        assert_eq!(
                            ChordDetailed::from_str(&chord_and_extension_str).unwrap(),
                            ChordDetailed {
                                base: Base::C,
                                accidental: None,
                                chord_type: ChordType::Major,
                                extensions: vec![
                                    Extension::from_str(extension_str1).unwrap(),
                                    Extension::from_str(extension_str2).unwrap()
                                ],
                            }
                        );
                    });
                });
            }
        }
    }
}
