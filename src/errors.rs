// should not contains xxx
pub const SECTION_META_INFO_KEY_SHOULD_NOT_CONTAINS_LINE_BREAK: &str =
    "Error: SectionMetaInfoKey should not contains line break";
pub const META_INFO_KEY_SHOULD_NOT_CONTAINS_LINE_BREAK: &str =
    "Error: MetaInfoKey should not contains line break";
pub const META_INFO_VALUE_SHOULD_NOT_CONTAINS_LINE_BREAK: &str =
    "Error: MetaInfoValue should not contains line break";
pub const CHORD_BLOCK_SHOULD_NOT_CONTAINS_LINE_BREAK: &str =
    "Error: Chord should not contains line break";

pub const CHORD_SHOULD_NOT_CONTAINS_MULTIPLE_SLASHES: &str =
    "Error: Chord should not contains multiple slashes";

// should not be empty
pub const SECTION_META_INFO_KEY_SHOULD_NOT_BE_EMPTY: &str =
    "Error: SectionMetaInfoKey should not be empty";
pub const SECTION_META_INFO_VALUE_SHOULD_NOT_BE_EMPTY: &str =
    "Error: SectionMetaInfoValue should not be empty";
pub const META_INFO_KEY_SHOULD_NOT_BE_EMPTY: &str = "Error: MetaInfoKey should not be empty";
pub const META_INFO_VALUE_SHOULD_NOT_BE_EMPTY: &str = "Error: MetaInfoValue should not be empty";
pub const CHORD_SHOULD_NOT_BE_EMPTY: &str = "Error: Chord should not be empty";
pub const CHORD_BLOCK_SHOULD_NOT_BE_EMPTY: &str = "Error: ChordBlock should not be empty";

// should have after
pub const SECTION_META_INFO_VALUE_NEEDS_LINE_BREAK_AFTER: &str =
    "Error: SectionMetaInfoValue needs line break after";
pub const META_INFO_VALUE_NEEDS_CLOSE_PARENTHESIS_AFTER: &str =
    "Error: MetaInfoValue needs close parenthesis after";

// invalid
pub const INVALID_TOKEN_TYPE: &str = "Error: Invalid token type";
pub const SECTION_META_INFO_KEY_IS_INVALID: &str = "Error: SectionMetaInfoKey is invalid";
pub const SECTION_META_INFO_VALUE_OF_REPEAT_NEEDS_TO_BE_NUMBER: &str =
    "Error: SectionMetaInfoValue of repeat needs to be number";
pub const META_INFO_KEY_IS_INVALID: &str = "Error: MetaInfoKey is invalid";
pub const META_INFO_VALUE_IS_INVALID: &str = "Error: MetaInfoValue is invalid";

// should not be isolated
// NOTE: test it on parser
// pub const CHORD_BLOCK_SEPARATOR_SHOULD_NOT_BE_ISOLATED: &str =
//     "Error: ChordBlockSeparator should not be isolated";
// pub const LINE_TOP_CHAR_IS_INVALID: &str = "Error: Line top char is invalid";
