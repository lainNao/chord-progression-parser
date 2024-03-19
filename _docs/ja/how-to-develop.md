# 開発の仕方

> 現在、開発環境は`MacOS`か`Linux`環境のみです。`Windows`の場合は`WSL`かそれに準ずるものを使わないと動きません。

## 環境構築

`.github/workflows/check-not-broken.yml`と`Makefile`を参照してください。
あと以下のコマンドを実行してください。

```bash
make install
```

## プルリクエスト

現在はルールはありません。
どんなプルリクエストでも歓迎です。
ブランチルールはまだ決まっていません。

## リリース

`Cargo.toml`の`version`を上げて`main`ブランチにプッシュされると、自動でタグがつけられてリリースされます。
