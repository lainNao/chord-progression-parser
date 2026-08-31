# Rust リファクタリング事前監査

作成日: 2026-08-30

## 目的

Rust 実装をすぐ書き換える前に、改善対象を複数回探索し、それぞれを敵対的に検証した結果をまとめる。

## 実行した確認

- `cargo test`
  - 結果: 成功。unit/doc test 合計 53 件が通過。
  - 警告: deprecated macro、deprecated `JsValue::from_serde`、noop clone。
- `cargo fmt --all -- --check`
  - 結果: 成功。
- `cargo clippy --all-targets --all-features -- -D warnings`
  - 結果: 失敗。既存警告が `-D warnings` でエラー化。
- 公開 API `parse_chord_progression_string` への境界入力プローブ
  - `C(`、`@section`、`[key=C` で `Err` ではなく panic を確認。

## Round 1: parser の巨大状態機械

### 改善候補

[src/parser/mod.rs](./src/parser/mod.rs) の `parse` が 800 行超の単一関数になっている。セクションメタ、コードメタ、コード、拡張、分母、改行、区切りの処理が同じループと同じ可変状態に混在している。

特に以下が目立つ。

- `sections.last_mut().unwrap()`、`token_with_position_list.next().unwrap()`、`peek().unwrap()` が多い。
- `tmp_chord_info_meta_infos` の寿命が暗黙的で、どの chord に適用されるかを追いにくい。
- `Token::Chord` の通常コードと `?` / `%` / `_` で、Bar への追加処理が重複している。
- `Token::Extension` の処理が後続 token を内部で消費するため、外側の while ループの状態遷移が局所的に追えない。

### 敵対的検証

単に関数が長いだけなら、テストで十分守られていれば即リファクタ対象とは限らない。そこで既存テストと公開 API の挙動を確認した。

- `cargo test` は通るため、主要な正常系と一部の異常系は守られている。
- ただし公開 API 経由で以下が panic する。
  - `C(`: [src/parser/mod.rs](./src/parser/mod.rs) の `Token::ExtensionStart` 処理で `peek().unwrap()`。
  - `@section`: section meta の `Equal` 期待箇所で `next().unwrap()`。
  - `[key=C`: chord meta の `MetaInfoEnd` 期待箇所で `next().unwrap()`。

### 結論

これはスタイル問題ではなく、公開 API が malformed input で panic する品質問題。最優先で、parser 内部に「期待 token を読む小さな関数」を作り、EOF を `ErrorInfoWithPosition` に変換する形へ寄せるべき。

## Round 2: tokenizer と parser の責務境界

### 改善候補

[src/tokenizer/mod.rs](./src/tokenizer/mod.rs) が字句解析だけでなく、構文・ドメイン寄りの判定まで行っている。

例:

- 改行時に `SectionMetaInfoKey`、`MetaInfoKey`、`MetaInfoValue`、`Comma` の構文エラーを返す。
- `Token::Equal` の直前 token を見て、section meta value か chord meta value かを決める。
- chord token に数字または `o` が含まれると `CHO-1` にする。
- 分母だけは `is_reading_extension` で括弧内を特別扱いする。

### 敵対的検証

tokenizer が構文まで持つ設計でも、小さい言語なら実装量を減らせる可能性がある。既存テストも tokenizer の期待値をかなり細かく持っているため、現状の挙動は一応固定されている。

一方で、parser 側にも extension validation や meta validation があり、責務はすでに二重化している。さらに公開 API の panic は tokenizer では防げていない。つまり「tokenizer が先に不正入力を弾くから parser は unwrap してよい」という前提は成立していない。

### 結論

tokenizer は `TokenWithPosition` の生成に寄せ、構文上の期待順序は parser に集約するのがよい。いきなり全面移動すると差分が大きいので、まずは parser の EOF/panic 対策を入れ、その後に tokenizer の構文エラーを parser に段階移管する。

## Round 3: chord detail / extension 解析

### 改善候補

[src/parser/types/chord_detailed.rs](./src/parser/types/chord_detailed.rs) の `ChordDetailed::from_str` は実装が短い一方で、暗黙のルールが多い。

- `starts_with('m')`、`starts_with('M')`、`starts_with("aug")`、`starts_with("dim")` で chord type を決めている。
- extension は `Extension::VARIANTS` を長さ順に並べ、`extension_str.starts_with(candidate)` で解決している。
- `Extension::from_str(...).unwrap()` が残っている。
- `Extension::VARIANTS.clone()` は noop clone として警告が出ている。

### 敵対的検証

`starts_with` は `C(111)` を `11` と誤認する懸念がある。しかし公開 API プローブでは `C(111)` と `C(9,111)` はどちらも `EXT-1` になった。これは通常の入力では tokenizer が括弧内を `Extension("111")` として渡し、parser 側の `Extension::from_str` で落としているため。

ただし `ChordDetailed::from_str` 単体では `starts_with` の曖昧さが残る。今後 parser の構成を変えて `C(...)` 全体を直接 `ChordDetailed::from_str` に渡す経路が増えると、誤受理の温床になる。

### 結論

単体関数としては「完全一致で extension を解決する」実装に直すべき。併せて `impl FromStr for ChordDetailed` に寄せると、Rust の慣用にも合う。これは公開 API の panic 対策よりは優先度を下げてよいが、parser リファクタの前に固めたい。

## Round 4: wasm 変換と公開 API

### 改善候補

[src/lib.rs](./src/lib.rs) の wasm 関数は `serde_json::json!` を組んだあと `JsValue::from_serde(&json_result).unwrap()` を呼んでいる。

また Rust API の `parse_chord_progression_string` は、`Result` を `is_err` / `unwrap` / `err().unwrap()` で分解している。

### 敵対的検証

`parse_chord_progression_string` 内の unwrap は直前で `is_err` を見ているため、そこ自体が直ちに panic する可能性は低い。読みづらさと保守性の問題が中心。

一方で wasm 側の `JsValue::from_serde` はコンパイラ警告でも deprecated と出ており、コメントにも既知の FIXME がある。公開 JS API はこの crate の重要な利用面なので、警告を放置するとリリース時の信頼性を落とす。

### 結論

Rust API は `let tokens = tokenize(input)?; parse(&tokens)` に直す。wasm API は `serde_wasm_bindgen` に寄せるか、JS 側の返却形式要件を明文化して panic しない変換にする。`unwrap` で落とすなら、少なくとも「変換不能時は `OTHER-1` 形式を返す」など公開 API としての方針を決める。

## Round 5: 仕様と実装のズレ

### 改善候補

ドキュメント上、ChordInfo は `[key=value]Chord(Extension)` 形式と書かれている。一方、公開 API プローブでは `C[key=A]` が `Ok` になり、後置メタ情報が無視される。

また [TODO.md](./TODO.md) にも `C[key=A]` はバリデーションで落とす旨のメモがある。

### 敵対的検証

後置メタ情報を許可する仕様とも解釈できなくはない。しかし現状の AST では `C[key=A]` の `key=A` が chord にも section にも反映されない。許可ではなく黙殺になっているため、利用者には危険。

### 結論

`C[key=A]` のような chord 後の meta info は明示的にエラーにするべき。これは小さめの修正で実害を減らせるため、panic 対策の次に着手しやすい。

## 優先順位

1. 公開 API での panic をなくす。
   - `next().unwrap()` / `peek().unwrap()` を期待 token 読み取り関数に置き換える。
   - EOF や空 extension を既存 `ErrorCode` に落とす。
2. `C[key=A]` の黙殺をエラー化する。
   - 仕様にない token 順序を parser 側で検出する。
3. `parse_chord_progression_string` の Result 処理を `?` に置き換える。
   - 低リスクで読みやすさが上がる。
4. clippy の機械的警告を解消する。
   - `EnumVariantNames` から `VariantNames`。
   - `JsValue::from_serde` の置き換え方針決定。
   - noop clone と empty string 比較の修正。
5. `ChordDetailed::from_str` を完全一致ベースにする。
   - `starts_with` に依存した extension 解決をやめる。
   - `impl FromStr` を検討する。
6. tokenizer/parser の責務を段階的に整理する。
   - tokenizer は文字列から token への変換に寄せる。
   - 構文上の期待順序とドメイン validation は parser に寄せる。
7. parser の Bar 追加処理を共通化する。
   - `ChordInfo` 生成と「comma なら同じ Bar、そうでなければ新 Bar」を小関数化する。
   - `tmp_chord_info_meta_infos` は `std::mem::take` で所有権を移し、不要 clone を減らす。

## 次に実装するなら

最初の PR は「公開 API の panic 排除」に絞るのがよい。理由は、利用者影響が明確で、既存 AST の構造を変えずに進められるため。

推奨する進め方:

1. `parse` 内に `unexpected_eof` と `expect_next_token` 相当の小さな helper を追加する。
2. section meta、chord meta、extension start の `unwrap` を置き換える。
3. `C(`、`@section`、`[key=C` の regression test を追加する。
4. `cargo test`、`cargo fmt --all -- --check`、可能なら `cargo clippy --all-targets --all-features` を実行する。
