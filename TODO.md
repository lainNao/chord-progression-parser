# TODO

## 必須

- `doc`
  - readme
    - TBDのところ埋める
    - versionをtipみたいなUIで書くやつやりたい

- `メタ実装`
  - set_panic_hook、イメージ通りに使えてないのでは？これを使えばcatchに行かないようにできるかも？
    - 本当はpanic時にスタックトレースを出したいので、出せるようにしてほしいなと思う

- `リリース設計`（ここらへんは別リポジトリでテストしてから持ってくる形で…色々試すの汚いので）
  - 1コマンド打てば、以下が終わるようにしたい。それをgithub actionsでもローカルでもどちらでも行えるように
    - 1. cloudflare
      - ASTをパースする静的ページ一つを作ってそれを使えるようにする
    - 2. jsDeliver
      - browser版のビルドのみ
      - 参考 <https://zenn.dev/nino_cast/articles/98a0a87f58026f#cdn%E5%8B%95%E4%BD%9C%E3%81%AE%E3%83%AD%E3%83%BC%E3%82%AB%E3%83%AB%E3%83%81%E3%82%A7%E3%83%83%E3%82%AF>
    - 3. npm
      - browser、node、bundlerいずれも
        - これやるために出力したpkgのpackage.jsonの内容にリネームかけないといけない
        - <https://docs.github.com/en/actions/publishing-packages/publishing-nodejs-packages>
      - npmコマンドとかで
      - 参考 <https://docs.npmjs.com/creating-and-publishing-private-packages> <https://rustwasm.github.io/docs/wasm-pack/tutorials/npm-browser-packages/packaging-and-publishing.html>
    - 4. github
      - ghコマンドで
    - 全体的な指針
      - リリースノートは自動生成（そんなに重要じゃないので、これまでのコミットを全部リストアップでいい）
        - リリースノート自動生成はgithub自体が機能として持っているっぽい　<https://docs.github.com/en/repositories/releasing-projects-on-github/automatically-generated-release-notes>
      - CHANGELOGも自動生成したい。メジャーバージョンごとにCHANGELOG.v{n}.mdを作る感じで
      - 良いツールなければ自前。semantic-releaseは使いづらかった。
      - タグつけたものをプッシュしたら、勝手にCIが「まだリリースされてないタグが見つかったらリリースしておく」というのをやってくれるような感じにしたら楽そう。あまり意識したくないので。

- `自動テストやCI`
  - TSでコード進行ジェネレータを作る。実装的にはASTジェネレータを作って、それにtoString()を生やせばいいと思う
    - 機能テストに使うのと、パフォーマンステストにも使えたらよいなと思う（1万件のコード進行があった時の検索パフォーマンスとか）

## 後でOK

- `保守面`
  - 新バージョンをリリースしたら勝手にREADME内に書かれてるバージョンも上げたい。github actions使ってsedでいじってREADMEをコミットしちゃっていい気がする

- `コードクオリティ面（むしろチマチマやる方がメンテ感が出るのもあるので、あとの方がよいまである）`
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
  - lefthookの`commit-msg`フックでコミットメッセージに日本語入ってたら落としたい。なぜかうまくいかず一旦諦めた。commitlintとか使えばいいのかも
  - wasm-bindgen-testの盛り込み <https://rustwasm.github.io/docs/wasm-bindgen/wasm-bindgen-test/index.html>
    - 不要ならば無しでいいけども調べてほしい

- `仕様面`
  - エラーメッセージ、現状は1個しか返せないが、パーサーなら複数返してほしいな。そういう懸念とか集めてからV1の公開にしたいな
    - これやる方法、エラー出たmatchはエラーをスタックに詰んだ時点でbreakして、一旦なかったことにして次のループに進めば良い気がする
  - エラー時、できればスタックトレースがほしい。
    - line!, file!, column!とか使えばソース上の位置情報が取れるっぽい。というかスタックトレース取れないものか？
    - エラー情報、GPTに聞いてみたら以下もありらしい

      ```txt
      message　・・・エラーの具体的な説明。
      errorType　・・・エラーの種類やコード（例：「SyntaxError」、「TypeError」など）。
      context　・・・エラーが発生したコードの周辺部分。特定のエラーをより明確に理解するのに役立ちます。
      severity　・・・エラーの重大度。例えば、「警告」、「エラー」、「致命的エラー」など。
      suggestions　・・・エラーの解決策やヒント。
      ```

  - `-5`と`b5`を同じものと扱うのはstrumの技術的に一旦諦めたので、後で対応する。今は一旦`b5`のみ対応
    - strumやめて独自で管理すれば一応は対応可能なはず
      - 例えば単に以下のようなオブジェクトの配列にするとかね
        {
          name: "FlatFive",
          aliases: ["-5", "b5"],
          intervals: [-5],
        }
  - エスケープとか対応するならする
  - というかエラーのみならず正常パースしたTokenにもPosition周りの情報を付けたほうが逆パース（？）をしやすいのでは。その場合ASTを大改修になるのでやめておくか…
    - いやもしかしたらだけど、ASTツリー自体にフィールドを追加するのでなく単にtokenizerの結果も返すだけでいいのかも。以下イメージ

      ```json
        {
          "ast": AST,
          "tokensWithPosition": [
            {
              "token": Token,
              "position": Position
            }
          ]
        }
      ```
