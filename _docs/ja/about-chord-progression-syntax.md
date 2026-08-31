# コード進行構文の定義

## 例

```txt
@section=SimpleVerse
C - Dm - Em - F
G - Am - Bm(o) - C

@section=ComplexChorus
[key=Gm]F(9,13) - Fm(6) - Bb(add9,13) - Bbaug
[key=E]F#m(b5,7),Gm(M9) - Bm(7,9) - E(M13) - Edim
```

## 構文

- 基本
  - 厳密な文法は [chord-progression.ebnf](../chord-progression.ebnf) を参照
  - [generatedTypes.ts](../../resources/generatedTypes.ts)を参照
- 詳細
  - `SectionMeta`
    - 形式：`@key=value`
  - `ChordInfo`
    - 形式：`[key=value]Chord(Extension)`
    - 捕捉：
      - `[key=value]`　・・・オプショナル
      - `(extension)`　・・・オプショナル。extensionはカンマ区切りで複数指定可能
      - `Chord`　・・・`C/B`のような分数コード、`?`、`%`, `_`も可能
        - `?` ・・・ 不明
        - `%` ・・・ 前のコードと同じ
        - `_` ・・・ コードなし
  - 区切り
    - `C-D` ・・・ `C`と`D`を別々のbarとして扱う
    - `C,D` ・・・ `C`と`D`を同じbarとして扱う
    - 1個の改行 ・・・ 同じsection内の改行として扱う
    - 2個以上の連続改行 ・・・ 新しいsectionとして扱う
  - 制約
    - chord metaは対象chordの直前に置く。`C[key=A]`のような後置は不可
    - section metaは専用行に置く
    - pipeを使う`|C|`記法には対応しない
