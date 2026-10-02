# メモ

リリース用ワークフローは、公開処理が一部だけ失敗した場合の再実行に対応しています。

- npm と crates.io の公開では、共通の
  [公開済み判定スクリプト](../../_tools/release/is_published.sh)で対象バージョンを確認し、
  公開済みなら成功としてスキップします。
- 通信障害やレジストリのエラーでは公開ジョブを停止します。
  HTTP 404 の場合だけ未公開と判断します。
- GitHub Release は、すべてのパッケージ公開ジョブが成功した後に作成します。
  同じタグを再実行したときは、既存の GitHub Release を保持します。

再実行する場合は GitHub Actions から
[test-and-release](../../.github/workflows/test-and-release.yml)を手動実行し、
既存のタグを `tag-to-release` に指定してください。
タグは、そのタグの `Cargo.toml` に記載したバージョンと一致する必要があります。
リリースの設定は[開発ドキュメント](./how-to-develop.md#リリース)を参照してください。
