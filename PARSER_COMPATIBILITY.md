# Parser compatibility policy

作成日: 2026-08-31

この文書は、新旧 parser の差分を判定する基準を定める。文法の構造は [_docs/chord-progression.ebnf](./_docs/chord-progression.ebnf) を正とする。

## 維持する公開契約

- Rust API は `parse_chord_progression_string(&str)` を提供する。
- JavaScript API は `parseChordProgressionString(string)` を提供する。
- 既存のAST JSONの型・フィールドは維持する。空行によるsection構成の変更と参照用の新しいtagの追加は、後述の契約変更に従う。
- syntax error は戻り値として返し、ユーザー入力によって panic または JavaScript throw を発生させない。
- `lineNumber` と `columnNumber` は 1 始まりとする。
- syntax error は入力順の配列で返し、安全に再開できるcomma、bar、改行の境界から解析を継続する。
- extensionの括弧が壊れていてもbar区切りで再開し、後続chordのerrorやwarningを独立して判定する。
- `startOffset` と `endOffset` は JavaScript の文字列・editor APIに合わせた0始まりのUTF-16 offsetとし、`endOffset`は範囲末尾の次を指す。
- 空入力は空の AST を返す。

## 正式に維持する構文

| 入力上の表現 | 意味 |
| --- | --- |
| `@section=Verse` | section名 |
| `@repeat=2` | sectionの繰り返し回数 |
| `@same=Verse` | 名前で指定したsectionのコード進行への未解決の参照 |
| `[key=C]C` | 後続chordに属するkey情報 |
| `C-D` | 2個のbar |
| `C,D` | 1個のbar内の2個のchord |
| 1個の改行 | 同一section内の `ChordBlock::Br` |
| chord line間の2個以上の連続改行 | 同一section内の空行1行（`ChordBlock::Br`を2個） |
| `C/B` | denominatorが`B`のchord |
| `?` | 未特定chord |
| `_` | no chord |
| `%` | 同一section内の先行chordを参照するsame chord |

- spaceとtabは構造記号の前後、およびextensionのcomma前後で許可する。
- chordを構成するbase、accidental、chord typeの途中には空白を許可しない。
- denominatorは今回も文字列として保持する。ただし空文字列と2個目のslashは拒否する。
- extensionは定義済み文字列との完全一致だけを許可する。
- bar末尾のcommaは1個まで受理し、formatterは省略する（`C/D,`、`C,-D`など）。extensionリストの末尾commaは許可しない。
- 複数のsection metaは、通常はchord lineが始まるまで同じsectionへ属する。`@same`で参照が完成した後の境界は「セクション参照の追加」に従う。
- section metaと最初のchord lineの間にある空行はsectionを分割しない。
- 文書の先頭・末尾とsectionの最初のchord lineより前の改行・空行は省略する。

## 空行の契約変更（0.10.0から）

- 空行だけではsectionを分割せず、section名・key・`%`の先行chord参照を継続する。
- chord line間の空行は既存の`ChordBlock::Br`を2個並べて保持し、複数の空行は1行へまとめる。ASTの型・JSONフィールドは増やさない。
- chord lineの後にsection metaが現れると、新しいsectionを開始する。明示した新sectionでは`%`の参照をリセットする。参照だけのsectionの境界は「セクション参照の追加」に従う。
- formatterも同一section内の空行を1行保持する。編集中の入力原文を自動的に書き換える契約ではない。
- 0.9.4以前のASTで空行から作られた匿名sectionは、新しいparserの同一sectionと一致しない。利用側の更新時は原文からAST・section別の派生データ・検索索引を再生成する。旧ASTを新formatterへ渡すと、同じ構造へ往復できないため拒否される場合がある。

## セクション参照の追加（0.10.0から）

- `@same=名前`を`SectionMeta::Same(String)`、JSONでは`{ "type": "same", "value": "名前" }`として保持する。
- 参照先の存在確認・コード進行への展開・重複名からの候補選択は行わない。未定義・前方・自己・循環参照もそのまま保持し、解決方法は利用側に委ねる。
- `@section`で参照側の名前を付ける場合は`@same`より前へ置く。`@repeat`は`@same`の前後に置ける。
- 参照だけのsectionも独立したsectionとして返す。`@same`の後の`@section`・`@same`は新sectionを開始し、`@repeat`だけは同じ参照側へ付く。
- `@same`と同じsectionへのコード本文の併記は構文エラー`SMIV-4`とする。拒否した本文は参照へ混ぜず、後続sectionの解析を継続する。
- 値の必須性・単一値の読み取り・専用行・診断位置は既存のsection metaと同じ契約を使う。formatterは参照の指定をそのまま出力してASTを往復させる。
- 利用側は`SectionMeta`の判別可能unionへ追加された`same`を扱う必要がある。アプリケーションへの取り込みは別作業とし、この変更では行わない。

## 意図して修正する挙動

| 入力 | 旧挙動 | 新しい契約 |
| --- | --- | --- |
| `C(` | panic | position付きerror |
| `@section` | panic | position付きerror |
| `[key=C` | panic | position付きerror |
| `C[key=A]` | metaを黙殺して成功 | token順序error |
| chord line間の空行 | section境界 | 同じsection内の空行1行として保持 |
| 3個以上の連続改行 | `BL-1` | 同じsection内の空行1行へまとめて許可 |
| 未完の括弧・角括弧・slash | 一部でpanicまたは不定なerror | position付きerror |
| 2個目のslash | `DEN-1` | `DEN-2` |
| 定義文字列のprefixだけが一致するextension | 経路によって受理可能 | `EXT-1` |

panic、入力の黙殺、入力範囲外のerror positionは互換対象にしない。

## 非対応として明確化する構文

- `|C|` のpipe記法は正式構文に含めない。
- chord後方のmeta情報は許可しない。meta情報は必ず対象chordの直前に置く。
- section metaは専用行に置き、値の後には改行またはEOFが必要である。
- `%` は同一section内に先行chordがない場合は許可しない。
- 先行するchord lineに構文エラーがあっても、section metaによるsection境界で `%` の参照状態をリセットする。空行ではリセットしない。

## 今回追加しない機能

- `-5` を `b5` のaliasとして扱う機能
- denominatorのchordまたはdegreeへの構造化
- AST nodeへのsource span追加
- pipe記法やescape構文

## 差分判定

新旧parserの比較結果は次の順で扱う。

1. この文書の「意図して修正する挙動」に該当すれば、新parserの結果を採用する。
2. 正式に維持する構文のASTが異なれば、新parserの回帰として修正する。
3. どちらにも該当しない差分は、仕様を追記するまで公開APIを切り替えない。
