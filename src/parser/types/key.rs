use serde::{Deserialize, Serialize};
use strum_macros::EnumString;
use typeshare::typeshare;

#[typeshare]
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Clone, EnumString, Serialize, Deserialize)]
pub enum Key {
    #[strum(serialize = "Cb")]
    Cb_M, //C flat major
    #[strum(serialize = "Cbm")]
    Cb_m, //C flat minor
    #[strum(serialize = "C")]
    C_M, //C major
    #[strum(serialize = "Cm")]
    C_m, //C minor
    #[strum(serialize = "C#")]
    Cs_M, //C sharp major
    #[strum(serialize = "C#m")]
    Cs_m, //C sharp minor
    #[strum(serialize = "Db")]
    Db_M,
    #[strum(serialize = "Dbm")]
    Db_m,
    #[strum(serialize = "D")]
    D_M,
    #[strum(serialize = "Dm")]
    D_m,
    #[strum(serialize = "D#")]
    Ds_M,
    #[strum(serialize = "D#m")]
    Ds_m,
    #[strum(serialize = "Eb")]
    Eb_M,
    #[strum(serialize = "Ebm")]
    Eb_m,
    #[strum(serialize = "E")]
    E_M,
    #[strum(serialize = "Em")]
    E_m,
    #[strum(serialize = "E#")]
    Fb_M,
    #[strum(serialize = "E#m")]
    Fb_m,
    #[strum(serialize = "F")]
    F_M,
    #[strum(serialize = "Fm")]
    F_m,
    #[strum(serialize = "F#")]
    Fs_M,
    #[strum(serialize = "F#m")]
    Fs_m,
    #[strum(serialize = "Gb")]
    Gb_M,
    #[strum(serialize = "Gbm")]
    Gb_m,
    #[strum(serialize = "G")]
    G_M,
    #[strum(serialize = "Gm")]
    G_m,
    #[strum(serialize = "G#")]
    Gs_M,
    #[strum(serialize = "G#m")]
    Gs_m,
    #[strum(serialize = "Ab")]
    Ab_M,
    #[strum(serialize = "Abm")]
    Ab_m,
    #[strum(serialize = "A")]
    A_M,
    #[strum(serialize = "Am")]
    A_m,
    #[strum(serialize = "A#")]
    As_M,
    #[strum(serialize = "A#m")]
    As_m,
    #[strum(serialize = "Bb")]
    Bb_M,
    #[strum(serialize = "Bbm")]
    Bb_m,
    #[strum(serialize = "B")]
    B_M,
    #[strum(serialize = "Bm")]
    B_m,
}
