use std::str::FromStr;

use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::error_code::ErrorInfo;

use super::{accidental::Accidental, base::Base, chord_type::ChordType, extension::Extension};

#[typeshare]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChordDetailed {
    pub base: Base,
    pub accidental: Option<Accidental>,
    pub chord_type: ChordType,
    /** Extensions in fixed sort_order order, including repeated values. */
    #[cfg_attr(
        target_arch = "wasm32",
        serde(deserialize_with = "crate::wasm_ast::deserialize_array")
    )]
    pub extensions: Vec<Extension>,
}

impl FromStr for ChordDetailed {
    type Err = ErrorInfo;

    /** Uses the same complete chord grammar and extension ordering as the document parser. */
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        crate::parser::parse_chord_symbol(value).map(|chord| chord.detailed)
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
        for input in ["C)", "C(7", "C(111)", "C()", "C(7)(9)", "C(7,)"] {
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
