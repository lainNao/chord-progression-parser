# How to develop

> currently, this development environment is for `MacOS` and `Linux` users. for `Windows` users, please use `WSL` or something.

## Make development environment

Install Rust through `rustup` and install Bun first. `make install` adds the
WASM target, development tools, and a missing `wasm-pack` executable.

please refer to `.github/workflows/check-not-broken.yml` and `Makefile`.
and please run these commands for local CI

```bash
make install
```

Dependency installation uses `bun install --frozen-lockfile` in setup, checks,
generator tests, and npm builds. When changing JavaScript dependencies, update
and commit the corresponding `bun.lock` together with `package.json`.

## Generated TypeScript declarations

Rust model changes require `make generate-ts-types` to update
`resources/generatedTypes.ts`. `make check-local` and CI compare the complete
file with fresh generator output and fail on stale declarations without editing
them. Generation failures leave the existing file intact.

Setup pins `typeshare-cli` to version `1.13.4` with its published lockfile so
local and CI generation agree. To upgrade it, change the version in `Makefile`
and regenerate the declarations in the same commit.

## Diagnostic codes

Edit `resources/error_code_message_map.ts` or `resources/warning_code_message_map.ts`,
then run `make generate-diagnostic-codes`. Both Rust enums are generated from these
localized message maps. `make check-local` verifies that the generated sources match.

## Fuzzing

The optional [fuzz workspace](../../fuzz/README.md) checks parser diagnostics and
AST round trips with coverage-guided input generation. It uses an explicit nightly
toolchain; normal development and deterministic regression tests remain on stable.

## Performance measurements

Run `cargo bench --bench parser` for deterministic native Rust workloads: short
and long progressions, duplicate warnings, the legacy API without warnings,
independent syntax errors, Unicode, single chords, and AST/diagnostic formatting. The benchmark
validates its fixtures, calibrates each workload, and prints CSV with the median,
minimum, and maximum microseconds per operation across seven samples. Each sample
targets at least 100 ms; the timings include dropping returned ASTs and diagnostics.

Compare revisions with the same Rust toolchain, hardware, and quiet background
workload. These are native API timings, not JavaScript/WASM conversion or browser
rendering timings. CI compiles the benchmark but does not enforce timing thresholds.

After `make build-wasm-web`, run `node _tools/benchmark-wasm.mjs` to include the
JS/WASM boundary. It reports CSV across seven samples for short and long inputs,
warnings, syntax errors, Unicode, formatting, and rejection of malformed nested
arrays. Node.js GC and JavaScript exception handling are included; browser rendering
is excluded. Pass another revision's web package directory as the argument to
compare builds. Use the same Node.js version and WASM build settings with heavy
background workloads stopped.

## Pull Request

currently no rules.
every PR is welcome.
branch rule is not decided yet.

## Commit messages

CI and hooks do not enforce a commit-message format. Releases are detected from
the `Cargo.toml` version and Git tags, and release notes include every commit
subject regardless of its prefix. Use a concise subject that describes the change.

## WASM tests

The WASM boundary is tested through Bun and Node.js ESM/CommonJS, Vite production
bundles, and browser E2E tests loading the web package directly. These exercise
the generated JavaScript, declarations, and WASM together, so there is currently
no separate `wasm-bindgen-test` suite. Clippy also checks WASM-specific Rust code.

After `make build-wasm-web`, run `make test-wasm-references` to verify that repeated
malformed ASTs reuse reference slots. This is also part of `make check-not-broken`.
Reconsider WASM-specific unit tests if Rust starts using more DOM or JavaScript APIs.

### JavaScript exceptions and runtime support

Ordinary ASTs and JSON values release their reference slots even when array fields
have invalid values. AST array fields accept JavaScript Arrays. Uint8Arrays and
ArrayBuffers are rejected, but dependency conversion temporarily copies their
contents before rejection, requiring memory proportional to their size. Repeated
inputs of the same size reuse that allocation.

Throwing getters and revoked Proxies can still skip Rust
cleanup through dependency-internal JavaScript calls. After `make build-wasm-web`,
run `node _tools/audit-wasm-references.mjs --js-exceptions` to reproduce this.
This optional diagnostic exits with code 1 when the reference table grows and is
excluded from the regular CI check.

The [`wasm-bindgen` recommendation of `panic=unwind`](https://wasm-bindgen.github.io/wasm-bindgen/reference/attributes/on-js-imports/catch.html)
released references for getter and Proxy exceptions in an isolated build. Its
[requirements](https://wasm-bindgen.github.io/wasm-bindgen/reference/catch-unwind.html#requirements)
include Rust nightly, rebuilding the standard library, and a runtime supporting
WASM exception handling. Adoption is deferred because it changes the stable build
and supported runtime requirements.

Published filenames such as `chord_progression_parser.js` stay unchanged. Renaming
them to camelCase would break existing imports and CDN URLs and requires a separate
decision about a breaking change.

## Release

When the `Cargo.toml` `version` is raised and pushed to the `main` branch, it will be automatically tagged and released.

After raising the version, run `make prepare-release` to update the README's CDN
version automatically. Commit `Cargo.toml` and the README together. CI and the
commit hook detect drift without rewriting files. Generated npm package READMEs
are also updated to their package version during the build.

The npm packages use Trusted Publishing. In each package's npm settings, configure `lainNao/chord-progression-parser` as the GitHub repository and `test-and-release.yml` as the workflow.
Under Allowed actions, also allow direct publication with `npm publish`. See the
[npm configuration guide](https://docs.npmjs.com/trusted-publishers/) for these settings.

If a release partially fails, run `test-and-release` manually in GitHub Actions with the existing tag in `tag-to-release`. Artifacts that are already published will be skipped.

Publication checks query npm or crates.io directly. Registry and network errors stop
the job; only HTTP 404 means the requested version has not been published. When
retrying an older source tag, the checks use the workflow revision's release tooling.

The GitHub Release is created after all three npm packages and the crates.io publication jobs succeed.
