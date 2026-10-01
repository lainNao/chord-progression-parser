# Definition of chord progression syntax

## Canonical representation

This syntax is a canonical serialization format for chord progression ASTs,
not an attempt to accept every conventional chord-symbol notation.
It intentionally differs from conventional chord names: extensions are always
written inside `(...)` to reduce spelling variations and support stable
AST → string → AST round trips. The formatter may standardize whitespace;
round trips preserve the AST, not the original text layout.

Handling aliases such as `Cmaj7`, `CM7`, and `C△7` directly in the parser is
outside its purpose. Applications accepting conventional chord names should
provide a separate conversion layer that maps them to canonical syntax
(e.g. all three aliases above to `C(M7)`) before parsing.
When writing examples or generating input, including with AI, keep this syntax;
do not rewrite it into conventional chord notation.

## example

```txt
@section=SimpleVerse
C - Dm - Em - F
G - Am - Bm(o) - C

@section=ComplexChorus
[key=Gm]F(9,13) - Fm(6) - Bb(add9,13) - Bbaug
[key=E]F#m(b5,7),Gm(M9) - Bm(7,9) - E(M13) - Edim
```

## syntax

- Basic
  - Refer to [chord-progression.ebnf](../chord-progression.ebnf) for the exact grammar
  - please refer to [generatedTypes.ts](../../resources/generatedTypes.ts)
- Details
  - `SectionMeta`
    - format: `@key=value`
  - `ChordInfo`
    - format: `[key=value]Chord(Extension)`
    - capture:
      - `[key=value]` ...Optional
      - `(extension)` ..Optional. Multiple extensions can be specified, separated by commas
      - `Chord` ...Fractional codes like `C/B`, `?`,`%`, and `_` are also possible.
        - `?` ...Unknown chord
        - `%` ...Same as previous chord
        - `_` ...No chord
  - Separators
    - `C-D` treats `C` and `D` as separate bars
    - `C,D` treats `C` and `D` as chords in the same bar
    - One line break is preserved inside the same section
    - Two or more consecutive line breaks after chord lines start a new section
    - Section metadata after chord lines also starts a new section, even with one line break
    - Blank lines before the first chord line retain preceding section metadata in the same section
  - Constraints
    - Chord metadata must precede its chord; postfix forms such as `C[key=A]` are invalid
    - Section metadata must occupy its own line
    - Pipe notation such as `|C|` is not supported

## Duplicate-extension warnings

`C(9,9)` remains valid and preserves both extensions in the AST. Each occurrence
after the first identical value in one chord's extension list produces a
`DUPLICATE_EXTENSION` warning. Related musical meanings, such as `9` and `add9`,
are not compared.

The JavaScript API always returns a `warnings` array, on success and failure.
Warnings do not fail parsing; no warnings produces an empty array. Positions
refer to the original input and use the same display line/column numbers and
UTF-16 offsets as errors. Duplicates detected in malformed chord lists may
accompany errors. Unsupported extensions on `%`, `_`, or `?`, and opaque slash
denominators are not checked.

Each npm package provides `getWarningMessage({ warningCode: warning.code, lang: "en" })`
through its `warning_code_message_map` submodule. `lang` accepts `"en"` or `"ja"`.
Use the `.js` filename when importing directly in a browser.

## Extensions

The accepted values are exactly the following 30 case-sensitive strings,
matching [`Extension`](../../resources/generatedTypes.ts).

<!-- extension-values:start -->
```txt
1
2
3
b3
4
b5
5
#5
b6
6
7
b9
9
#9
b11
11
#11
b13
13
#13
M7
M9
M11
M13
add9
add11
add13
sus2
sus4
o
```
<!-- extension-values:end -->

Extensions are optional, but when present must be inside one non-empty `(...)`
group immediately after the chord head. Separate multiple values with commas.
Write the slash denominator after the group, as in `C(7)/E`.
Chord types `m`, `M`, `aug`, and `dim` belong to the chord head, not the extension list.

Use `1`, as in `C(1)`, to express a root-only notation.
The parser stores this modifier in `extensions` and preserves the chord head's type.
It does not derive sounding pitches or validate extension combinations musically;
applications are responsible for interpreting the sounding pitches.

Valid canonical examples:

```txt
C
Cm
C#
C(1)
C(7)
Cm(7)
C(M7)
C(7,9)
C(b5,7)
C(add9)
C(sus4)
Cm(o)
C(7)/E
```

These conventional spellings are not canonical syntax and fail to parse:

```txt
C1
C7
Cm7
Cmaj7
CM7
C△7
C9
Cadd9
Csus4
Cm7b5
C7/E
```

See the [machine-readable fixture](../../tests/fixtures/canonical_syntax.json).
