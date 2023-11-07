use super::chord::Chord;
use super::chord_info_meta::ChordInfoMeta;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]

pub struct ChordInfo {
    pub meta_infos: Vec<ChordInfoMeta>,
    pub chord: ChordOrUnidentified,
    pub denominator: Option<String>, // 曖昧で扱いようが無いのでstring
}

#[typeshare]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum ChordOrUnidentified {
    Chord(Chord),
    Unidentified,
}
