# メモ

- github actionsのリリース類、すでにそのタグでリリースされていたらそのジョブをスキップする等したくて以下のように考えてみたが、処理が長すぎるので一旦やめた。crates.ioの方もやるとさらに面倒だし。別にこれをしなくても勝手にエラーになってくれたりするし機能的には問題ないので。処理が1行で済むように別action化されてたらやったほうがメリット大きくなってくるかも。

  ```yml
      # すでにリリースされてるなら落とす
      - name: Check the version does not released
        run: |
          AVAILABLE_VERSIONS=$(npm view @lainnao/chord-progression-parser-bundler versions --json)
          TARGET_VERSION=$(echo ${{ inputs.tag-to-release }} | sed -e 's/^v//')
          if [[ $(echo $AVAILABLE_VERSIONS | jq 'index("'$TARGET_VERSION'")') == null ]]; then
            echo "Version $TARGET_VERSION does not exist"
          else
            echo "Version $TARGET_VERSION already exists"
            exit 1
          fi
  ```
