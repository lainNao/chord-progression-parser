use super::chord_block::ChordBlock;
use super::section_meta::SectionMeta;

#[derive(Debug, PartialEq, Clone)]
pub struct Section {
    pub meta_infos: Vec<SectionMeta>,
    pub chord_blocks: Vec<ChordBlock>,
}
