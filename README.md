# chord-progression-parser

A converter from chord progression strings to AST built in Rust that outputs wasm, so it can be used from JavaScript too.

> NOTE: This library releases multiple packages. ![GitHub release (latest SemVer)](https://img.shields.io/github/v/release/lainNao/chord-progression-parser)
>
> - Rust: <https://crates.io/crates/chord-progression-parser>
> - JS/TS(bundler): <https://www.npmjs.com/package/@lainnao/chord-progression-parser-bundler>
> - JS/TS(server): <https://www.npmjs.com/package/@lainnao/chord-progression-parser-node>
> - JS(CDN): <https://www.npmjs.com/package/@lainnao/chord-progression-parser-web>

## Preferred chord spelling

**Use shorthand without parentheses by default for readability.** When writing
or generating chord progressions in this format, prefer `Am7`, `C7`, `CM7`,
`C7sus4`, `Cm7b5`, and `C7omit5`.

```txt
@section=Verse
[key=C]CM7 - Am7 - Dm7 - G7
```

Parenthesized spellings such as `Am(7)`, `C(7)`, and `C(M7)` are also valid,
but shorthand is the recommended default. Use parentheses when needed, such as
`C7(9)`, `C6(9)`, or `C(#5)`. Write `C7sus4` when parentheses are unnecessary;
`Csus4(7)` is also accepted. `CM7` means `C(M7)`, while `CM(7)` retains its
existing meaning. `maj`, bare `sus`, symbolic `o`/`ø`/`°` aliases, `alt`, and
`no3` are not supported. The existing
parenthesized `o` extension remains accepted. Use `C6(9)` rather than `C6/9`.

`plain` preserves the input chord spelling and extension order, with whitespace
removed. The formatter retains that spelling. `detailed.extensions` uses a
fixed order, preserves duplicates, and does not infer additional chord tones.
For example, `C7sus4` and `Csus4(7)` have the same detailed representation but
different `plain` strings. `omit5` records omission of the fifth without
computing sounding pitches.

See the [syntax reference](./_docs/en/about-chord-progression-syntax.md)
([日本語](./_docs/ja/about-chord-progression-syntax.md)) for the complete rules
and extension order.

## Example

You can try it on [CodeSandbox](https://codesandbox.io/p/devbox/vite-react-ts-forked-phmkrs?file=%2Fsrc%2FApp.tsx)

![example gif](https://i.imgur.com/kGwySIJ.gif)

## Documents

- English
  - [about chord progression syntax](./_docs/en/about-chord-progression-syntax.md)
  - [how to develop](./_docs/en/how-to-develop.md)
  - [memo](./_docs/en/memo.md)
- Japanese
  - [コード進行ASTの文法の説明](./_docs/ja/about-chord-progression-syntax.md)
  - [開発についての説明](./_docs/ja/how-to-develop.md)
  - [メモ](./_docs/ja/memo.md)

## How to use

### `Rust`

- Install

  ```sh
  cargo add chord-progression-parser
  ```

- And use

  ```rust
  use chord_progression_parser::{format_chord_progression, parse_chord_progression_string};

  fn main() {
    let input: &str = "
  @section=Intro
  [key=E]E - C#m7 - Bm7 - C#7
  F#m7 - Am7 - F#7 - B

  @section=Verse
  E - C#m7 - Bm7 - C#7
  F#m7 - Am7 - F#7 - B
  ";

      let result = parse_chord_progression_string(input);
      if let Ok(ast) = result {
        println!(
          "{}",
          format_chord_progression(&ast).expect("parsed AST must be formattable")
        );
      }
  }
  ```

### `JavaScript/TypeScript (using bundler, like Vite, or If you are using Next.js)`

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
  import {
    formatChordProgression,
    parseChordProgressionString,
  } from "@lainnao/chord-progression-parser-bundler/chord_progression_parser";

  const result = parseChordProgressionString("C");
  if (result.success) {
    console.log(formatChordProgression(result.ast));
  } else {
    console.log(result.errors);
  }
  console.log(result);
  ```

### `JavaScript/TypeScript (server like Node.js, Bun)`

- Install

  ```sh
  npm install @lainnao/chord-progression-parser-node
  ```

- And use

  ```typescript
  import {
    formatChordProgression,
    parseChordProgressionString,
  } from "@lainnao/chord-progression-parser-node";

  const result = parseChordProgressionString("C");
  if (result.success) {
    console.log(formatChordProgression(result.ast));
  } else {
    console.log(result.errors);
  }
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
        import * as mod from "https://cdn.jsdelivr.net/npm/@lainnao/chord-progression-parser-web@1.1.1/chord_progression_parser.js";

        (async () => {
          // initialize wasm
          await mod.default();
          // use
          const result = mod.parseChordProgressionString("C");
          if (result.success) {
            console.log(mod.formatChordProgression(result.ast));
          } else {
            console.log(result.errors);
          }
          console.log(result);
          document.querySelector("#result").textContent = JSON.stringify(
            result,
            null,
            2,
          );
        })();
      </script>
    </body>
  </html>
  ```

## Diagnostic positions

Errors and warnings use the same position units:

- `lineNumber` is one-based; CRLF is one line break.
- `columnNumber` is a one-based Unicode scalar count within that line. Tabs
  count as one scalar rather than advancing to a tab stop.
- `length` counts Unicode scalar values, not UTF-8 bytes, UTF-16 code units,
  or rendered grapheme clusters.
- `startOffset` and `endOffset` are zero-based UTF-16 offsets into the original
  input, with an exclusive end. Use `input.slice(startOffset, endOffset)` in
  JavaScript to extract the exact diagnostic text.

For example, the invalid chord `H😀` has `length: 2` but occupies the UTF-16
range `[0, 3)`. An EOF diagnostic has `length: 0` and equal start/end offsets;
a UI can render a caret at that position. Rust exposes the same values through
`Position`, using snake-case field names. Rust string slices require byte
indices, so UTF-16 offsets must not be used directly as Rust slice boundaries.

## Parse warnings

The warning API described here was added after v0.9.3. Packages at v0.9.3 and
earlier do not expose `warnings` or the warning-message submodule.

`parseChordProgressionString` always returns `warnings: ParseWarning[]`, on both
success and failure. An empty array means no warnings were found. Warnings do
not change `success`, the AST, or the existing `errors` shape and error codes.

The JavaScript parser requires a primitive string. Passing another type throws
`invalid chord progression input: expected a string`; it does not coerce values
to strings. Invalid chord syntax in a string still returns `success: false`
with diagnostics.

For example, `C(9,9)` succeeds and preserves both `9` entries in the AST. Its
second `9` produces a warning with code `DUPLICATE_EXTENSION` and
`additionalInfo: "9"`. Each further repetition in the same chord produces its
own warning. Related spellings such as `9` and `add9` are not duplicates.
Warnings encountered in malformed chord extension lists can accompany errors;
extensions on `%`, `_`, or `?`, and slash denominator extensions do not receive
duplicate warnings.

Slash denominators must be complete chord spellings, such as `C/G`, `C/G7`, or
`C/F#m7b5`, or `?` for an unidentified bass (`C/?`). Prefer shorthand here too;
parenthesized spellings such as `C/F#m(7,b5)` are also accepted. Arbitrary text
and numeric denominators such as `C/Unknown` and `C/5` return `DEN-1`.

`ParsePosition`, `ParseError`, `WarningCode`, and `ParseWarning` are exported
TypeScript types. Warning positions use the original source text: display
line/column numbers are one-based, and `startOffset`/`endOffset` are zero-based
UTF-16 offsets with an exclusive end, just like error positions. Formatting
and reparsing preserves the AST but recomputes positions for the formatted text.

Rust callers can use `parse_chord_progression_string_with_warnings` to obtain
a `ParseReport` containing `result` and `warnings`. The existing
`parse_chord_progression_string` still returns `Result<Ast, Vec<ErrorInfoWithPosition>>`.
Consumers may ignore warnings; displaying them is a separate application concern.

All three npm packages provide localized warning messages through the
`warning_code_message_map` submodule, alongside the existing error-message module:

```ts
import { parseChordProgressionString } from "@lainnao/chord-progression-parser-node";
import { getWarningMessage } from "@lainnao/chord-progression-parser-node/warning_code_message_map.js";

const result = parseChordProgressionString("C(9,9)");
for (const warning of result.warnings) {
  console.log(getWarningMessage({ warningCode: warning.code, lang: "ja" }));
}
```

`lang` accepts `"en"` or `"ja"`; `WARNING_CODE_MESSAGE_MAP` is also exported.
For native Node.js ESM and browser imports, use the submodule's `.js` filename.

## Article

- Japanese
  - <https://zenn.dev/kyln/articles/13637e86e3ff07>
