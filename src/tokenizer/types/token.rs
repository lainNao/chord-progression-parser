use strum_macros::{Display, EnumString, EnumVariantNames};

#[derive(Debug, PartialEq, Clone, Display, EnumString, EnumVariantNames)]
pub enum Token {
    // Common
    #[strum(serialize = "=")]
    Equal,
    #[strum(serialize = ",")]
    Comma,
    #[strum(serialize = "\r\n", serialize = "\n")]
    LineBreak,
    #[strum(serialize = "/")]
    Slash,

    // SectionMetaInfoElement
    #[strum(serialize = "@")]
    SectionMetaInfoStart,
    SectionMetaInfoKey(String),
    SectionMetaInfoValue(String),

    // ChordBlockElement
    #[strum(serialize = "|")]
    ChordBlockSeparator,
    Chord(String),       // 分子
    Denominator(String), // 分母

    // MetaInfoElement
    #[strum(serialize = "[")]
    MetaInfoStart,
    #[strum(serialize = "]")]
    MetaInfoEnd,
    MetaInfoKey(String),
    MetaInfoValue(String),

    // Extension
    #[strum(serialize = "(")]
    ExtensionStart,
    #[strum(serialize = ")")]
    ExtensionEnd,
    Extension(String),
}
