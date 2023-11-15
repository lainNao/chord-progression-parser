# TODO

## 必須

- doc配下を静的ページとしてデプロイしたい
- CI、OSSなら無料みたいなやつあるはず。色々探してみたい
  - これとか <https://zenn.dev/binnmti/articles/7e3690ebe80951>
- `実装`
  - line!をErrorInfoにあてがうようにしたい（フィールド追加） <https://qiita.com/elipmoc101/items/f76a47385b2669ec6db3>
    - できればline, column, fileのように細かくできればより嬉しい
  - `CHORD_SHOULD_NOT_BE_EMPTY`、これいらないのでは？空にすることは可能なはず。エラーメッセージ自体がおかしいだけか？
  - `RuntimeError: unreachable`が嫌なので、どうにかしたいな。
    - これ、そもそも把握してるエラーはエラーコードで返すようにし、把握されてるエラーコードが返らなかったら不明なエラーとしてしまっていいと思う。不明なエラーはsentry等でキャッチしてあげてバグ潰しすればいい。
      - なので、とにかくエラーメッセージを返しているところをエラーコードで返すようにする。
        - で、json等でエラーコードとエラーメッセージの対応表を作る。
          - こうしたほうが、エラーメッセージもリファクタしやすい。本当は「こういう時は、こういうことはNGです」というのが真実だけど「◯◯は空ではいけません」みたいな嘘になってしまっている。
          - エラーコードが「ER-◯◯」の形式なのは検索ビリティ高いのでよいのかなとは思う
  - wasm-bindgen-testの盛り込み <https://rustwasm.github.io/docs/wasm-bindgen/wasm-bindgen-test/index.html>
  - makefile等にあるTODOの対応
  - カバレッジ見て足りてないところをリファクタしていく
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

## 仕様面

- なんかよく分かってないからか動作が重くなった気がする
  - cloneってパフォーマンス上の劣化があるっぽい
- 不正な入力へのテストをたくさん
  - エスケープとか対応するならする

## コード品質面

- `自動テスト`
  - TSでコード進行ジェネレータを作る
    - 機能テストに使うのと、パフォーマンステストにも使う（1万件のコード進行があった時の検索パフォーマンスとか）
  - コミットメッセージやソースコードに日本語を混ぜられないようにする。lefthookのcommit-msgでいけるはず
- `CI`
  - lefthookでやってるやつをgithub actionsにも入れる
  - dependabotや、そのrust版のようなもの
  - githubにプライベートなセキュリティチェック機能もあったはず
- `rust知識系`
  - serde詳しくなる <https://serde.rs/container-attrs.html>
  - PartialEqとかDebugとかのderiveの値は何なのか
  - exhaustive matchできてないところがあるかもなので網羅
  - unwrap()多すぎ問題。panic起こるので適切にハンドリングした方がjs側でエラーメッセージが親切になるにはなる。けどそこは親切にする必要無さそう。だけどエラーメッセージはJS側で制御できるのか…？
    - 多言語化のためにも、エラーメッセージはIDで管理して出し分けたほうがいいな。その番号以外のが来たら「不明なエラーが発生しました」って出すみたいな。それで解決だ。つまりハンドリングの必要は無い。（sentryとかに送るのは必要）
- `他`
  - メタ情報が重複した時のエラーが不足しているので追加
  - まだstrum使ってない箇所あるので使う（Accidentalとか）
    - 逆にserdeを使えばstrum使わなくてもいい可能性感じるので見てみる
  - コメント追加
  - 関数に切り出し
  - prettifyされない。動かなくなってるかも
  - errors、「the xx is not yy」のような動的なエラーメッセージに対応できないっぽいので修正する必要がありそう。ただしエラーIDも持ちたい。idと、stringを返す関数で作るしかない？
    - relay-compilerのようなコードが参考になるかも
      - 文字列だと自動テストでの比較も面倒になるので、エラーコードとかで管理したほうがいい…?
  - chord_detailed.rs、extension周りのコードが汚いので修正したい
  - is_token_char、Token::VARIANTSを使って書き換える

## 後に改善系

- `-5`と`b5`を同じものと扱うのはstrumの技術的に一旦諦めたので、後で対応する。今は一旦`b5`のみ対応
  - strumやめて独自で管理すれば一応は対応可能なはず
    - 例えば単に以下のようなオブジェクトの配列にするとかね
      {
        name: "FlatFive",
        aliases: ["-5", "b5"],
        intervals: [-5],
      }
