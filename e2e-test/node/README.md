# e2e-test/node

`bun run test` runs the Bun tests, native Node.js ESM and CommonJS tests, and the TypeScript check.
Run `bun run test:node` to check the native Node.js imports and WASM API separately.

Build the current parser with `make build-wasm-node` at the repository root first.
The shorthand tests share literal AST expectations with Rust through
`tests/fixtures/shorthand.json` and separately check warning serialization and
original UTF-16 source positions.
