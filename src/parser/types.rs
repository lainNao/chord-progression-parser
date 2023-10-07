use strum_macros::Display;
use strum_macros::EnumString;

pub type Ast = Vec<Section>;

#[derive(Debug, PartialEq, Clone)]
pub struct Section {
    pub meta_infos: Vec<SectionMeta>,
    pub chord_blocks: Vec<ChordBlock>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SectionMeta {
    Section { value: String },
    Repeat { value: u32 },
    // or more
}

pub type ChordBlock = Vec<ChordInfo>;

#[derive(Debug, PartialEq, Clone)]
pub struct ChordInfo {
    pub meta_infos: Vec<ChordInfoMeta>,
    pub chord: Chord,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ChordInfoMeta {
    Key { value: Key },
    // or more
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Clone, EnumString)]
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

#[derive(Debug, PartialEq, Clone)]
pub struct Chord {
    pub plain: String,
    pub detailed: ChordDetailed,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ChordDetailed {
    pub base: Base,
    pub accidental: Option<Accidental>,
    pub chord_type: ChordType,
    pub extension: Option<Extension>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Base {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Accidental {
    Sharp,
    Flat,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ChordType {
    Minor,
    Major,
    Augmented,
    Diminished,
    Add,
    Sus2,
    Sus4,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Extension {
    //TODO: なんかおかしいな
    MinorThird,
    Third,
    Fourth,
    FlatFifth,
    DiminishedFifth,
    Fifth,
    SharpFifth,
    FlatSixth,
    Sixth,
    Seventh,
    FlatNinth,
    Ninth,
    SharpNinth,
    FlatEleventh,
    Eleventh,
    SharpEleventh,
    FlatThirteenth,
    Thirteenth,
    SharpThirteenth,
}
