#[derive(Debug, PartialEq, Clone)]
pub enum SectionMeta {
    Section { value: String },
    Repeat { value: u32 },
    // or more
}
