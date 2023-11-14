default: fmt check lint
check:
	cargo check
	cargo test

fmt:
	cargo fmt

lint:
	cargo clippy --fix --allow-dirty

install:
	cargo install --path . --locked
