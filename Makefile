############### common

# run
run:
	cargo run

# build
build:
	cargo build

# release
release:
	cargo build --release

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

# test
test:
	cargo test


############### util

# doc by comments
doc:
	cargo doc --open

# see doc
see-doc:
	rustup doc
