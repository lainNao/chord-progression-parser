check-not-broken:
	make generate-error-code-rs
	make lint-check
	make build-check
	make format-check
	make build-wasm-web
	make build-wasm-node
	make build-wasm-bundler
	make test-rust
	make test-e2e
# TODO: pkg配下のがts的にエラー起きてないかどうかも見る。それもe2e落ちてると考える
# TODO: ts等のツールも型チェック

################################################################
################################################################ common 
################################################################

# run
run:
	cargo run

# build
build:
	cargo build

# clean
clean:
	cargo clean

# release
# TODO: 
release:
	cargo build --release

# build wasm for web (use for browser javascript without any bundler?)
build-wasm-web:
	wasm-pack build \
		--release \
		--scope lainNao \
		--out-dir ./pkg/pkg-web \
		--target web
	make generate-ts-declare-file-for-pkg-web

# build wasm for node (use for server javascript without any bundler?)
build-wasm-node:
	wasm-pack build \
		--release \
		--scope lainNao \
		--out-dir ./pkg/pkg-node \
		--target nodejs
	make generate-ts-declare-file-for-pkg-node

# build wasm for bundler (use for server/client javascript with bundler?)
build-wasm-bundler:
	wasm-pack build \
		--release \
		--scope lainNao \
		--out-dir ./pkg/pkg-bundler \
		--target bundler
	make generate-ts-declare-file-for-pkg-bundler

################################################################
################################################################ generator 
################################################################

# generate src/error_code.rs
generate-error-code-rs:
	bun resources/error_code_message_map.util.ts

# generate and modify d.ts
# FIXME: this is not good way. do it by wasm_bindgen directly
generate-ts-declare-file-for-pkg-web:
	make generate-ts-types
# append contents of generatedTypes.ts to pkg-web/chord_progression_ast_parser.d.ts
	cat generatedTypes.ts >> pkg/pkg-web/chord_progression_ast_parser.d.ts
# Rewrite definition of return value of run function in pkg-web/chord_progression_ast_parser.d.ts to "Ast"
	sed -i.bak 's/any/Ast/g' pkg/pkg-web/chord_progression_ast_parser.d.ts && rm pkg/pkg-web/chord_progression_ast_parser.d.ts.bak
# copy resources/error_code_message_map.ts under pkg/pkg-web, overwriting
	cp resources/error_code_message_map.ts pkg/pkg-web
# compile it to .js and d.ts
	cd pkg/pkg-web && npx tsc error_code_message_map.ts --declaration --allowJs --module ES6
# add error_code_message_map.js and d.ts to pkg/pkg-web/package.json files
	sed -i.bak 's/"files": \[/"files": \[\
		"error_code_message_map.js", "error_code_message_map.ts", "error_code_message_map.d.ts",/g' pkg/pkg-web/package.json && rm pkg/pkg-web/package.json.bak

# generate and modify d.ts
# FIXME: this is not good way. do it by wasm_bindgen directly
generate-ts-declare-file-for-pkg-node:
	make generate-ts-types
# append contents of generatedTypes.ts to pkg-node/chord_progression_ast_parser.d.ts
	cat generatedTypes.ts >> pkg/pkg-node/chord_progression_ast_parser.d.ts
# Rewrite definition of return value of run function in pkg-node/chord_progression_ast_parser.d.ts to "Ast"
	sed -i.bak 's/any/Ast/g' pkg/pkg-node/chord_progression_ast_parser.d.ts && rm pkg/pkg-node/chord_progression_ast_parser.d.ts.bak
# copy resources/error_code_message_map.ts under pkg/pkg-node, overwriting
	cp resources/error_code_message_map.ts pkg/pkg-node
# compile it to .js and d.ts
	cd pkg/pkg-node && npx tsc error_code_message_map.ts --declaration --allowJs --module ES6
# add error_code_message_map.js and d.ts to pkg/pkg-node/package.json files
	sed -i.bak 's/"files": \[/"files": \[\
		"error_code_message_map.js", "error_code_message_map.ts", "error_code_message_map.d.ts",/g' pkg/pkg-node/package.json && rm pkg/pkg-node/package.json.bak

# generate and modify d.ts
# FIXME: this is not good way. do it by wasm_bindgen directly
generate-ts-declare-file-for-pkg-bundler:
	make generate-ts-types
# append contents of generatedTypes.ts to pkg-bundler/chord_progression_ast_parser.d.ts
	cat generatedTypes.ts >> pkg/pkg-bundler/chord_progression_ast_parser.d.ts
# Rewrite definition of return value of run function in pkg-bundler/chord_progression_ast_parser.d.ts to "Ast"
	sed -i.bak 's/any/Ast/g' pkg/pkg-bundler/chord_progression_ast_parser.d.ts && rm pkg/pkg-bundler/chord_progression_ast_parser.d.ts.bak
# copy resources/error_code_message_map.ts under pkg/pkg-bundler, overwriting
	cp resources/error_code_message_map.ts pkg/pkg-bundler
# compile it to .js and d.ts
	cd pkg/pkg-bundler && npx tsc error_code_message_map.ts --declaration --allowJs --module ES6
# add error_code_message_map.j and d.tss to pkg/pkg-bundler/package.json files
	sed -i.bak 's/"files": \[/"files": \[\
		"error_code_message_map.js", "error_code_message_map.ts","error_code_message_map.d.ts", /g' pkg/pkg-bundler/package.json && rm pkg/pkg-bundler/package.json.bak

generate-ts-types:
	typeshare ./src \
		--lang=typescript \
		--output-file=generatedTypes.ts

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

# lint check
lint-check:
	cargo clippy

# build check
build-check:
	cargo check

# format check
format-check:
	cargo fmt --all -- --check

# unit & integration test
test-rust:
	cargo test --lib

# e2e test
# TODO: clientの方も作る
test-e2e:
	make build-wasm-bundler
	cd e2e-test/node && bun i --frozen-lockfile && bun test
	cd e2e-test/bundler && bun i --frozen-lockfile && bun test
	make run-web-e2e

run-web-e2e:
# copy pkg-web to e2e-test/web/generated-src, by overrite
	rm -rf ./e2e-test/web/generated-src && cp -r ./pkg/pkg-web ./e2e-test/web/generated-src
# copy e2e-test/web/originl.index.html to e2e-test/web/src/index.html
	cp ./e2e-test/web/original.index.html ./e2e-test/web/generated-src/index.html
# NOTE: 一旦やらないが、 cd e2e-test/web/generated-src && npx http-server .　でテストしたい。というかplaywrightか何かでe2eテストをする
	echo "TODO: test"

################################################################
################################################################ util 
################################################################

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
# TODO: DEVELOMENT.mdに必要かどうか見てみる。CIにはどうかな？
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
# TODO: 
release-github:
	gh release create v$$(awk -F' = ' '/^version/ {print $$2}' Cargo.toml | tr -d '"') \
		--title "v$$(awk -F' = ' '/^version/ {print $$2}' Cargo.toml | tr -d '"')" \
		--notes "v$$(awk -F' = ' '/^version/ {print $$2}' Cargo.toml | tr -d '"')"
