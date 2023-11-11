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

# build
build:
	cargo build

# build wasm for web 
# use for browser javascript without any bundler?
build-wasm-web:
	wasm-pack build \
		--release \
		--out-dir ./pkg-web \
		--target web

# build wasm for node
# use for server javascript without any bundler?
build-wasm-node:
	wasm-pack build \
		--release \
		--out-dir ./pkg-node \
		--target nodejs
	make generate-ts-declare-file-for-pkg-node

# build wasm for bundler
# use for server/client javascript with bundler?
build-wasm-bundler:
	wasm-pack build \
		--release \
		--out-dir ./pkg-bundler \
		--target bundler
	make generate-ts-declare-file-for-pkg-bundler

# generate and modify d.ts
# FIXME: this is not good way. do it by wasm_bindgen directly
generate-ts-declare-file-for-pkg-node:
	make generate-ts-types
# pkg-node/chord_progression_ast_parser.d.tsに、generatedTypes.tsのコンテンツを追記する	
	cat generatedTypes.ts >> pkg-node/chord_progression_ast_parser.d.ts
# pkg-node/chord_progression_ast_parser.d.tsのrun関数の戻り値の定義を「Ast」に書き換える
	sed -i.bak 's/any/Ast/g' pkg-node/chord_progression_ast_parser.d.ts && rm pkg-node/chord_progression_ast_parser.d.ts.bak

# generate and modify d.ts
# FIXME: this is not good way. do it by wasm_bindgen directly
generate-ts-declare-file-for-pkg-bundler:
	make generate-ts-types
# pkg-bundler/chord_progression_ast_parser.d.tsに、generatedTypes.tsのコンテンツを追記する	
	cat generatedTypes.ts >> pkg-bundler/chord_progression_ast_parser.d.ts
# pkg-bundler/chord_progression_ast_parser.d.tsのrun関数の戻り値の定義を「Ast」に書き換える
	sed -i.bak 's/any/Ast/g' pkg-bundler/chord_progression_ast_parser.d.ts && rm pkg-bundler/chord_progression_ast_parser.d.ts.bak

generate-ts-types:
	typeshare ./src \
		--lang=typescript \
		--output-file=generatedTypes.ts

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
# TODO clientの方も作る
test-e2e:
	make build-wasm-bundler
	cd e2e-test/server && bun test
	cd e2e-test/client && bun test

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

# needs: "chmod +x _tools/find_files_include_multibyte_characters.sh"
find-multibyte:
	./_tools/find_files_include_multibyte_characters.sh

# make git tag
tag:
	VERSION=$$(awk -F' = ' '/^version/ {print $$2}' Cargo.toml | tr -d '"'); \
	echo "Creating tag v$$VERSION"; \
	git tag -a v$$VERSION -m "tag v$$VERSION" && git tag -n1 | head -n1;
	
# push git tag
push-tag:
	git push origin --tags

# release in github
# TODO ビルド成果物をexportした状態でリリースしないと。むしろリリースするのはrustでなくpkg-*配下のみでいいと思う。ただ複数のwasmがあるからそれはどうしよう
#      https://developer.mozilla.org/ja/docs/WebAssembly/Rust_to_Wasm
release-github:
	gh release create v$$(awk -F' = ' '/^version/ {print $$2}' Cargo.toml | tr -d '"') \
		--title "v$$(awk -F' = ' '/^version/ {print $$2}' Cargo.toml | tr -d '"')" \
		--notes "v$$(awk -F' = ' '/^version/ {print $$2}' Cargo.toml | tr -d '"')"