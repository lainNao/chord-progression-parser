check-not-broken:
	bun i
	make generate-error-code-rs
	make check-lint
	make check-build
	make build-wasm-web
	make build-wasm-node
	make build-wasm-bundler
	make install-e2e-dependencies
	make test-rust
	make test-resources
	make test-e2e
	make test-generator

check-local:
	make check-lint
	make check-build
	make test-rust
	make test-resources

################################################################
################################################################ common 
################################################################

# install
install:
	rustup target add wasm32-unknown-unknown
	rustup component add rustfmt clippy
	cargo install typeshare-cli
	cargo install cargo-insta
	bun i
	bun lefthook install

# run
run:
	cargo run

# build
build:
	cargo build

# clean
clean:
	cargo clean

# build wasm for web (use for browser javascript without any bundler?)
build-wasm-web:
	wasm-pack build \
		--release \
		--scope lainnao \
		--out-dir ./pkg/pkg-web \
		--target web
	make generate-ts-declare-file-for-pkg-web

# build wasm for node (use for server javascript without any bundler?)
build-wasm-node:
	wasm-pack build \
		--release \
		--scope lainnao \
		--out-dir ./pkg/pkg-node \
		--target nodejs
	make generate-ts-declare-file-for-pkg-node

# build wasm for bundler (use for server/client javascript with bundler?)
build-wasm-bundler:
	wasm-pack build \
		--release \
		--scope lainnao \
		--out-dir ./pkg/pkg-bundler \
		--target bundler
	make generate-ts-declare-file-for-pkg-bundler

################################################################
################################################################ generator 
################################################################

# generate src/error_code.rs
generate-error-code-rs:
	bun resources/error_code_message_map.util.ts

# generate additional TypeScript files
# NOTE: depends on build-wasm-web
generate-ts-declare-file-for-pkg-web:
	make generate-ts-types
	npx tsc resources/error_code_message_map.ts --ignoreConfig --declaration --allowJs --module CommonJS --outDir pkg/pkg-web
	npx tsc resources/generatedTypes.ts --ignoreConfig --declaration --allowJs --module CommonJS --outDir pkg/pkg-web
	bun resources/prepare_wasm_package.ts web

# generate additional TypeScript files
# NOTE: depends on build-wasm-node
generate-ts-declare-file-for-pkg-node:
	make generate-ts-types
	npx tsc resources/error_code_message_map.ts --ignoreConfig --declaration --allowJs --module CommonJS --outDir pkg/pkg-node
	npx tsc resources/generatedTypes.ts --ignoreConfig --declaration --allowJs --module CommonJS --outDir pkg/pkg-node
	bun resources/prepare_wasm_package.ts node

# generate additional TypeScript files
# NOTE: depends on build-wasm-bundler
generate-ts-declare-file-for-pkg-bundler:
	make generate-ts-types
	npx tsc resources/error_code_message_map.ts --ignoreConfig --declaration --allowJs --module NodeNext --moduleResolution nodenext --outDir pkg/pkg-bundler
	npx tsc resources/generatedTypes.ts --ignoreConfig --declaration --allowJs --module NodeNext --moduleResolution nodenext --outDir pkg/pkg-bundler
	bun resources/prepare_wasm_package.ts bundler

# generate types
generate-ts-types:
	typeshare ./src \
		--lang=typescript \
		--output-file=resources/generatedTypes.ts
	bun resources/fix_generated_types.ts

################################################################
################################################################ fixer 
################################################################

# format
fmt:
	cargo fmt

# fix
fix:
	cargo fix

################################################################
################################################################ tester 
################################################################

# Install e2e dependencies without changing their tracked manifests or lockfiles.
install-e2e-dependencies:
	cd e2e-test/node && bun install --frozen-lockfile
	cd e2e-test/bundler && bun install --frozen-lockfile
	cd e2e-test/web && bun install --frozen-lockfile

# review snapshot
# use it when you want to update snapshot and pass snapshot test
review-snapshot:
	cargo insta review

# lint check
check-lint:
	cargo clippy --all-targets --all-features -- -D warnings
	cargo fmt --all -- --check

# build check
check-build:
	cargo check

# unit & integration test
test-rust:
	cargo test --all-targets --all-features

# e2e test
test-e2e:
	cd e2e-test/node && bun run test
	cd e2e-test/bundler && npx playwright install --with-deps && bun run test
	make run-web-e2e

# generator test
test-generator:
	cd _tools/chord-progression-generator && bun i && bun run test

# e2e test of web
run-web-e2e:
# copy pkg-web to e2e-test/web/generated-src, by overrite
	rm -rf ./e2e-test/web/generated-src && cp -r ./pkg/pkg-web ./e2e-test/web/generated-src
# copy e2e-test/web/originl.index.html to e2e-test/web/src/index.html
	cp ./e2e-test/web/original.index.html ./e2e-test/web/generated-src/index.html
# test
	cd ./e2e-test/web && npx playwright install --with-deps && bun run test

# test resources
test-resources:
	cd resources && bun test

################################################################
################################################################ util 
################################################################

# see doc
doc:
	cargo doc --open

# see coverage
see-coverage:
	cargo llvm-cov --show-missing-lines --open

# needs: "chmod +x _tools/find_files_include_multibyte_characters.sh"
find-multibyte:
	./_tools/find_files_include_multibyte_characters.sh
