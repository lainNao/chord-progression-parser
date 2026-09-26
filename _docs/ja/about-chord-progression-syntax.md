# コード進行構文の定義

## Canonical representation（標準の文字列表現）

この構文は、コード進行 AST を安定して表現するための canonical serialization format です。
世の中のあらゆるコードネーム表記を直接受理する notation parser を目的としていません。
一般的なコードネーム表記とは意図的に異なり、extension を必ず `(...)` 内に統一することで
表記揺れを減らし、AST → 文字列 → AST の安定した round-trip を重視しています。
formatter は空白などを整えるため、保持するのは AST であり、元の文字列のレイアウトではありません。

`Cmaj7`、`CM7`、`C△7` などの表記揺れを parser 本体で直接扱うことは目的としていません。
一般的なコードネームを受理するアプリケーションでは、parse 前に canonical syntax へ変換する層を
別途設けてください（上記の3例はいずれも `C(M7)` に変換）。
利用者や AI が例や入力を作成する際も、この構文を一般的なコード表記へ勝手に補正しないでください。

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

## Extension の有効値

受理する値は以下の29種類です。大文字・小文字は区別され、
[`Extension`](../../resources/generatedTypes.ts) enum と一致します。

<!-- extension-values:start -->
```txt
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

extension 自体は省略可能ですが、指定する場合はコード本体の直後の空でない `(...)` 内に
必ず記述します。複数の値はカンマで区切り、括弧のグループは1つにまとめます。
分数コードの `/E` などは `C(7)/E` のように括弧の後に置きます。
コード種別の `m`、`M`、`aug`、`dim` は extension ではなく、コード本体に記述します。

canonical syntax として有効な例：

```txt
C
Cm
C#
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

以下の一般的な表記は canonical syntax ではなく、parse に失敗します：

```txt
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

機械的に確認できる [fixture](../../tests/fixtures/canonical_syntax.json) も参照してください。
