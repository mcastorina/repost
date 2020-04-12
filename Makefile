RM := rm -f

.DEFAULT_GOAL := build

.PHONY: check
check:
	cargo check

build: target/debug/repost

target/debug/repost: Cargo.toml src/main.rs src/lib.rs
	cargo build

fmt: src/main.rs src/lib.rs
	cargo fmt

.PHONY: test
test:
	cargo test

.PHONY: clean
clean:
	$(RM) target/debug/repost
