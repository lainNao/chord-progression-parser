# Definition of chord progression syntax (v1)

## Chord spelling and structured representation

Both explicit parenthesized extensions and the shorthand described below are accepted.
`plain` retains the input spelling and extension order, removing whitespace; the
formatter retains that spelling. `detailed.extensions` is sorted by the fixed
order below, without removing duplicates or deriving sounding pitches.
Consequently `C7sus4` and `Csus4(7)` share a detailed representation, but their
`plain` strings and complete ASTs differ.

When upgrading from 0.x to 1.0.0, reparse saved source text: previously stored ASTs
with a different extension order can be rejected by the formatter's round-trip
validation. Regenerate order-sensitive derived data, and update exhaustive
extension handling for the new `omit5` value. If only ASTs are stored, export
source using the previous formatter before upgrading.

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
    - `@section=Name`: section name
    - `@repeat=2`: repetition count
    - `@same=Name`: reference to the chord progression of the named section
  - `ChordInfo`
    - format: `[key=value]Chord`; the chord may use shorthand and/or a final extension group
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
    - Blank lines between chord lines stay inside the same section; multiple blank lines collapse to one (two consecutive `br` blocks in the AST)
    - Blank lines do not reset the section name, key, or `%` reference to the preceding chord
    - Section metadata after chord lines also starts a new section, even with one line break
    - Blank lines before the first chord line retain preceding section metadata in the same section
    - Line breaks at document edges and before a section's first chord line are omitted
  - Constraints
    - Chord metadata must precede its chord; postfix forms such as `C[key=A]` are invalid
    - Section metadata must occupy its own line
    - Pipe notation such as `|C|` is not supported

## Section names

Values of `@section=` and `@same=` are literal strings up to the next line ending
(LF, CRLF, or CR) or end of input. Names may contain hyphens, spaces, tabs,
parentheses, slashes, `=`, `@`, and Unicode characters. Surrounding ASCII spaces
and tabs are removed; internal whitespace, punctuation, and case are preserved.
Empty names produce `SMIV-1`. Names cannot contain line endings.

```txt
@section=pre-chorus / サビ前 (2)
C - G

@same=pre-chorus / サビ前 (2)
```

Text such as `@repeat=2` on the same line is part of the name. Put each additional
metadata directive or chord progression on its own line. Quotes and comments have
no special meaning. `@repeat` still requires a non-negative integer.

## Named-section references (`@same`)

```txt
@section=Verse
C - G - Am - F

@section=Reprise
@same=Verse
@repeat=2
```

An optional `@section` before `@same` names the referencing section. The AST stores
the reference as `SectionMeta` `{ "type": "same", "value": "Verse" }` without
expanding the target's chords. A reference-only section has an empty `chordBlocks`
array. Reference names use the same literal line-value syntax as section names
and retain their exact Unicode spelling and case.

Following `@repeat` metadata belongs to the referencing section. Following
`@section` or `@same` metadata starts a new section even without any chord lines.

`@same` references the entire section. Combining it with chord lines in that
section is a syntax error (`SMIV-4`). Start a new section with `@section` before
writing another chord progression.

Applications decide target existence, forward/self/cyclic references, and which
definition to choose when duplicate names have different contents. The parser
retains references and definitions in source order without choosing a definition.

## Slash chord denominators

A denominator must be a note name or a complete chord in the same syntax as the
numerator. `C/G`, `C/F#`, and `C/F#m(7,b5)` are accepted, and the denominator
spelling remains a string in the AST. `C/?` means a C chord with an unidentified
bass; `?` must occupy the entire denominator.

Numeric degrees and arbitrary text, including `C/5`, `C/Unknown`, `C/Garbage`,
`C/%`, and `C/_`, produce `DEN-1`. The same shorthand is accepted in
denominators: `C/G7`, `C/F#m7b5`, and `C/Gsus4(7)` are valid. Spaces may
surround punctuation but must not split an atom, so `C/G m` is invalid.

## Diagnostic positions

Errors and warnings use one-based line numbers and Unicode scalar columns.
Tabs count as one scalar. `length` also counts Unicode scalars, while
`startOffset` and `endOffset` use zero-based UTF-16 offsets with an exclusive end.
For example, `H😀` has length 2 and covers offsets `[0, 3)`.
Use JavaScript's `input.slice(startOffset, endOffset)` to extract the source;
these offsets are not Rust string byte indices. EOF diagnostics have an empty
range. CRLF is one line break but contains two scalars and two UTF-16 code units.

## Duplicate-extension warnings

`C(9,9)` remains valid and preserves both extensions in the AST. Each occurrence
after the first identical value across a chord's shorthand and parenthesized
extensions produces a
`DUPLICATE_EXTENSION` warning. Related musical meanings, such as `9` and `add9`,
are not compared.

The JavaScript API always returns a `warnings` array, on success and failure.
Warnings do not fail parsing; no warnings produces an empty array. Positions
refer to the original input and use the same display line/column numbers and
UTF-16 offsets as errors. Duplicates detected in malformed chord lists may
accompany errors. Unsupported extensions on `%`, `_`, or `?`, and slash
denominator extensions do not receive duplicate warnings.

Each npm package provides `getWarningMessage({ warningCode: warning.code, lang: "en" })`
through its `warning_code_message_map` submodule. `lang` accepts `"en"` or `"ja"`.
Use the `.js` filename for native Node.js ESM or direct browser imports.

## Extensions

The accepted values are exactly the following 31 case-sensitive strings,
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
omit5
```
<!-- extension-values:end -->

One optional non-empty parenthesized group may follow a chord atom. Separate
its values with commas. All declared extensions are accepted inside parentheses.
Chord types `m`, `M`, `aug`, and `dim` belong to the chord head, not the extension list.

Outside parentheses, allow at most one leading primary extension:
`1`, `2`, `3`, `4`, `5`, `6`, `7`, `9`, `11`, `13`, `M7`, `M9`, `M11`, `M13`.
It may be followed by any number of `sus2`, `sus4`, `add9`, `add11`, `add13`,
`b3`, `b5`, `#5`, `b6`, `b9`, `#9`, `b11`, `#11`, `b13`, `#13`, and `omit5`.
A primary extension cannot follow another extension outside parentheses.
Longest tokens win: `C13` means `C(13)`, while `C79` and `C69` are rejected.
Use `C7(9)` and `C6(9)` instead. The final parenthesized group can add any
extensions: `C7(13)`, `Csus4(7)`, and `C7sus4(9,13)` are valid.
`C(7)sus4` and `C7(9)(13)` are invalid. Place the slash denominator last,
as in `C7sus4(13)/G`.

`M7`, `M9`, `M11`, and `M13` are whole extension tokens, not a chord-type
marker followed by a number. Thus `CM7` means `C(M7)`, while `CM(7)` still
means an explicitly major chord head with the `7` extension. `CmM7` and
`CaugM7` use the same extension rule. `Cdim7` stores the same detailed data
as `Cdim(7)`; the parser does not calculate its sounding pitches.

Root accidentals take precedence immediately after the base: `C#5` is `C#(5)`
and `Cb5` is `Cb(5)`. Write `C(#5)` or `C(b5)` to alter C's fifth instead.
`omit5` requests omission of the fifth; `omit1`, `omit3`, and `no3` are unsupported.
No implicit seventh is inserted into `C9`, which stores only the `9` extension.
Musically conflicting combinations such as `C7#5omit5` are stored without validation.

Structured extension order (independent of `plain` and enum declaration order):

<!-- extension-order:start -->
```txt
1, 2, 3, 4, 5, 6, 7, M7, 9, M9, 11, M11, 13, M13,
sus2, sus4, b3, b5, #5, b6, b9, #9, b11, #11, b13, #13,
add9, add11, add13, omit5, o
```
<!-- extension-order:end -->

Warnings are computed in original source order before sorting. For example,
`C7(7)` warns at the parenthesized `7` and retains both entries in the AST.
Shorthand applies only to chords and denominators, not keys or `?`, `_`, `%`.

Use `1`, as in `C(1)`, to express a root-only notation.
The parser stores this modifier in `extensions` and preserves the chord head's type.
It does not derive sounding pitches or validate extension combinations musically;
applications are responsible for interpreting the sounding pitches.

Valid examples:

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

Additional valid shorthand examples:

```txt
C1
C7
Cm7
CM7
C9
Cadd9
Csus4
Csus4(7)
Cm7b5
Cdim7
Caug7
CaugM7
C7omit5
C7/E
```

Unsupported spellings include `Cmaj7`, `C△7`, `C+7`, `Co`, `Co7`, `Cø7`,
`C°7`, `Csus`, `C7sus`, `C6/9`, `Cadd2`, `Cadd4`, `C7alt`, and `Cno3`.
The existing parenthesized `o` extension remains accepted; it is not a new
shorthand for diminished chords.

See the [machine-readable fixture](../../tests/fixtures/canonical_syntax.json).

## Empty named sections and source positions (1.1.1)

A new `@section` starts a separate section even before any chords have been entered. Consecutive names are not merged into the first definition. `@repeat` and a qualifying `@same` remain attached to their section. Reference names remain literal strings; target lookup, duplicate precedence and identifier semantics belong to the application.

Successful JavaScript results include `sectionMetaPositions` outside the AST. Each entry addresses `ast[sectionIndex].metaInfos[metaIndex]`; `position` covers only the original metadata value after `=`, excluding surrounding ASCII spaces and tabs. Offsets are zero-based UTF-16 with an exclusive end; line/column are one-based, CRLF counts as one newline, and columns/length count Unicode scalar values. Applications can use these exact ranges for semantic diagnostics without rescanning metadata syntax. The stored AST shape is unchanged.
