use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub enum SectionMeta {
    Section { value: String },
    Repeat { value: u32 },
    // or more
}
