#[derive(Debug, PartialEq, Clone)]
pub enum ChordType {
    Minor,      // m
    Major,      // M
    Augmented,  // aug
    Diminished, // dim
    Add,        // add
    Sus2,       // sus2
    Sus4,       // sus4
}
