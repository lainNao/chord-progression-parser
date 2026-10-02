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

JS/WASM の変換を含む性能は、`make build-wasm-web` の後に
`node _tools/benchmark-wasm.mjs` で計測します。短い・長い入力、warning、構文エラー、
Unicode、整形、不正なネスト配列の拒否を 7 回ずつ計測し、CSV で出力します。
Node.js の GC と JavaScript の例外処理も計測に含み、ブラウザ描画は含みません。
別のリビジョンの web package directory を引数に指定して比較できます。
重い並行処理を止め、Node.js と WASM のビルド条件を揃えてください。

## プルリクエスト

現在はルールはありません。
どんなプルリクエストでも歓迎です。
ブランチルールはまだ決まっていません。

## コミットメッセージ

メッセージの形式は CI や hook では強制しません。リリースの判定は
`Cargo.toml` の version とタグを使い、リリースノートには種別にかかわらず
すべてのコミットの件名を載せます。変更内容が分かる簡潔な件名を付けてください。

## WASM のテスト

WASM 境界は、Bun と Node.js の ESM/CommonJS、Vite の production bundle、
web package を直接読み込むブラウザの E2E で検証します。生成された JavaScript、
型定義、WASM の組み合わせを確認するため、現状は `wasm-bindgen-test` による
別の単体テスト群は追加しません。Rust 側の WASM 固有コードには Clippy をかけます。

不正な AST を繰り返し渡したときの参照枠の再利用は、`make build-wasm-web` の後に
`make test-wasm-references` で確認できます。この検証は `make check-not-broken` にも
含まれます。Rust から DOM や JavaScript API を操作する処理が増えた場合は、
WASM 専用の単体テストの必要性を再検討します。

### JavaScript の例外と対応環境

通常の AST と JSON の値では、配列フィールドに不正な値を渡しても参照枠を回収します。
AST の配列フィールドは JavaScript の Array を受け取ります。Uint8Array や ArrayBuffer は
拒否しますが、依存の変換処理が拒否前に内容を一時コピーするため、大きな値では
その分のメモリを使います。同じサイズを繰り返してもコピー領域は再利用されます。
ただし、例外を投げる getter や revoked Proxy を AST に含めると、依存内の JavaScript
呼び出しが Rust の解放処理を飛ばし、参照が残ります。`make build-wasm-web` の後に
`node _tools/audit-wasm-references.mjs --js-exceptions` で再現できます。この任意の診断は
参照テーブルの増加を検出して終了コード 1 を返し、通常の CI には含めません。

[`wasm-bindgen` の推奨する `panic=unwind`](https://wasm-bindgen.github.io/wasm-bindgen/reference/attributes/on-js-imports/catch.html)
で、getter・Proxy の例外でも解放されることを隔離したビルドで確認しました。
ただし、[要件](https://wasm-bindgen.github.io/wasm-bindgen/reference/catch-unwind.html#requirements)は
Rust nightly、標準ライブラリの再ビルド、WASM exception handling 対応ランタイムです。
現在の stable ビルドと対応環境を変えるため、自動では採用せず判断を保留します。

公開されている `chord_progression_parser.js` などのファイル名は維持します。
camelCase へ変えると、既存の import と CDN の URL が壊れるため、変更する場合は
破壊的変更として別途判断します。

## リリース

`Cargo.toml`の`version`を上げて`main`ブランチにプッシュされると、自動でタグがつけられてリリースされます。

version を上げた後は `make prepare-release` を実行し、README の CDN の
バージョンを自動更新してください。`Cargo.toml` と README を同じコミットへ含めます。
CI と commit hook は不一致を検出しますが、ファイルを書き換えません。
生成する npm パッケージの README も、ビルド時にその package version へ合わせます。

npmパッケージはTrusted Publishingを使います。各パッケージのnpm設定では、GitHubリポジトリを`lainNao/chord-progression-parser`、workflowを`test-and-release.yml`として登録してください。
Allowed actions では、`npm publish` による直接公開も許可してください。
設定項目は [npm の説明](https://docs.npmjs.com/trusted-publishers/)を参照してください。

リリースが一部だけ失敗した場合は、GitHub Actionsから`test-and-release`を開き、既存タグを`tag-to-release`に指定して再実行します。公開済みの成果物はスキップされます。

公開済みの判定は npm または crates.io に直接問い合わせます。通信障害やレジストリの
エラーではジョブを停止し、HTTP 404 の場合だけ未公開と判断します。過去のタグを再実行
するときも、公開済み判定にはワークフロー側のリリース用ツールを使います。

GitHub Release は npm の3パッケージと crates.io の公開ジョブがすべて成功した後に作成されます。
