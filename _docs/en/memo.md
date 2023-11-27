# MEMO

- I tried to do the following for github actions releases, skipping the job if it has already been released with that tag, but the code was too long, so I decided not to do it. Even if I don't do this, the error will occur on its own, and it's not a functional problem. If the code is made into a separate action so that it is only one line, it might be more beneficial to do it.

  ```yml
      # if already released, exit
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
