use strum_macros::{Display, EnumString, EnumVariantNames};

#[derive(Debug, PartialEq, Clone, Display, EnumString, EnumVariantNames)]
pub enum Extension {
    #[strum(serialize = "2")]
    Two,
    #[strum(serialize = "3")]
    Three,
    #[strum(serialize = "b3")]
    FlatThree,
    #[strum(serialize = "4")]
    Four,
    #[strum(serialize = "b5")]
    FlatFive,
    #[strum(serialize = "5")]
    Five,
    #[strum(serialize = "#5")]
    SharpFive,
    #[strum(serialize = "b6")]
    FlatSix,
    #[strum(serialize = "6")]
    Six,
    #[strum(serialize = "7")]
    Seven,
    #[strum(serialize = "b9")]
    FlatNine,
    #[strum(serialize = "9")]
    Nine,
    #[strum(serialize = "#9")]
    SharpNine,
    #[strum(serialize = "b11")]
    FlatEleven,
    #[strum(serialize = "11")]
    Eleven,
    #[strum(serialize = "#11")]
    SharpEleven,
    #[strum(serialize = "b13")]
    FlatThirteen,
    #[strum(serialize = "13")]
    Thirteen,
    #[strum(serialize = "#13")]
    SharpThirteen,
    #[strum(serialize = "M7")]
    MajorSeven,
    #[strum(serialize = "M9")]
    MajorNine,
    #[strum(serialize = "M11")]
    MajorEleven,
    #[strum(serialize = "M13")]
    MajorThirteen,
    #[strum(serialize = "add9")]
    Add9,
    #[strum(serialize = "add11")]
    Add11,
    #[strum(serialize = "add13")]
    Add13,
    #[strum(serialize = "sus2")]
    Sus2,
    #[strum(serialize = "sus4")]
    Sus4,
    #[strum(serialize = "o")]
    HalfDiminish,
}
