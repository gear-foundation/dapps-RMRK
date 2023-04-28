.PHONY: all build clean fmt fmt-check init linter pre-commit test

all: init build test

build:
	@echo ──────────── Build release ────────────────────
	@cargo +nightly build --release
	@ls -l ./target/wasm32-unknown-unknown/release/*.wasm

clean:
	@echo ──────────── Clean ────────────────────────────
	@rm -rvf target

fmt:
	@echo ──────────── Format ───────────────────────────
	@cargo fmt --all

fmt-check:
	@echo ──────────── Check format ─────────────────────
	@cargo fmt --all -- --check

init:
	@echo ──────────── Install toolchains ───────────────
ifeq ($(shell uname -s),Linux)
	@echo Linux detected..
	make pin-toolchain-linux
else ifeq ($(shell uname -s),Darwin)
	@echo Macos detected..
	make pin-toolchain-mac-m1
endif


pin-toolchain-mac-m1:
	@rustup toolchain install nightly-2023-03-20 --component llvm-tools-preview
	@rustup target add wasm32-unknown-unknown --toolchain nightly-2023-03-20
	@rm -rf ~/.rustup/toolchains/nightly-aarch64-apple-darwin
	@ln -s ~/.rustup/toolchains/nightly-2023-03-20-aarch64-apple-darwin ~/.rustup/toolchains/nightly-aarch64-apple-darwin

pin-toolchain-linux:
	@rustup toolchain install nightly-2023-03-20 --component llvm-tools-preview
	@rustup target add wasm32-unknown-unknown --toolchain nightly-2023-03-20
	@rm -rf ~/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu
	@ln -s ~/.rustup/toolchains/nightly-2023-03-20-x86_64-unknown-linux-gnu ~/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu
	@rustup component add clippy --toolchain nightly-x86_64-unknown-linux-gnu

linter:
	@echo ──────────── Run linter ───────────────────────
	@cargo +nightly clippy --all-targets -- --no-deps -D warnings -A "clippy::missing_safety_doc"

pre-commit: fmt linter test

test: build
	@echo ──────────── Run tests ────────────────────────
	@cargo +nightly test --release
