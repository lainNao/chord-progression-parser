#[derive(Debug, PartialEq, Clone, strum_macros::Display)]
pub enum Token {
    // Common
    Equal,
    Comma,
    LineBreak,
    Slash,

    // SectionMetaInfoElement
    SectionMetaInfoStart, // @
    SectionMetaInfoKey(String),
    SectionMetaInfoValue(String),

    // ChordBlockElement
    ChordBlockSeparator, // |
    Chord(String),       // 分子
    Denominator(String), // 分母

    // MetaInfoElement
    MetaInfoStart, //(
    MetaInfoKey(String),
    MetaInfoValue(String),
    MetaInfoEnd, //)
}

#[derive(Debug, PartialEq, Clone, strum_macros::Display)]
pub enum ValueToken {
    SectionMetaInfoKey,
    SectionMetaInfoValue,
    MetaInfoKey,
    MetaInfoValue,
    Chord,
    Denominator,
}
