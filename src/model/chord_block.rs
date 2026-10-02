use super::bar::Bar;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum ChordBlock {
    Bar(
        #[cfg_attr(
            target_arch = "wasm32",
            serde(deserialize_with = "crate::wasm_ast::deserialize_array")
        )]
        Bar,
    ),
    Br, // break of line
}
