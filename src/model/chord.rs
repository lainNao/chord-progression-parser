use super::chord_detailed::ChordDetailed;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chord {
    /** Input chord spelling and extension order, with whitespace removed. */
    pub plain: String,
    pub detailed: ChordDetailed,
}
