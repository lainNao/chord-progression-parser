use super::chord::Chord;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum ChordExpression {
    Chord(Chord),
    Unidentified, // ?
    NoChord,      // -
    Same,         // %
}
