# How to develop

> currently, this development environment is for `MacOS` and `Linux` users. for `Windows` users, please use `WSL` or something.

## Make development environment

please refer to `.github/workflows/check-not-broken.yml` and `Makefile`.
and please run these commands for local CI

```bash
make install
```

## Pull Request

currently no rules.
every PR is welcome.
branch rule is not decided yet.

## Release

When the `Cargo.toml` `version` is raised and pushed to the `main` branch, it will be automatically tagged and released.
