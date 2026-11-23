use super::chord::Chord;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum ChordExpression {
    Chord(Chord),
    UnIdentified, // ?
    NoChord,      // _
    Same,         // %
}
