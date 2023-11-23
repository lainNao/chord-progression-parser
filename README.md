# chord-progression-parser

a converter from chord progression strings to AST built in Rust that outputs wasm, so it can be used from JavaScript too.

> NOTE: this library releases multiple packages.
>
> - Rust: <https://crates.io/crates/chord-progression-parser>
> - JS(CDN): <https://www.npmjs.com/package/@lainnao/chord-progression-parser-web>
> - JS/TS(bundler): <https://www.npmjs.com/package/@lainnao/chord-progression-parser-bundler>
> - JS/TS(server): <https://www.npmjs.com/package/@lainnao/chord-progression-parser-server>

## example

- `TBD: sandbox URL`

## how to use

- `Rust user:` <https://crates.io/crates/chord-progression-parser>
  - install
    - `cargo add chord-progression-parser`
  - and use

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

- `JavaScript/TypeScript user (frontend using some bundler)`
  - install (example, use with `Vite`)

    ```sh
    npm install @lainnao/chord-progression-parser-bundler
    npm install -D vite-plugin-wasm
    ```

  - `vite.config.js`

      ```js
      import { defineConfig } from "vite";
      import wasm from "vite-plugin-wasm";

      export default defineConfig({
        plugins: [wasm()],
      });
      ```

  - and use

    ```typescript
    import { parseChordProgressionString } from "@lainnao/chord-progression-parser-bundler/chord_progression_parser";

    const result = parseChordProgressionString("|C|");
    console.log(result);
    ```

- `JavaScript/TypeScript user (server)`
  - install
    - `npm install @lainnao/chord-progression-parser-server`
  - and use

    ```typescript
    import { parseChordProgressionString } from "@lainnao/chord-progression-parser-server/chord_progression_parser";

    const result = parseChordProgressionString("|C|");
    console.log(result);
    ```

- `for JavaScript user (frontend using CDN)`

    ```html
    <!DOCTYPE html>
      <html lang="en">
        <head>
          <meta charset="UTF-8" />
          <meta name="viewport" content="width=device-width, initial-scale=1.0" />
          <title>Document</title>
        </head>
        <body>
          <h1>load wasm directly example</h1>
          <h2>parse |C|</h2>
          <pre id="result"></pre>
          <script type="module">
            import * as mod from "https://cdn.jsdelivr.net/npm/@lainnao/chord-progression-parser-web@0.1.12/chord_progression_parser.js";
            (async () => {
              await mod.default();
              const result = mod.parseChordProgressionString("|C|");
              console.log(result);
              document.querySelector("#result").innerHTML = JSON.stringify(
                result,
                null,
                2
              );
            })();
          </script>
        </body>
      </html>
    ```

## for more info

- English docs
  - [about chord progression syntax](./_docs/en/about-chord-progression-syntax.md)
  - [how to develop](./_docs/en/how-to-develop.md)
- Japanese docs
  - [コード進行ASTの文法の説明](./_docs/ja/about-chord-progression-syntax.md)
  - [開発についての説明](./_docs/ja/how-to-develop.md)
