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
    - コード行の後の2個以上の連続改行 ・・・ 新しいsectionとして扱う
    - コード行の後のsection metaも新しいsectionを開始する（改行は1個でも可）
    - 最初のコード行より前の空行ではsectionを分割せず、先行するsection metaを保持する
  - 制約
    - chord metaは対象chordの直前に置く。`C[key=A]`のような後置は不可
    - section metaは専用行に置く
    - pipeを使う`|C|`記法には対応しない

## 重複指定の警告

`C(9,9)` は有効な表記として受理し、AST の重複も保持します。同じコードの
拡張指定で同一の値が2回以上出現すると、2個目以降それぞれに
`DUPLICATE_EXTENSION` の警告を付けます。`9` と `add9` などの音楽的な意味の
重複は判定しません。

JavaScript API は成功・失敗のどちらにも `warnings` 配列を返します。
警告だけでは解析に失敗せず、警告がなければ空配列になります。警告の位置は
元の入力に対するもので、エラーと同じ行・列および UTF-16 offset を使います。
構文エラーを含むコードで検出した重複も警告として返しますが、拡張指定を許さない
`%`・`_`・`?` や、テキストとして保持するスラッシュ分母は対象にしません。

各 npm パッケージの `warning_code_message_map` サブモジュールから
`getWarningMessage({ warningCode: warning.code, lang: "ja" })` を使って
表示用メッセージを取得できます。`lang` は `"ja"` または `"en"` です。
ブラウザから直接 import する場合は `.js` を付けます。

## Extension の有効値

受理する値は以下の30種類です。大文字・小文字は区別され、
[`Extension`](../../resources/generatedTypes.ts) enum と一致します。

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

extension 自体は省略可能ですが、指定する場合はコード本体の直後の空でない `(...)` 内に
必ず記述します。複数の値はカンマで区切り、括弧のグループは1つにまとめます。
分数コードの `/E` などは `C(7)/E` のように括弧の後に置きます。
コード種別の `m`、`M`、`aug`、`dim` は extension ではなく、コード本体に記述します。

`1` は `C(1)` のように、ルート単音を意図する表記に使えます。
parser はこの指定を `extensions` に保存し、コード本体の種別は保持します。
構成音の算出や extension の組み合わせの音楽的な検証は行わず、実音の解釈は利用側に委ねます。

canonical syntax として有効な例：

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

以下の一般的な表記は canonical syntax ではなく、parse に失敗します：

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

機械的に確認できる [fixture](../../tests/fixtures/canonical_syntax.json) も参照してください。
