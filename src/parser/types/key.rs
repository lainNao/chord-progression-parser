use serde::{Deserialize, Serialize};
use strum_macros::EnumString;
use typeshare::typeshare;

#[typeshare]
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Clone, EnumString, Serialize, Deserialize)]
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
    Fb_M,
    #[strum(serialize = "E#m")]
    #[serde(rename = "E#m")]
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
}
