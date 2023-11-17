# TODO

## 必須

- `実装`
  - できればline!をErrorInfoにあてがうようにしたい（フィールド追加） <https://qiita.com/elipmoc101/items/f76a47385b2669ec6db3>
    - できればline, column, fileのように細かくできればより嬉しい
    - たぶんtokenizerの時点で各トークンに共通インターフェースとしてそれらの値を埋め込むんだと思う。となると結構な改修になるかも

- `自動テストやCI`
  - `man githooks`で色々見れるな。これをlefthookであてがう
    - 例えば`post-checkout`時に初期化処理しちゃうとか
    - <https://rfs.jp/server/git/gite-lab/git-hook-post-receive.html>
  - wasm-bindgen-testの盛り込み <https://rustwasm.github.io/docs/wasm-bindgen/wasm-bindgen-test/index.html>
  - TSでコード進行ジェネレータを作る
    - 機能テストに使うのと、パフォーマンステストにも使う（1万件のコード進行があった時の検索パフォーマンスとか）
    - というかASTジェネレータを作って、それにtoString()を生やせばいいと思う
  - コミットメッセージやソースコードに日本語を混ぜられないようにする。lefthookのcommit-msgでいけるはず
    - 全角スペースはソースコードとしてありうるようにしてるので許したい
  - lefthookでやってるやつをgithub actionsにも入れる
  - dependabotや、そのrust版のようなもの
  - githubにプライベートなセキュリティチェック機能もあったはず
  - CI、OSSなら無料みたいなやつあるはず。色々探してみたい
    - これとか <https://zenn.dev/binnmti/articles/7e3690ebe80951>

- `リリース設計`（ここらへんは別リポジトリでテストしてから持ってくる形で…色々試すの汚いので）
  - 1コマンド打てば、以下が終わるようにしたい。それをgithub actionsでもローカルでもどちらでも行えるように
    - 1. cloudflare
      - ASTをパースする静的ページ一つを作ってそれを使えるようにする
    - 2. jsDeliver
      - browser版のビルドのみ
      - 参考 <https://zenn.dev/nino_cast/articles/98a0a87f58026f#cdn%E5%8B%95%E4%BD%9C%E3%81%AE%E3%83%AD%E3%83%BC%E3%82%AB%E3%83%AB%E3%83%81%E3%82%A7%E3%83%83%E3%82%AF>
    - 2. npm
      - browser、node、bundlerいずれも
        - これやるために出力したpkgのpackage.jsonの内容にリネームかけないといけない
      - npmコマンドとかで
      - 参考 <https://docs.npmjs.com/creating-and-publishing-private-packages> <https://rustwasm.github.io/docs/wasm-pack/tutorials/npm-browser-packages/packaging-and-publishing.html>
    - 3. github
      - ghコマンドで
    - 4. crates.io
      - crates.ioはrustのやつのみ。
    - 全体的な指針
      - リリースノートは自動生成（そんなに重要じゃないので、これまでのコミットを全部リストアップでいい）
        - リリースノート自動生成はgithub自体が機能として持っているっぽい　<https://docs.github.com/en/repositories/releasing-projects-on-github/automatically-generated-release-notes>
      - CHANGELOGも自動生成したい。メジャーバージョンごとにCHANGELOG.v{n}.mdを作る感じで
      - 良いツールなければ自前。semantic-releaseは使いづらかった。
      - タグつけたものをプッシュしたら、勝手にCIが「まだリリースされてないタグが見つかったらリリースしておく」というのをやってくれるような感じにしたら楽そう。あまり意識したくないので。
- `ドキュメント`
  - [ ] README.md
  - [x] DEVELOPMENT.md
  - [x] _docs配下

## 後でOK

- `コードクオリティ面（むしろチマチマやる方がメンテ感が出るのもある）`
  - カバレッジ見て足りてないところをリファクタしていく
  - パフォーマンスチューニング（cloneを減らすとか）
  - 全体的にリファクタ等
    - chord_detailed.rs、extension周りのコードが汚いので修正したい
    - serde詳しくなる <https://serde.rs/container-attrs.html>
    - exhaustive matchできてないところがあるかもなので網羅
  - メタ情報が重複した時のエラーが不足しているので追加？
  - まだstrum使ってない箇所あるので使う（Accidentalとか）
    - 逆にserdeを使えばstrum使わなくてもいい可能性感じるので見てみる
    - is_token_char、Token::VARIANTSを使って書き換える
  - コメント追加
  - 関数に切り出し
  - prettifyされない。動かなくなってるかも

- `仕様面`
  - エラーメッセージ、現状は1個しか返せないが、パーサーなら複数返してほしいな。そういう懸念とか集めてからV1の公開にしたいな
  - エスケープとか対応するならする
  - `-5`と`b5`を同じものと扱うのはstrumの技術的に一旦諦めたので、後で対応する。今は一旦`b5`のみ対応
    - strumやめて独自で管理すれば一応は対応可能なはず
      - 例えば単に以下のようなオブジェクトの配列にするとかね
        {
          name: "FlatFive",
          aliases: ["-5", "b5"],
          intervals: [-5],
        }

- `他周辺`
  - doc配下を静的ページとしてデプロイする必要あるならする。GitBookみたいな何かで
  