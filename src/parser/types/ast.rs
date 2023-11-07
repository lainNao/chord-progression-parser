use super::section::Section;
use typeshare::typeshare;

#[typeshare]
pub type Ast = Vec<Section>;
