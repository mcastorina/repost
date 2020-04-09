RM := rm -f

.DEFAULT_GOAL := build

.PHONY: check
check:
	cargo check

build: target/debug/repost

target/debug/repost: Cargo.toml src/main.rs
	cargo build

fmt: src/main.rs
	cargo fmt

.PHONY: clean
clean:
	$(RM) target/debug/repost
