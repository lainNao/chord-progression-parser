# chord-progression-parser

a converter from chord progression strings to AST built in Rust that outputs wasm, so it can be used from JavaScript too.

## example

- TBD

## how to use

- `for Rust user:` <https://crates.io/crates/chord-progression-parser>
  - install
    - `cargo add chord-progression-parser`
  - use

    ```rust
    use chord_progression_parser::parse_chord_progression_string;

    fn main() {
        let input: &str = "
    @section=Intro
    |[key=E]E|C#m(7)|Bm(7)|C#(7)|
    |F#m(7)|Am(7)|F#(7)|B|
    ///
    @section=Verse
    |E|C#m(7)|Bm(7)|C#(7)|
    |F#m(7)|Am(7)|F#(7)|B|
    ";
        
        let result = parse_chord_progression_string(input);
        println!("{:#?}", result);
    }
    ```

- `for JavaScript user (frontend using CDN)`
  - TBD
- `for JavaScript/TypeScript user (frontend using some bundler)`
  - TBD
- `for JavaScript/TypeScript user (server)`
  - TBD

## for more info

- docs
  - English
    - [about chord progression syntax](./_docs/en/about-chord-progression-syntax.md)
    - [how to develop](./_docs/en/how-to-develop.md)
  - Japanese
    - [コード進行ASTの文法の説明](./_docs/ja/about-chord-progression-syntax.md)
    - [開発についての説明](./_docs/ja/how-to-develop.md)
