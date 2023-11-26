check-not-broken:
	bun i
	make generate-error-code-rs
	make check-lint
	make check-build
	make build-wasm-web
	make build-wasm-node
	make build-wasm-bundler
	make modify-package-name-web
	make modify-package-name-node
	make modify-package-name-bundler
	make test-rust
	make test-resources
	make test-e2e

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

# modify package-name
# append "-web" to package json "name" field
modify-package-name-web:
	cd pkg/pkg-web && npx change-package-name @lainnao\/chord-progression-parser-web && bun i

# build wasm for node (use for server javascript without any bundler?)
build-wasm-node:
	wasm-pack build \
		--release \
		--scope lainnao \
		--out-dir ./pkg/pkg-node \
		--target nodejs
	make generate-ts-declare-file-for-pkg-node

# modify package-name
# append "-node" to package json "name" field
modify-package-name-node:
	cd pkg/pkg-node && npx change-package-name @lainnao\/chord-progression-parser-node && bun i

# build wasm for bundler (use for server/client javascript with bundler?)
build-wasm-bundler:
	wasm-pack build \
		--release \
		--scope lainnao \
		--out-dir ./pkg/pkg-bundler \
		--target bundler
	make generate-ts-declare-file-for-pkg-bundler

# modify package-name
# append "-bundler" to package json "name" field
modify-package-name-bundler:
	cd pkg/pkg-bundler && npx change-package-name @lainnao\/chord-progression-parser-bundler && bun i

################################################################
################################################################ generator 
################################################################

# generate src/error_code.rs
generate-error-code-rs:
	bun resources/error_code_message_map.util.ts

# generate and modify d.ts
# HACK: this is not good way. do it by wasm_bindgen directly
generate-ts-declare-file-for-pkg-web:
	make generate-ts-types
# append contents of generatedTypes.ts to pkg-web/chord_progression_parser.d.ts
	cat generatedTypes.ts >> pkg/pkg-web/chord_progression_parser.d.ts
# prepend contents of additionalType.ts.txt to pkg/pkg-web/chord_progression_parser.d.ts
	cat resources/additionalType.ts.txt pkg/pkg-web/chord_progression_parser.d.ts > pkg/pkg-web/chord_progression_parser.d.ts.tmp && mv pkg/pkg-web/chord_progression_parser.d.ts.tmp pkg/pkg-web/chord_progression_parser.d.ts
# Rewrite definition of return value of run function in pkg-web/chord_progression_parser.d.ts to "Ast"
	sed -i.bak 's/any/ParsedResult/g' pkg/pkg-web/chord_progression_parser.d.ts && rm pkg/pkg-web/chord_progression_parser.d.ts.bak
# copy resources/error_code_message_map.ts under pkg/pkg-web, overwriting
	cp resources/error_code_message_map.ts pkg/pkg-web
# compile it to .js and d.ts
	cd pkg/pkg-web && bun i -D typescript && npx tsc error_code_message_map.ts --declaration --allowJs --module ES6
# add error_code_message_map.js and d.ts to pkg/pkg-web/package.json files
	sed -i.bak 's/"files": \[/"files": \[\
		"error_code_message_map.js", "error_code_message_map.ts", "error_code_message_map.d.ts",/g' pkg/pkg-web/package.json && rm pkg/pkg-web/package.json.bak

# generate and modify d.ts
# HACK: this is not good way. do it by wasm_bindgen directly
generate-ts-declare-file-for-pkg-node:
	make generate-ts-types
# append contents of generatedTypes.ts to pkg-node/chord_progression_parser.d.ts
	cat generatedTypes.ts >> pkg/pkg-node/chord_progression_parser.d.ts
# prepend contents of additionalType.ts.txt to pkg/pkg-node/chord_progression_parser.d.ts
	cat resources/additionalType.ts.txt pkg/pkg-node/chord_progression_parser.d.ts > pkg/pkg-node/chord_progression_parser.d.ts.tmp && mv pkg/pkg-node/chord_progression_parser.d.ts.tmp pkg/pkg-node/chord_progression_parser.d.ts
# Rewrite definition of return value of run function in pkg-node/chord_progression_parser.d.ts to "Ast"
	sed -i.bak 's/any/ParsedResult/g' pkg/pkg-node/chord_progression_parser.d.ts && rm pkg/pkg-node/chord_progression_parser.d.ts.bak
# copy resources/error_code_message_map.ts under pkg/pkg-node, overwriting
	cp resources/error_code_message_map.ts pkg/pkg-node
# compile it to .js and d.ts
	cd pkg/pkg-node && bun i -D typescript && npx tsc error_code_message_map.ts --declaration --allowJs --module ES6
# add error_code_message_map.js and d.ts to pkg/pkg-node/package.json files
	sed -i.bak 's/"files": \[/"files": \[\
		"error_code_message_map.js", "error_code_message_map.ts", "error_code_message_map.d.ts",/g' pkg/pkg-node/package.json && rm pkg/pkg-node/package.json.bak

# generate and modify d.ts
# HACK: this is not good way. do it by wasm_bindgen directly
generate-ts-declare-file-for-pkg-bundler:
	make generate-ts-types
# append contents of generatedTypes.ts to pkg-bundler/chord_progression_parser.d.ts
	cat generatedTypes.ts >> pkg/pkg-bundler/chord_progression_parser.d.ts
# prepend contents of additionalType.ts.txt to pkg/pkg-bundler/chord_progression_parser.d.ts
	cat resources/additionalType.ts.txt pkg/pkg-bundler/chord_progression_parser.d.ts > pkg/pkg-bundler/chord_progression_parser.d.ts.tmp && mv pkg/pkg-bundler/chord_progression_parser.d.ts.tmp pkg/pkg-bundler/chord_progression_parser.d.ts
# Rewrite definition of return value of run function in pkg-bundler/chord_progression_parser.d.ts to "Ast"
	sed -i.bak 's/any/ParsedResult/g' pkg/pkg-bundler/chord_progression_parser.d.ts && rm pkg/pkg-bundler/chord_progression_parser.d.ts.bak
# copy resources/error_code_message_map.ts under pkg/pkg-bundler, overwriting
	cp resources/error_code_message_map.ts pkg/pkg-bundler
# compile it to .js and d.ts
	cd pkg/pkg-bundler && bun i -D typescript && npx tsc error_code_message_map.ts --declaration --allowJs --module ES6
# add error_code_message_map.j and d.tss to pkg/pkg-bundler/package.json files
	sed -i.bak 's/"files": \[/"files": \[\
		"error_code_message_map.js", "error_code_message_map.ts","error_code_message_map.d.ts", /g' pkg/pkg-bundler/package.json && rm pkg/pkg-bundler/package.json.bak

# generate types
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

# review snapshot
# use it when you want to update snapshot and pass snapshot test
review-snapshot:
	cargo insta review

# lint check
check-lint:
	cargo clippy
	cargo fmt --all -- --check

# build check
check-build:
	cargo check

# unit & integration test
test-rust:
	cargo test --lib

# e2e test
test-e2e:
	cd e2e-test/node && bun i -D typescript && bun run test
	cd e2e-test/bundler && bun i -D typescript && npx playwright install --with-deps && bun run test
	make run-web-e2e

# e2e test of web
run-web-e2e:
# copy pkg-web to e2e-test/web/generated-src, by overrite
	rm -rf ./e2e-test/web/generated-src && cp -r ./pkg/pkg-web ./e2e-test/web/generated-src
# copy e2e-test/web/originl.index.html to e2e-test/web/src/index.html
	cp ./e2e-test/web/original.index.html ./e2e-test/web/generated-src/index.html
# test
	cd ./e2e-test/web && bun i -D typescript && npx playwright install --with-deps && bun run test

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
