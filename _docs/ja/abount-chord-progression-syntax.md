# コード進行構文の定義

## ASTの構造

- 基本
  - [generatedTypes.ts](../../generatedTypes.ts)を参照
- 詳細
  - `SectionMeta`
    - 形式：`@key=value`
  - `ChordInfo`
    - 形式：`(key=value)Chord(Extension)`
    - 捕捉：
      - `(key=value)`　・・・オプショナル
      - `(extension)`　・・・オプショナル。extensionはカンマ区切りで複数指定可能
      - `Chord`　・・・`C/B`のような分数コード、`?`、`%`も可能
