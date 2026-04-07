.PHONY: man

man-src := $(wildcard man-src/*.md)

build:
	cargo build

release:
	cargo build --release

build-all:
	cargo build --workspace

release-all:
	cargo build --release --workspace

clean:
	cargo clean

test-rust:
	cargo test --workspace --verbose

fmt:
	cargo fmt

fix:
	cargo clippy --all --fix --allow-dirty --allow-staged

install:
	echo "Installing pay-respects core. Use `install-all` to install all modules."
	cargo install --path core

install-all:
	echo "Installing pay-respects core and all modules."
	cargo install --path core
	cargo install --path module-runtime-rules
	cargo install --path module-request-ai

test-suggestions: build
	cd tests && bash test.sh

test: test-rust test-suggestions

benchmark: release-all
	cd tests && bash benchmark.sh

man:
	@for i in $(man-src); do \
		output=$$(echo $$i | sed 's/\.md$$//' | sed 's/^man-src\///'); \
		pandoc -s -t man $$i -o man/$$output; \
		echo "Generated man page: $$output"; \
	done
