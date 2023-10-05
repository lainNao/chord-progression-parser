#[derive(Debug, PartialEq, Clone)]
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
