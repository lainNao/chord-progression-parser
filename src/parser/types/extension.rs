use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString, EnumVariantNames};
use typeshare::typeshare;

#[typeshare]
#[derive(
    Debug, PartialEq, Clone, Display, EnumString, EnumVariantNames, Serialize, Deserialize,
)]
pub enum Extension {
    #[strum(serialize = "2")]
    #[serde(rename = "2")]
    Two,
    #[strum(serialize = "3")]
    #[serde(rename = "3")]
    Three,
    #[strum(serialize = "b3")]
    #[serde(rename = "b3")]
    FlatThree,
    #[strum(serialize = "4")]
    #[serde(rename = "4")]
    Four,
    #[strum(serialize = "b5")]
    #[serde(rename = "b5")]
    FlatFive,
    #[strum(serialize = "5")]
    #[serde(rename = "5")]
    Five,
    #[strum(serialize = "#5")]
    #[serde(rename = "#5")]
    SharpFive,
    #[strum(serialize = "b6")]
    #[serde(rename = "b6")]
    FlatSix,
    #[strum(serialize = "6")]
    #[serde(rename = "6")]
    Six,
    #[strum(serialize = "7")]
    #[serde(rename = "7")]
    Seven,
    #[strum(serialize = "b9")]
    #[serde(rename = "b9")]
    FlatNine,
    #[strum(serialize = "9")]
    #[serde(rename = "9")]
    Nine,
    #[strum(serialize = "#9")]
    #[serde(rename = "#9")]
    SharpNine,
    #[strum(serialize = "b11")]
    #[serde(rename = "b11")]
    FlatEleven,
    #[strum(serialize = "11")]
    #[serde(rename = "11")]
    Eleven,
    #[strum(serialize = "#11")]
    #[serde(rename = "#11")]
    SharpEleven,
    #[strum(serialize = "b13")]
    #[serde(rename = "b13")]
    FlatThirteen,
    #[strum(serialize = "13")]
    #[serde(rename = "13")]
    Thirteen,
    #[strum(serialize = "#13")]
    #[serde(rename = "#13")]
    SharpThirteen,
    #[strum(serialize = "M7")]
    #[serde(rename = "M7")]
    MajorSeven,
    #[strum(serialize = "M9")]
    #[serde(rename = "M9")]
    MajorNine,
    #[strum(serialize = "M11")]
    #[serde(rename = "M11")]
    MajorEleven,
    #[strum(serialize = "M13")]
    #[serde(rename = "M13")]
    MajorThirteen,
    #[strum(serialize = "add9")]
    #[serde(rename = "add9")]
    Add9,
    #[strum(serialize = "add11")]
    #[serde(rename = "add11")]
    Add11,
    #[strum(serialize = "add13")]
    #[serde(rename = "add13")]
    Add13,
    #[strum(serialize = "sus2")]
    #[serde(rename = "sus2")]
    Sus2,
    #[strum(serialize = "sus4")]
    #[serde(rename = "sus4")]
    Sus4,
    #[strum(serialize = "o")]
    #[serde(rename = "o")]
    HalfDiminish,
}
