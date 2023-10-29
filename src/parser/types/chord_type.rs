#[derive(Debug, PartialEq, Clone)]
pub enum ChordType {
    Minor,      // m
    Major,      // M
    Augmented,  // aug
    Diminished, // dim
}
