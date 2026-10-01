#![no_main]

// Share the same invariants with the stable-toolchain regression suite.
#[path = "../../tests/support/mod.rs"]
mod support;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(input) = std::str::from_utf8(data) {
        support::assert_parse_invariants(input);
    }
});
