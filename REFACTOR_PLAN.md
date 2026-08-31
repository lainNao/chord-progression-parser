# Rust 解析基盤リファクタリング計画

作成日: 2026-08-31

関連資料: [REFACTOR_AUDIT.md](./REFACTOR_AUDIT.md)

## 結論

crate 全体を捨てて作り直すのではなく、外部契約を維持したまま解析パイプライン内部を準リライトする。

- 当面維持するもの
  - Rust の `parse_chord_progression_string` 関数名と戻り値の意味
  - JavaScript の `parseChordProgressionString` 関数名
  - 成功時の AST JSON 形式
  - 既存の chord、extension、section meta、chord meta の表現
- 置き換えるもの
  - 文脈依存の token を生成する現在の tokenizer
  - 1 個の巨大な状態機械になっている現在の parser
  - `unwrap` を前提とした構文エラー処理
- 別の判断として後回しにするもの
  - AST への位置情報追加
  - 複数エラーの同時返却
  - denominator の型変更
  - エラーコード体系の全面変更

AST の形は解析処理の複雑さの主因ではない。一方、tokenizer と parser の責務混在は局所修正では解消しにくい。このため「公開面は互換、内部は新規実装」が最も費用対効果がよい。

## 現在の設計をそのまま分割しない理由

現在の `Token` は `MetaInfoKey`、`SectionMetaInfoValue`、`Denominator` など、字句ではなく構文上の意味をすでに持っている。tokenizer が parser の一部を先回りしているため、parser は token 列を信用して `unwrap` し、同じ制約を再検証している。

現在の `parse` を小関数へ機械的に分けても、次の問題が残る。

- tokenizer と parser の二重状態管理
- 後続 token を各分岐が勝手に消費する構造
- 一時メタ情報がどの chord に属するかという暗黙状態
- 不正な token 列を型として表現できてしまう問題
- tokenizer 単体テストと parser 単体テストで同じ仕様を二重に固定する問題

したがって、既存関数からの抽出を最終形にはしない。

## 目標アーキテクチャ

```text
input: &str
  -> Lexer
       構造記号、改行、テキスト片と位置だけを生成
  -> Parser
       文法に従って Section / Bar / ChordInfo を構築
       chord parser で chord 本体、種類、extension を完全一致で解釈
       重複、配置、前後関係などを AST 構築時に検証
  -> Ast
  -> Rust API / wasm adapter
```

### Lexer

Lexer は入力の意味を推測しない。token は概ね以下に限定する。

```rust
enum TokenKind<'src> {
    At,
    LeftBracket,
    RightBracket,
    LeftParen,
    RightParen,
    Equal,
    Comma,
    Slash,
    Dash,
    Newline,
    Text(&'src str),
}
```

各 token は source span を持つ。`SectionMetaInfoKey` や `Denominator` のような文脈依存 token は作らない。空白は改行と区別して読み飛ばし、CRLF は 1 回の改行として扱う。

`-` は chord 区切りと extension の別名候補 `-5` が衝突するため、Lexer では一律 `Dash` とし、意味は Parser が位置に応じて決める。

### Parser

`Parser<'src>` が token slice と cursor を所有する。各関数は 1 つの文法単位だけを担当する。

```text
parse_document
parse_section
parse_section_meta
parse_chord_line
parse_bar
parse_chord_info
parse_chord_meta
parse_chord_expression
parse_extensions
parse_denominator
```

`next().unwrap()` は使わず、`peek`、`advance`、`expect` が EOF を通常の構文エラーへ変換する。AST への追加は `parse_*` の戻り値を上位が所有する形にし、`sections.last_mut()` を各所から操作しない。

### Chord parser

chord 本体の解釈は token の走査から分離する。

- base と accidental を先頭から読む
- chord type は候補の優先順を明示する
- extension は文字列の完全一致で解釈する
- 入力の一部だけ一致した場合は成功させない
- `FromStr` を実装し、失敗理由を型付きエラーで返す

### Error と位置

内部では `ParseError { kind, span, additional_info }` を使い、公開境界で既存の `ErrorInfoWithPosition` に変換する。

- 行・列は現在と同じ 1 始まり
- 列と length は Unicode scalar value 単位
- EOF エラーは入力末尾の 0 長 span を許可
- syntax error と内部エラーを分離
- ユーザー入力に対して panic しない

既存エラーコードは既知ケースで可能な限り維持する。ただし、現在 panic または黙殺になる入力は互換対象にしない。意味に合わない既存コードは、変更理由を compatibility matrix に記録して直す。

## 実施フェーズ

### Phase 0A: 品質ゲートの基準整備

現在は既存警告により `clippy -D warnings` が失敗するため、挙動を変えない範囲の修正を先に行う。

- deprecated な `EnumVariantNames` と `JsValue::from_serde` を置き換える
- noop clone と空文字列比較を直す
- Rust API の `Result` 分解を `?` に置き換える
- wasm の戻り値が変わらないことを既存 E2E で確認する

完了条件:

- `cargo fmt --all -- --check` が通る
- `cargo clippy --all-targets --all-features -- -D warnings` が通る
- Rust test と 3 種の package E2E が従来どおり通る

### Phase 0: 仕様と互換範囲の固定

実装前に、ドキュメントと現在挙動の食い違いを一覧化する。

- 簡潔な EBNF を追加する
- 成功入力、失敗入力、現在の不具合を compatibility matrix にする
- README、日英構文資料、テスト、generator の入力例を corpus 化する
- 次の曖昧点を決定する
  - `|C|` 記法を正式に許可するか
  - 空行が section と改行を分ける規則、および 3 個以上を拒否する仕様を残すか
  - chord 後置の `[key=A]` をどのエラーにするか
  - `%` が参照できる「前の chord」の範囲
  - denominator を任意文字列のまま許可する範囲
  - extension の `-5` alias を今回含めるか
  - 空白と tab を許可する位置
- 既存の正常系 AST を public API 経由の golden test で固定する
- 既知の panic 入力を regression corpus に入れる

完了条件:

- 仕様として維持する挙動と、修正する挙動が文書上で区別されている
- 新しい契約テストが公開 API へ入力文字列を渡している
- 手書き token 列に依存する旧 parser テストは、旧実装削除まで既存挙動の記録として残す

### Phase 1: 新 Lexer の追加

既存 tokenizer は残したまま、新 Lexer を独立モジュールとして追加する。

- 借用文字列を使い、不要な `String` clone を避ける
- ASCII、Unicode、CRLF、末尾 EOF の span テストを追加する
- 記号が連続する入力でも panic しないことを確認する
- 文脈依存 token を追加しない

完了条件:

- Lexer 単体で任意の UTF-8 入力を panic せず走査できる
- 各 token の span が、対応する入力断片を正確に指している
- `cargo clippy --all-targets --all-features -- -D warnings` が通る

### Phase 2: 新 Parser と chord parser の追加

既存 AST 型を出力する新 Parser を追加する。この段階では公開 API からはまだ呼ばない。

- 文法単位ごとの `parse_*` 関数を実装する
- meta 情報は `parse_chord_info` のローカル値として所有する
- bar への chord 追加を 1 経路に統一する
- chord type と extension の完全一致解析を実装する
- すべての構文エラーを `Result` で返す
- 関数の契約と判断が分かりにくい箇所へコメントを追加する

完了条件:

- Phase 0 の成功 corpus をすべて AST に変換できる
- Phase 0 の失敗 corpus が panic せずエラーになる
- production code の入力依存経路に `unwrap` / `expect` がない

### Phase 3: 敵対的検証と差分テスト

旧実装と新実装を同じ入力で動かし、差分を分類する。

- documented valid corpus は AST の完全一致を要求する
- generator の大量生成入力で differential test を行う
- 任意の UTF-8 文字列に対して「panic しない」property test を行う
- 括弧、角括弧、slash、comma、改行の欠落・過剰を系統的に生成する
- `C(`、`@section`、`[key=C`、`C[key=A]` を明示的に検証する
- 位置情報を実際の入力上でハイライトし、範囲外にならないことを検証する

差分は次の 3 種類に分ける。

1. 意図した修正: compatibility matrix に理由を記録する
2. 旧実装の未文書挙動: 仕様を決めてから採否を判断する
3. 新実装の回帰: 切替前に修正する

完了条件:

- 許可リストにない正常系 AST 差分が 0 件
- property test で panic が 0 件
- エラー位置がすべて入力範囲内

### Phase 4: 公開 API の切替

`parse_chord_progression_string` を新 Lexer / Parser へ切り替える。

- Rust API は `?` でエラーを伝播する
- 公開型を crate root から再 export し、利用者が戻り値型を名前で参照できるようにする
- wasm の成功・失敗レスポンスを enum/struct として定義し、手組みの `serde_json::Value` を廃止する
- syntax error は従来どおりレスポンス値として返し、serialization failure だけを内部エラーとして区別する
- node、bundler、web の生成物で成功・失敗の両方を E2E テストする

完了条件:

- Rust と 3 種の npm package で同一 AST が返る
- malformed input が JavaScript 側へ panic/throw を漏らさない
- 生成 TypeScript 型と実際の JSON に `null` / optional の不一致がない

### Phase 5: 旧実装削除と配置整理

差分検証が終わった後でのみ旧 tokenizer / parser を削除する。

想定配置:

```text
src/
  lib.rs
  error.rs
  lexer.rs
  parser/
    mod.rs
    chord.rs
  model/
    mod.rs
    ...
  wasm.rs
```

- 旧 token 型と token 列を直接組み立てるテストを削除する
- model 型を解析処理から独立させる
- `error_code.rs` の生成元を正とし、生成手順を CI で検証する
- 不要 dependency と dead code を削除する
- README と日英構文資料を実装に合わせる

完了条件:

- 旧解析経路が残っていない
- `cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features -- -D warnings`、全テストが通る
- wasm build と node/bundler/web E2E が通る

### Phase 6: AST/API v2 の要否を再評価

内部リライト後に、実際の利用要件を根拠として別計画にする。Phase 0 から同時に変更しない。

候補:

- AST ノードまたは token 一覧への source span 付与
- denominator の構造化
- 複数エラー返却とエラー回復
- `Br` を AST に残すか、表示情報として分離するか
- `Ast = Vec<Section>` を document struct にするか
- error code と human-readable message の責務分離

これらは JSON と生成 TypeScript 型を壊す可能性が高いため、必要なら version 1.0 の変更として扱う。

## PR 分割

1. 既存警告の解消と品質ゲート整備
2. 仕様、compatibility matrix、public golden test
3. 新 Lexer と span test
4. 新 Parser、chord parser、単体テスト
5. differential/property test と判明した修正
6. 公開 API / wasm 切替と全 package E2E
7. 旧実装削除、module 整理、ドキュメント更新

各 PR は単独で lint と既存テストを通す。新旧実装の併存は一時的に許可するが、公開切替後の次 PR で必ず旧実装を削除する。

## 非目標

- parser generator や大規模 framework の導入
- 現在の parser を抽象化層で包んで延命すること
- 互換性のために既知の panic や token 黙殺を残すこと
- リファクタと同時に全 syntax feature を増やすこと
- 行数だけを目的にしたファイル分割

現在の文法規模では、source span 付きの小さな Lexer と手書き Parser で十分である。外部 parser library は依存と独自エラー制御の複雑さが増えるため、文法が大きくなる兆候が出るまでは導入しない。

## 品質ゲート

通常の完了確認:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
make build-wasm-web
make build-wasm-node
make build-wasm-bundler
make test-resources
make test-e2e
```

追加の判断基準:

- 公開入力による panic がない
- documented valid corpus の AST 互換率が 100%
- 意図しない入力黙殺がない
- エラー span が入力範囲外にならない
- 代表入力で大幅な速度低下や wasm サイズ増加があれば原因を説明する
- 新しい production 関数には役割と前提を示すコメントがある

## 最初に着手する内容

最初の PR は Phase 0A に限定し、以降の全 PR で同じ品質ゲートを使える状態にする。その次の実装 PR は Phase 0 に限定する。

1. 現在の入力例を `tests/fixtures` に集約する
2. 公開 API 経由の golden test を追加する
3. panic・黙殺・曖昧仕様を compatibility matrix に記録する
4. EBNF を日英ドキュメントの共通基準として追加する

この土台ができるまで新 Parser は書き始めない。先に旧挙動を無差別に固定するのではなく、「仕様として守る挙動」と「修正対象」を分けることが重要である。
