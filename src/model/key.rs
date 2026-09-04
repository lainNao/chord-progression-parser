use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use typeshare::typeshare;

#[typeshare]
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Clone, Display, EnumString, Serialize, Deserialize)]
pub enum Key {
    #[strum(serialize = "Cb")]
    #[serde(rename = "Cb")]
    Cb_M, //C flat major
    #[strum(serialize = "Cbm")]
    #[serde(rename = "Cbm")]
    Cb_m, //C flat minor
    #[strum(serialize = "C")]
    #[serde(rename = "C")]
    C_M, //C major
    #[strum(serialize = "Cm")]
    #[serde(rename = "Cm")]
    C_m, //C minor
    #[strum(serialize = "C#")]
    #[serde(rename = "C#")]
    Cs_M, //C sharp major
    #[strum(serialize = "C#m")]
    #[serde(rename = "C#m")]
    Cs_m, //C sharp minor
    #[strum(serialize = "Db")]
    #[serde(rename = "Db")]
    Db_M,
    #[strum(serialize = "Dbm")]
    #[serde(rename = "Dbm")]
    Db_m,
    #[strum(serialize = "D")]
    #[serde(rename = "D")]
    D_M,
    #[strum(serialize = "Dm")]
    #[serde(rename = "Dm")]
    D_m,
    #[strum(serialize = "D#")]
    #[serde(rename = "D#")]
    Ds_M,
    #[strum(serialize = "D#m")]
    #[serde(rename = "D#m")]
    Ds_m,
    #[strum(serialize = "Eb")]
    #[serde(rename = "Eb")]
    Eb_M,
    #[strum(serialize = "Ebm")]
    #[serde(rename = "Ebm")]
    Eb_m,
    #[strum(serialize = "E")]
    #[serde(rename = "E")]
    E_M,
    #[strum(serialize = "Em")]
    #[serde(rename = "Em")]
    E_m,
    #[strum(serialize = "E#")]
    #[serde(rename = "E#")]
    Es_M,
    #[strum(serialize = "E#m")]
    #[serde(rename = "E#m")]
    Es_m,
    #[strum(serialize = "Fb")]
    #[serde(rename = "Fb")]
    Fb_M,
    #[strum(serialize = "Fbm")]
    #[serde(rename = "Fbm")]
    Fb_m,
    #[strum(serialize = "F")]
    #[serde(rename = "F")]
    F_M,
    #[strum(serialize = "Fm")]
    #[serde(rename = "Fm")]
    F_m,
    #[strum(serialize = "F#")]
    #[serde(rename = "F#")]
    Fs_M,
    #[strum(serialize = "F#m")]
    #[serde(rename = "F#m")]
    Fs_m,
    #[strum(serialize = "Gb")]
    #[serde(rename = "Gb")]
    Gb_M,
    #[strum(serialize = "Gbm")]
    #[serde(rename = "Gbm")]
    Gb_m,
    #[strum(serialize = "G")]
    #[serde(rename = "G")]
    G_M,
    #[strum(serialize = "Gm")]
    #[serde(rename = "Gm")]
    G_m,
    #[strum(serialize = "G#")]
    #[serde(rename = "G#")]
    Gs_M,
    #[strum(serialize = "G#m")]
    #[serde(rename = "G#m")]
    Gs_m,
    #[strum(serialize = "Ab")]
    #[serde(rename = "Ab")]
    Ab_M,
    #[strum(serialize = "Abm")]
    #[serde(rename = "Abm")]
    Ab_m,
    #[strum(serialize = "A")]
    #[serde(rename = "A")]
    A_M,
    #[strum(serialize = "Am")]
    #[serde(rename = "Am")]
    A_m,
    #[strum(serialize = "A#")]
    #[serde(rename = "A#")]
    As_M,
    #[strum(serialize = "A#m")]
    #[serde(rename = "A#m")]
    As_m,
    #[strum(serialize = "Bb")]
    #[serde(rename = "Bb")]
    Bb_M,
    #[strum(serialize = "Bbm")]
    #[serde(rename = "Bbm")]
    Bb_m,
    #[strum(serialize = "B")]
    #[serde(rename = "B")]
    B_M,
    #[strum(serialize = "Bm")]
    #[serde(rename = "Bm")]
    B_m,
    #[strum(serialize = "?")]
    #[serde(rename = "?")]
    UnIdentified,
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::Key;

    /** Keeps F-flat and E-sharp distinct across parsing, display, and JSON. */
    #[test]
    fn preserves_enharmonic_key_spellings() {
        let cases = [
            ("Fb", Key::Fb_M),
            ("Fbm", Key::Fb_m),
            ("E#", Key::Es_M),
            ("E#m", Key::Es_m),
        ];

        for (source, expected) in cases {
            let parsed = Key::from_str(source).expect("supported key must parse");

            assert_eq!(parsed, expected);
            assert_eq!(parsed.to_string(), source);
            assert_eq!(
                serde_json::to_string(&parsed).expect("key must serialize"),
                format!("\"{source}\"")
            );
            assert_eq!(
                serde_json::from_str::<Key>(&format!("\"{source}\""))
                    .expect("key must deserialize"),
                parsed
            );
        }
    }

    /** Parses and round-trips every key spelling supported by the public grammar. */
    #[test]
    fn round_trips_every_supported_key_spelling() {
        let spellings = [
            "Cb", "Cbm", "C", "Cm", "C#", "C#m", "Db", "Dbm", "D", "Dm", "D#", "D#m", "Eb", "Ebm",
            "E", "Em", "E#", "E#m", "Fb", "Fbm", "F", "Fm", "F#", "F#m", "Gb", "Gbm", "G", "Gm",
            "G#", "G#m", "Ab", "Abm", "A", "Am", "A#", "A#m", "Bb", "Bbm", "B", "Bm", "?",
        ];

        for spelling in spellings {
            let parsed = Key::from_str(spelling)
                .unwrap_or_else(|error| panic!("supported key {spelling:?} failed: {error:?}"));
            let serialized = serde_json::to_string(&parsed).expect("supported key must serialize");
            let deserialized = serde_json::from_str::<Key>(&serialized)
                .expect("serialized supported key must deserialize");

            assert_eq!(parsed.to_string(), spelling);
            assert_eq!(deserialized, parsed);
        }
    }
}
