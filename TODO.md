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
- 公開パッケージのimportファイル名をcamelCaseへ変更するか判断する
  - 既存のimportとCDN URLを壊すため、破壊的変更として扱う

## 外部の文書

- Zenn の外部記事を現行仕様に合わせて更新する（原稿はこのリポジトリ外）
