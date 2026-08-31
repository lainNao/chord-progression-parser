use super::chord_detailed::ChordDetailed;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chord {
    pub plain: String,
    pub detailed: ChordDetailed,
}
