use super::bar::Bar;
use super::section_meta::SectionMeta;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare]
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Section {
    pub meta_infos: Vec<SectionMeta>,
    pub chord_blocks: Vec<Bar>,
}
