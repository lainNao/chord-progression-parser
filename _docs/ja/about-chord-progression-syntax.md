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
  - [generatedTypes.ts](../../generatedTypes.ts)を参照
- 詳細
  - `SectionMeta`
    - 形式：`@key=value`
  - `ChordInfo`
    - 形式：`[key=value]Chord(Extension)`
    - 捕捉：
      - `[key=value]`　・・・オプショナル
      - `(extension)`　・・・オプショナル。extensionはカンマ区切りで複数指定可能
      - `Chord`　・・・`C/B`のような分数コード、`?`、`%`も可能
