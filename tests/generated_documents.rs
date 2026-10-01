#[path = "support/generated.rs"]
mod generated;
mod support;

/** Exercises complete valid documents independently of formatter-produced source. */
#[test]
fn grammar_generated_documents_match_their_expected_asts() {
    for byte in 0..=u8::MAX {
        generated::assert_generated_document(&[byte]);
    }
    let mut state = 0x4d59_5df4_d0f3_3173_u64;
    for _ in 0..2_000 {
        let mut bytes = [0; 128];
        for byte in &mut bytes {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            *byte = state as u8;
        }
        generated::assert_generated_document(&bytes);
    }
}
