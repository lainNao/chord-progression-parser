# TODO

## 周辺ツール

- `chord-progression-hub`をv0.9.0以降へ更新する
  - parser依存を更新し、`Es_M`と`Es_m`を音高6へ対応付ける
  - 生成済みの検索用データがあれば再計算する
  - 検索設計ドキュメントへ、原文表記と検索用表現を分ける方針を反映する
- generatorを別リポジトリへ移し、GitHubでリリースする
  - 現時点では内部利用のみのため、npmへの公開は必要になってから検討する

## 仕様

- 次期AST/APIで破壊的変更を行うか判断する
  - denominatorを文字列のまま扱うか、コードまたはディグリーとして構造化する
  - ASTまたはtoken一覧へsource spanを付与するか
  - error codeと表示用メッセージの責務を分離するか
  - 詳細は[REFACTOR_PLAN.md](./REFACTOR_PLAN.md#phase-6-astapi-v2-の要否を再評価)を参照
- `-5`を`b5`の別名として受理するか判断する
- エスケープ構文が必要か判断する
- 同じ対象へメタ情報が複数指定された場合の仕様を決め、テストを追加する
- README、日英ドキュメント、Zennの記事を現行仕様に合わせて見直す

## 保守

- リリース時にREADME内のCDNバージョンを自動更新する
- 公開パッケージのimportファイル名をcamelCaseへ変更できるか調査する
- commit messageの規約をCIまたはcommit hookで検証するか判断する
- wasm固有の単体テストがE2Eとは別に必要か判断し、結論を開発ドキュメントへ残す

## 品質・パフォーマンス

- WASM の formatter で、不正な配列値を繰り返し渡すと参照管理用の領域が増え続ける
  - `make build-wasm-web && node _tools/audit-wasm-references.mjs` で再現できる
  - この診断コマンドは、ウォームアップ後も参照テーブルが拡大すると終了コード 1 を返す
  - `null` の AST、section の `metaInfos: null`、bar の `value: null` などが対象
  - 正常な AST と通常の型エラー `{}` では、同じ負荷で参照枠が再利用される
  - `wasm-bindgen` 0.2.127 / 0.2.129 と `serde-wasm-bindgen` 0.6.5 で確認済み
  - parser を除いた `js_sys::Reflect::get(null, ...)` でも、捕捉した例外ごとに参照枠が 1 つ残る
  - 生成された import shim が例外後の `undefined` に参照枠を割り当て、Rust 側の例外分岐がその戻り値を回収しない
  - 依存側での修正を確認して回帰テストに組み込む。生成された WASM の書き換えや AST スキーマの二重実装は避ける
- カバレッジを計測し、不足している重要な分岐へテストを追加する
- 再現可能なベンチマークを追加し、継続的に性能を比較できるようにする
- profilerまたはbenchmarkの結果を根拠に、不要なcloneなどを改善する
