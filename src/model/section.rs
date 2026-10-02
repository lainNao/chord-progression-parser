use super::chord_block::ChordBlock;
use super::section_meta::SectionMeta;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare]
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Section {
    #[cfg_attr(
        target_arch = "wasm32",
        serde(deserialize_with = "crate::wasm_ast::deserialize_array")
    )]
    pub meta_infos: Vec<SectionMeta>,
    #[cfg_attr(
        target_arch = "wasm32",
        serde(deserialize_with = "crate::wasm_ast::deserialize_array")
    )]
    pub chord_blocks: Vec<ChordBlock>,
}
