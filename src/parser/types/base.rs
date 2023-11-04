use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Base {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}
