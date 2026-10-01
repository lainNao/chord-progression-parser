mod support;

use support::assert_parse_invariants;

/** Advances a deterministic state so failing mutations can be reproduced. */
fn next_random(state: &mut u64) -> usize {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state as usize
}

/** Exercises diagnostic ranges and successful round trips after mutating Unicode documents. */
#[test]
fn mutated_documents_keep_diagnostics_aligned_with_the_original_source() {
    let documents = [
        "C(9,9)-%\n\n@section=😀\n[key=E#]E#M(1,1)/あ😀(9,11),_-%",
        "@section=日本語\r\n@repeat=4294967295\r\n[key=Fb]C#dim(7,b5)/Gm(9,11)-?\r\nC(9,9,9)",
        "C/D(あ,😀)-[key=C#]Fbm(1,1)\r\r[key=E#]C(9,9)-%",
        "C-H\n\n%",
        "@section=😀\n\nC(9,9)",
        "C/D(9,11)-C(7,7)\n[key=Bb]C(1)-%",
    ];
    let alphabet: Vec<char> = "ACm79@[]()=,/-?%_ \t\n\rあ♭😀\0\u{2028}\u{200b}"
        .chars()
        .collect();
    let mut state = 0x4d59_5df4_d0f3_3173;
    let mut valid_documents = 0;
    let mut warnings = 0;

    for _ in 0..2_000 {
        let mut characters: Vec<char> = documents[next_random(&mut state) % documents.len()]
            .chars()
            .collect();
        for _ in 0..next_random(&mut state) % 9 {
            let index = next_random(&mut state) % (characters.len() + 1);
            match next_random(&mut state) % 3 {
                0 if index < characters.len() => {
                    characters.remove(index);
                }
                1 if index < characters.len() => {
                    characters[index] = alphabet[next_random(&mut state) % alphabet.len()];
                }
                _ => characters.insert(index, alphabet[next_random(&mut state) % alphabet.len()]),
            }
        }

        let input: String = characters.into_iter().collect();
        let (valid, warning_count) = assert_parse_invariants(&input);
        valid_documents += usize::from(valid);
        warnings += warning_count;
    }

    assert!(
        valid_documents >= 100,
        "corpus must exercise successful parses"
    );
    assert!(
        warnings >= 100,
        "corpus must exercise warning source ranges"
    );
}
