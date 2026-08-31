# Parser compatibility policy

作成日: 2026-08-31

この文書は、新旧 parser の差分を判定する基準を定める。文法の構造は [_docs/chord-progression.ebnf](./_docs/chord-progression.ebnf) を正とする。

## 維持する公開契約

- Rust API は `parse_chord_progression_string(&str)` を提供する。
- JavaScript API は `parseChordProgressionString(string)` を提供する。
- 成功時の AST JSON 形式はリファクタ前と一致させる。
- syntax error は戻り値として返し、ユーザー入力によって panic または JavaScript throw を発生させない。
- `lineNumber` と `columnNumber` は 1 始まりとする。
- 空入力は空の AST を返す。

## 正式に維持する構文

| 入力上の表現 | 意味 |
| --- | --- |
| `@section=Verse` | section名 |
| `@repeat=2` | sectionの繰り返し回数 |
| `[key=C]C` | 後続chordに属するkey情報 |
| `C-D` | 2個のbar |
| `C,D` | 1個のbar内の2個のchord |
| 1個の改行 | 同一section内の `ChordBlock::Br` |
| 2個以上の連続改行 | section境界 |
| `C/B` | denominatorが`B`のchord |
| `?` | 未特定chord |
| `_` | no chord |
| `%` | 同一section内の先行chordを参照するsame chord |

- spaceとtabは構造記号の前後、およびextensionのcomma前後で許可する。
- chordを構成するbase、accidental、chord typeの途中には空白を許可しない。
- denominatorは今回も文字列として保持する。ただし空文字列と2個目のslashは拒否する。
- extensionは定義済み文字列との完全一致だけを許可する。
- 複数のsection metaは、chord lineが始まるまで同じsectionへ属する。
- section metaと最初のchord lineの間にある空行はsectionを分割しない。

## 意図して修正する挙動

| 入力 | 旧挙動 | 新しい契約 |
| --- | --- | --- |
| `C(` | panic | position付きerror |
| `@section` | panic | position付きerror |
| `[key=C` | panic | position付きerror |
| `C[key=A]` | metaを黙殺して成功 | token順序error |
| 3個以上の連続改行 | `BL-1` | 余分な空行として許可 |
| 未完の括弧・角括弧・slash | 一部でpanicまたは不定なerror | position付きerror |
| 定義文字列のprefixだけが一致するextension | 経路によって受理可能 | `EXT-1` |

panic、入力の黙殺、入力範囲外のerror positionは互換対象にしない。

## 非対応として明確化する構文

- `|C|` のpipe記法は正式構文に含めない。
- chord後方のmeta情報は許可しない。meta情報は必ず対象chordの直前に置く。
- section metaは専用行に置き、値の後には改行またはEOFが必要である。
- `%` は同一section内に先行chordがない場合は許可しない。

## 今回追加しない機能

- `-5` を `b5` のaliasとして扱う機能
- denominatorのchordまたはdegreeへの構造化
- 複数errorの同時返却
- AST nodeへのsource span追加
- pipe記法やescape構文

## 差分判定

新旧parserの比較結果は次の順で扱う。

1. この文書の「意図して修正する挙動」に該当すれば、新parserの結果を採用する。
2. 正式に維持する構文のASTが異なれば、新parserの回帰として修正する。
3. どちらにも該当しない差分は、仕様を追記するまで公開APIを切り替えない。
