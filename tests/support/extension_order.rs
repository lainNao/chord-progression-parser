use std::sync::OnceLock;

use chord_progression_parser::Extension;
use serde::Deserialize;

#[derive(Deserialize)]
struct OrderFixture {
    #[serde(rename = "extensionOrder")]
    extensions: Vec<Extension>,
}

/** Reads the fixed specification fixture without using the production parser or sort key. */
pub fn expected_order() -> &'static [Extension] {
    static ORDER: OnceLock<Vec<Extension>> = OnceLock::new();
    ORDER.get_or_init(|| {
        serde_json::from_str::<OrderFixture>(include_str!("../fixtures/canonical_syntax.json"))
            .expect("extension order fixture must be valid")
            .extensions
    })
}

/** Ranks generated expected values against the independently specified order. */
pub fn expected_rank(extension: &Extension) -> usize {
    expected_order()
        .iter()
        .position(|value| value == extension)
        .expect("every extension needs an explicit specification rank")
}
