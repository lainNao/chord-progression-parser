# 開発の仕方

> 現在、開発環境は`MacOS`か`Linux`環境のみです。`Windows`の場合は`WSL`かそれに準ずるものを使わないと動きません。

## 環境構築

`.github/workflows/check-not-broken.yml`と`Makefile`を参照してください。
あと以下のコマンドを実行してください。

```bash
make install
```

## 診断コード

`resources/error_code_message_map.ts` または `resources/warning_code_message_map.ts`
を編集し、`make generate-diagnostic-codes` を実行してください。Rust のエラーコードと
warning コードは、このメッセージ表から生成します。`make check-local` で生成結果との一致を確認します。

## プルリクエスト

現在はルールはありません。
どんなプルリクエストでも歓迎です。
ブランチルールはまだ決まっていません。

## リリース

`Cargo.toml`の`version`を上げて`main`ブランチにプッシュされると、自動でタグがつけられてリリースされます。

npmパッケージはTrusted Publishingを使います。各パッケージのnpm設定では、GitHubリポジトリを`lainNao/chord-progression-parser`、workflowを`test-and-release.yml`として登録してください。

リリースが一部だけ失敗した場合は、GitHub Actionsから`test-and-release`を開き、既存タグを`tag-to-release`に指定して再実行します。公開済みの成果物はスキップされます。

公開済みの判定は npm または crates.io に直接問い合わせます。通信障害やレジストリの
エラーではジョブを停止し、HTTP 404 の場合だけ未公開と判断します。過去のタグを再実行
するときも、公開済み判定にはワークフロー側のリリース用ツールを使います。

GitHub Release は npm の3パッケージと crates.io の公開ジョブがすべて成功した後に作成されます。
