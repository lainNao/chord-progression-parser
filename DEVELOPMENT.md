# How to develop

> NOTE: currently, this development environment is for `MacOS` and `Linux` users. for `Windows` users, please use `WSL` or `Cygwin`

## First Instructions

1. first, please build the `Rust` and `Bun` environment
2. next, execute the following commands

    ```bash
    rustup component add rustfmt clippy
    bun lefthook install
    ```

3. finally, please execute the following commands

    ```bash
    make check-not-broken
    ```

4. if the commands are executed successfully, the development environment is ready
    - if not, please check the error messages and fix them
5. done

## Pull Request rules

currently no rules. every PR is welcome.
branch rule is not decided yet.
