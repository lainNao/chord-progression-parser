# Definition of chord progression syntax

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
    - Two or more consecutive line breaks start a new section
  - Constraints
    - Chord metadata must precede its chord; postfix forms such as `C[key=A]` are invalid
    - Section metadata must occupy its own line
    - Pipe notation such as `|C|` is not supported
