RM := rm -f

RS_FILES := $(shell find src -type f)
.DEFAULT_GOAL := build

.PHONY: check
check:
	cargo check

build: target/debug/repost
release: target/release/repost

target/debug/repost: Cargo.toml $(RS_FILES)
	cargo build

target/release/repost: Cargo.toml $(RS_FILES)
	cargo build --release

fmt: $(RS_FILES)
	cargo fmt

.PHONY: test
test:
	cargo test

.PHONY: clean
clean:
	$(RM) target/debug/repost
