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
  - please refer to [generatedTypes.ts](../../generatedTypes.ts)
- Details
  - `SectionMeta`
    - format: `@key=value`
  - `ChordInfo`
    - format: `[key=value]Chord(Extension)`
    - capture:
      - `[key=value]` ...Optional
      - `(extension)` ..Optional. Multiple extensions can be specified, separated by commas
      - `Chord` ・・・Fractional codes like `C/B`, `?` and `%` are also possible
