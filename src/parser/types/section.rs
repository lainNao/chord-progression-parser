use super::chord_block::ChordBlock;
use super::section_meta::SectionMeta;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Section {
    pub meta_infos: Vec<SectionMeta>,
    pub chord_blocks: Vec<ChordBlock>,
}
