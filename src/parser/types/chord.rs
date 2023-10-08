use super::chord_detailed::ChordDetailed;

#[derive(Debug, PartialEq, Clone)]
pub struct Chord {
    pub plain: String,
    pub detailed: ChordDetailed,
}
