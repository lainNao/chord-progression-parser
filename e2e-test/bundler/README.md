# e2e-test/bundler

Run `bun run test` after building `pkg/pkg-bundler` and installing dependencies.
Playwright type-checks and builds the example, then tests its production output
through `vite preview`. Port 3034 must be free so an existing development server
cannot substitute for the build under test. Use `bun run dev` for manual editing.
