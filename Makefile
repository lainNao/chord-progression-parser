check-not-broken:
	make lint-check
	make check
	make test-rust
	make test-e2e
	make release

############### common

# run
run:
	cargo run

# run wasm in node
run-pkg-node:
	make build-wasm-node
	bun

# build
build:
	cargo build

# build wasm for web
build-wasm-web:
	wasm-pack build \
		--release \
		--out-dir ./pkg-web \
		--target web

# build wasm for node
build-wasm-node:
	wasm-pack build \
		--release \
		--out-dir ./pkg-node \
		--target nodejs

# build wasm for bundler
build-wasm-bundler:
	wasm-pack build \
		--release \
		--out-dir ./pkg-bundler \
		--target bundler

# release
release:
	cargo build --release
	wasm-pack build --release

# clean
clean:
	cargo clean

############### fixer

# format
fmt:
	cargo fmt

# fix
fix:
	cargo fix

############### tester

# lint check
lint-check:
	cargo clippy

# check
check:
	cargo check

# unit & integration test
test-rust:
	cargo test --lib

# e2e test
# TODO browserの方も作る
test-e2e:
	make build-wasm-node
	make build-wasm-browser
	cd test && bun test

############### util

# doc by comments
doc:
	cargo doc --open

# see doc
see-doc:
	rustup doc

# see coverage
see-coverage:
	cargo llvm-cov --show-missing-lines --open

generate-ts-types:
	typeshare ./src \
		--lang=typescript \
		--output-file=generatedTypes.ts

# needs: "chmod +x _tools/find_files_include_multibyte_characters.sh"
find-multibyte:
	./_tools/find_files_include_multibyte_characters.sh
