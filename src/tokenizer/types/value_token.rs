#[derive(Debug, PartialEq, Clone, strum_macros::Display)]
pub enum ValueToken {
    SectionMetaInfoKey,
    SectionMetaInfoValue,
    MetaInfoKey,
    MetaInfoValue,
    Chord,
    Extension,
    Denominator,
}
