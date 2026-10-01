# How to develop

> currently, this development environment is for `MacOS` and `Linux` users. for `Windows` users, please use `WSL` or something.

## Make development environment

please refer to `.github/workflows/check-not-broken.yml` and `Makefile`.
and please run these commands for local CI

```bash
make install
```

## Diagnostic codes

Edit `resources/error_code_message_map.ts` or `resources/warning_code_message_map.ts`,
then run `make generate-diagnostic-codes`. Both Rust enums are generated from these
localized message maps. `make check-local` verifies that the generated sources match.

## Fuzzing

The optional [fuzz workspace](../../fuzz/README.md) checks parser diagnostics and
AST round trips with coverage-guided input generation. It uses an explicit nightly
toolchain; normal development and deterministic regression tests remain on stable.

## Pull Request

currently no rules.
every PR is welcome.
branch rule is not decided yet.

## Release

When the `Cargo.toml` `version` is raised and pushed to the `main` branch, it will be automatically tagged and released.

The npm packages use Trusted Publishing. In each package's npm settings, configure `lainNao/chord-progression-parser` as the GitHub repository and `test-and-release.yml` as the workflow.

If a release partially fails, run `test-and-release` manually in GitHub Actions with the existing tag in `tag-to-release`. Artifacts that are already published will be skipped.

Publication checks query npm or crates.io directly. Registry and network errors stop
the job; only HTTP 404 means the requested version has not been published. When
retrying an older source tag, the checks use the workflow revision's release tooling.

The GitHub Release is created after all three npm packages and the crates.io publication jobs succeed.
