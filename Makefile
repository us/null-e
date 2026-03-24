.PHONY: hooks fmt lint build test check

## Install git pre-commit hook
hooks:
	@git config core.hooksPath .githooks
	@echo "Pre-commit hook installed."

## Format source code
fmt:
	@cargo fmt --all

## Run linter
lint:
	@cargo clippy -- -D warnings

## Build the project
build:
	@cargo build

## Run tests
test:
	@cargo test

## Run all checks in CI order: format → lint → build → test
check: fmt lint build test
