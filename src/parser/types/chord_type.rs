use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum ChordType {
    Minor,      // m
    Major,      // M
    Augmented,  // aug
    Diminished, // dim
}
