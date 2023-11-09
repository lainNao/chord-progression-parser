use super::chord_expression::ChordExpression;
use super::chord_info_meta::ChordInfoMeta;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChordInfo {
    pub meta_infos: Vec<ChordInfoMeta>,
    pub chord: ChordExpression,
    pub denominator: Option<String>, // ambiguous so string
}
