use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Accidental {
    #[serde(rename = "#")]
    Sharp,
    #[serde(rename = "b")]
    Flat,
}
