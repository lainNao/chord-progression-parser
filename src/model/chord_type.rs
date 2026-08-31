use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum ChordType {
    #[serde(rename = "m")]
    Minor,
    #[serde(rename = "M")]
    Major,
    #[serde(rename = "aug")]
    Augmented,
    #[serde(rename = "dim")]
    Diminished,
}
