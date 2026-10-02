# TODO

## 仕様

- 次期AST/APIで破壊的変更を行うか判断する
  - denominatorを文字列のまま扱うか、コードまたはディグリーとして構造化する
  - ASTまたはtoken一覧へsource spanを付与するか
  - error codeと表示用メッセージの責務を分離するか
  - 詳細は[REFACTOR_PLAN.md](./REFACTOR_PLAN.md#phase-6-astapi-v2-の要否を再評価)を参照
- `-5`を`b5`の別名として受理するか判断する
- エスケープ構文が必要か判断する
- 同じ対象へメタ情報が複数指定された場合の仕様を決め、テストを追加する
  - →memo: メタ情報が固定文字列ならばそれぞれの文字列に対して仕様を決めたいが、自由文字列ならば重複ありとしたい（配列みたいに複数指定できるようにしても妥当だと思うし）
- 公開パッケージのimportファイル名をcamelCaseへ変更するか判断する
  - 既存のimportとCDN URLを壊すため、破壊的変更として扱う
- WASM の例外処理を `panic=unwind` に変更するか判断する
  - 例外を投げる getter や revoked Proxy は、現行ビルドでは Rust の解放処理を飛ばす
  - 対応には nightly と WASM exception handling の対応環境が必要になるため保留
  - 詳細と再現方法は[開発ドキュメント](./_docs/ja/how-to-develop.md#javascript-の例外と対応環境)を参照
