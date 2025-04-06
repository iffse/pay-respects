build:
	cargo build

release:
	cargo build --release

build-all:
	cargo build --workspace

release-all:
	cargo build --release --workspace

test-rust:
	cargo test --workspace --verbose

install:
	echo "Installing pay-respects core. Use `install-all` to install all modules."
	cargo install --path core

install-all:
	echo "Installing pay-respects core and all modules."
	cargo install --path core
	cargo install --path module-runtime-rules
	cargo install --path module-request-ai

test-suggestions: build
	cd tests && bash main.sh

test: test-rust test-suggestions
