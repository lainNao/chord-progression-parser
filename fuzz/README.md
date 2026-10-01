# Parser fuzzing

This optional workspace runs coverage-guided fuzzing with `cargo-fuzz` and
AddressSanitizer. It does not change the main crate's stable toolchain or add
dependencies to published packages.

## Setup

```sh
rustup toolchain install nightly --profile minimal --component rust-src
cargo install cargo-fuzz --locked
```

See the [Rust Fuzz Book](https://rust-fuzz.github.io/book/cargo-fuzz/setup.html)
for supported platforms and prerequisites.

## Run

From the repository root:

```sh
mkdir -p fuzz/corpus/parser
cargo +nightly fuzz run parser fuzz/corpus/parser fuzz/seeds -- \
  -max_total_time=300 -max_len=4096 -timeout=10 -rss_limit_mb=2048
```

The first corpus directory receives generated inputs. The checked-in seeds
exercise all four chord types, extensions, duplicate warnings, Unicode, CRLF,
section boundaries, and recovery from malformed notation. The parser accepts UTF-8 strings, so
invalid UTF-8 bytes are discarded before calling its API.

The target checks the same invariants as `tests/source_positions.rs` through
`tests/support/mod.rs`: diagnostic ranges and order, Unicode display coordinates,
warning source text, and preservation of the AST and warning values after
formatting and reparsing. Accepted `ChordDetailed::from_str` inputs must also
agree with the document parser. The legacy Rust API's result must match the
warning-aware API. The stable regression suite runs these checks against
deterministic mutations without installing nightly or cargo-fuzz.

Crashing inputs are saved under `fuzz/artifacts/parser/`. Replay a saved input
with `cargo +nightly fuzz run parser <artifact-path>`, minimize it with
`cargo +nightly fuzz tmin parser <artifact-path>`, and add the reproducer to the
regular regression tests before fixing the bug. Generated corpora, artifacts,
lockfiles, and build outputs are ignored by Git.

## Valid-document grammar target

The `grammar` target interprets bytes as bounded choices for complete valid
progressions. It builds the expected AST independently of both the document
parser and formatter, then checks parsing, duplicate-warning counts, source
positions, and formatting round trips. Choices cover all chord types and
extensions, special expressions, key and section metadata, opaque denominators,
optional horizontal whitespace, trailing bar commas, and all newline forms.
It also appends malformed chords to otherwise valid progressions and checks
that earlier warnings are preserved, later chords are inspected, and repetition
context resets at section boundaries. The same generator runs deterministically
in `tests/generated_documents.rs`.

```sh
cargo +nightly fuzz run grammar -- \
  -max_total_time=300 -max_len=1024 -timeout=10 -rss_limit_mb=2048
```

Each input produces at most four sections, five bars per section, four chords
per bar, and five extensions per chord. Unlike `parser`, its bytes are grammar
choices rather than source text and need not be valid UTF-8. Crashes are saved
under `fuzz/artifacts/grammar/`; replay and minimize them with the `grammar`
target in place of `parser`.
