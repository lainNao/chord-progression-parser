use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
