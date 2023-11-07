use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare]
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
