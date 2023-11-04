use super::chord_detailed::ChordDetailed;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Chord {
    pub plain: String,
    pub detailed: ChordDetailed,
}
