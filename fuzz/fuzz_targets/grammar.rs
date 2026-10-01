#![no_main]

#[path = "../../tests/support/generated.rs"]
mod generated;
#[path = "../../tests/support/mod.rs"]
mod support;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    generated::assert_generated_document(data);
});
