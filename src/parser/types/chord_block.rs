use super::chord_info::ChordInfo;
use typeshare::typeshare;

#[typeshare]
pub type ChordBlock = Vec<ChordInfo>;
