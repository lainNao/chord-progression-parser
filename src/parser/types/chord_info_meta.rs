use super::key::Key;

#[derive(Debug, PartialEq, Clone)]
pub enum ChordInfoMeta {
    Key { value: Key },
    // or more
}
