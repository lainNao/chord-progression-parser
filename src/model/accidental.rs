use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Accidental {
    #[serde(rename = "#")]
    Sharp,
    #[serde(rename = "b")]
    Flat,
}
