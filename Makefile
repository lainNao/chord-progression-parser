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

# build wasm for browser
build-wasm-browser:
	wasm-pack build \
		--release \
		--out-dir ./pkg-browser

# build wasm for node
build-wasm-node:
	wasm-pack build \
		--release \
		--out-dir ./pkg-node \
		--target nodejs

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

# integration test
test-integration:
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
