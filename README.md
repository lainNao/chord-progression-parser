# chord-progression-parser

A converter from chord progression strings to AST built in Rust that outputs wasm, so it can be used from JavaScript too.

> NOTE: This library releases multiple packages. ![GitHub release (latest SemVer)](https://img.shields.io/github/v/release/lainNao/chord-progression-parser)
>
> - Rust: <https://crates.io/crates/chord-progression-parser>
> - JS/TS(bundler): <https://www.npmjs.com/package/@lainnao/chord-progression-parser-bundler>
> - JS/TS(server): <https://www.npmjs.com/package/@lainnao/chord-progression-parser-server>
> - JS(CDN): <https://www.npmjs.com/package/@lainnao/chord-progression-parser-web>

## Example

You can try it on [CodeSandbox](https://codesandbox.io/p/devbox/vite-react-ts-forked-phmkrs?file=%2Fsrc%2FApp.tsx)

![example gif](https://i.imgur.com/kGwySIJ.gif)

## Documents

- English
  - [about chord progression syntax](./_docs/en/about-chord-progression-syntax.md)
  - [how to develop](./_docs/en/how-to-develop.md)
- Japanese
  - [コード進行ASTの文法の説明](./_docs/ja/about-chord-progression-syntax.md)
  - [開発についての説明](./_docs/ja/how-to-develop.md)

## How to use

### `Rust`

- Install

  ```sh
  cargo add chord-progression-parser
  ```

- And use

  ```rust
  use chord_progression_parser::parse_chord_progression_string;

  fn main() {
    let input: &str = "
  @section=Intro
  [key=E]E - C#m(7) - Bm(7) - C#(7)
  F#m(7) - Am(7) - F#(7) - B
  
  @section=Verse
  E - C#m(7) - Bm(7) - C#(7)
  F#m(7) - Am(7) - F#(7) - B
  ";

      let result = parse_chord_progression_string(input);
      println!("{:#?}", result);
  }
  ```

### `JavaScript/TypeScript (using bundler)`

- Install (example, use with `Vite`)

  ```sh
  npm install @lainnao/chord-progression-parser-bundler
  npm install -D vite-plugin-wasm
  ```

- Edit `vite.config.js`

  ```js
  import { defineConfig } from "vite";
  import wasm from "vite-plugin-wasm";

  export default defineConfig({
    plugins: [wasm()],
  });
  ```

- And use

  ```typescript
  import { parseChordProgressionString } from "@lainnao/chord-progression-parser-bundler/chord_progression_parser";

  const result = parseChordProgressionString("C");
  console.log(result);
  ```

### `JavaScript/TypeScript (server like Node.js, Bun)`

- Install

  ```sh
  npm install @lainnao/chord-progression-parser-server
  ```

- And use

  ```typescript
  import { parseChordProgressionString } from "@lainnao/chord-progression-parser-server/chord_progression_parser";

  const result = parseChordProgressionString("C");
  console.log(result);
  ```

### `JavaScript(CDN)`

- `index.html`

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
      <h2>parse C</h2>
      <pre id="result"></pre>
      <script type="module">
        import * as mod from "https://cdn.jsdelivr.net/npm/@lainnao/chord-progression-parser-web@0.3.0/chord_progression_parser.js";

        (async () => {
          // initialize wasm
          await mod.default();
          // use
          const result = mod.parseChordProgressionString("C");
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
