use super::key::Key;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum ChordInfoMeta {
    Key { value: Key },
    // or more
}
