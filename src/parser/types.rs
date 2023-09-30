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
#[derive(Debug, PartialEq, Clone)]
pub enum Key {
    Cb_M, //C flat major
    Cb_m, //C flat minor
    C_M,  //C major
    C_m,  //C minor
    Cs_M, //C sharp major
    Cs_m, //C sharp minor
    Db_M,
    Db_m,
    D_M,
    D_m,
    Ds_M,
    Ds_m,
    Eb_M,
    Eb_m,
    E_M,
    E_m,
    Fb_M,
    Fb_m,
    F_M,
    F_m,
    Fs_M,
    Fs_m,
    Gb_M,
    Gb_m,
    G_M,
    G_m,
    Gs_M,
    Gs_m,
    Ab_M,
    Ab_m,
    A_M,
    A_m,
    As_M,
    As_m,
    Bb_M,
    Bb_m,
    B_M,
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
