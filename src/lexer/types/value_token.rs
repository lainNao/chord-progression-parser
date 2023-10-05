#[derive(Debug, PartialEq, Clone)]
pub enum ValueToken {
    SectionMetaInfoKey,
    SectionMetaInfoValue,
    MetaInfoKey,
    MetaInfoValue,
    Chord,
    Denominator,
}
