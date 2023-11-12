# Definition of chord progression syntax

## Structure of AST

- Basic
  - please refer to [generatedTypes.ts](../../generatedTypes.ts)
- Details
  - `SectionMeta`
    - format: `@key=value`
  - `ChordInfo`
    - format: `(key=value)Chord(Extension)`
    - capture:
      - `(key=value)` ...Optional
      - `(extension)` ..Optional. Multiple extensions can be specified, separated by commas
      - `Chord` ・・・Fractional codes like `C/B`, `?` and `%` are also possible
