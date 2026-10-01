# 開発の仕方

> 現在、開発環境は`MacOS`か`Linux`環境のみです。`Windows`の場合は`WSL`かそれに準ずるものを使わないと動きません。

## 環境構築

事前に `rustup` で Rust を導入し、Bun もインストールしてください。
`make install` は WASM のターゲットと開発ツールを導入し、`wasm-pack` が
見つからない場合はそれもインストールします。

`.github/workflows/check-not-broken.yml`と`Makefile`を参照してください。
あと以下のコマンドを実行してください。

```bash
make install
```

環境構築・チェック・生成ツールのテスト・npm ビルドでは、
`bun install --frozen-lockfile` を使います。JavaScript の依存関係を変更したら、
対応する `bun.lock` も更新し、`package.json` と一緒にコミットしてください。

## TypeScript の型定義

Rust のモデルを変更したら、`make generate-ts-types` で
`resources/generatedTypes.ts` を更新してください。`make check-local` と CI は
生成結果とファイル全体を比較し、ずれがあれば書き換えずに失敗します。
生成処理が失敗した場合も、既存のファイルは保持します。

ローカルと CI で生成結果を揃えるため、環境構築では `typeshare-cli` を
`1.13.4` に固定し、公開時のロックファイルで導入します。更新する場合は、
`Makefile` のバージョン変更と型定義の再生成を同じコミットに含めてください。

## 診断コード

`resources/error_code_message_map.ts` または `resources/warning_code_message_map.ts`
を編集し、`make generate-diagnostic-codes` を実行してください。Rust のエラーコードと
warning コードは、このメッセージ表から生成します。`make check-local` で生成結果との一致を確認します。

## ファジング

[専用のワークスペース](../../fuzz/README.md)で、実行した分岐を手掛かりに入力を生成し、
診断位置とASTの往復変換を検証できます。この検証だけ明示的にnightlyを使い、
通常の開発と決定的な回帰テストは引き続きstableで実行します。

## 性能の計測

`cargo bench --bench parser` で、短い・長いコード進行、重複 warning、warning を
収集しない従来 API、独立した構文エラー、Unicode、単一コード、AST と診断の整形を計測できます。
入力は毎回同じです。入力の妥当性を確認してから実行回数を調整し、7 回の計測結果を
1 操作あたりのマイクロ秒として、中央値・最小値・最大値の CSV で出力します。
各回は 100 ms 以上を目安にし、返された AST や診断の破棄も計測に含めます。

比較するときは Rust のバージョンとマシンを揃え、重い並行処理を止めてください。
これはネイティブ Rust API の計測であり、JavaScript/WASM 間の変換やブラウザ描画は
含みません。CI ではコンパイルを確認しますが、時間による合否判定は行いません。

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
