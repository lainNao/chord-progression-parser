use super::chord_info::ChordInfo;
use typeshare::typeshare;

#[typeshare]
pub type Bar = Vec<ChordInfo>;
