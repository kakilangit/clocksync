# ==============================================================================
# Makefile for clocksync
# ==============================================================================
#
# This Makefile provides convenient commands for development, testing, and
# publishing the clocksync crate. It uses the same commands as the GitHub
# Actions CI/CD workflows to ensure consistency between local development
# and continuous integration.
#
# The Rust version is automatically extracted from Cargo.toml
# to ensure alignment with the project's rust-version requirement.
#
# Usage:
#   make help          - Show all available targets
#   make ci            - Run all CI checks (fmt-check, clippy, test, etc.)
#   make pre-commit    - Run pre-commit checks (fmt, clippy, test)
#   make publish       - Publish to crates.io
#
# ==============================================================================

.PHONY: help
help: ## Show this help message
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

# Rust version - automatically extracted from Cargo.toml
RUST_VERSION := $(shell grep 'rust-version = ' Cargo.toml | head -1 | sed 's/.*rust-version = "\(.*\)"/\1/')

.PHONY: install-rust
install-rust: ## Install Rust toolchain with required components
	rustup toolchain install $(RUST_VERSION)
	rustup component add rustfmt clippy --toolchain $(RUST_VERSION)

.PHONY: check
check: ## Run cargo check
	cargo +$(RUST_VERSION) check

.PHONY: fmt
fmt: ## Format all code
	cargo +$(RUST_VERSION) fmt --all

.PHONY: fmt-check
fmt-check: ## Check code formatting
	cargo +$(RUST_VERSION) fmt --all --check

.PHONY: clippy
clippy: ## Run clippy lints
	cargo +$(RUST_VERSION) clippy --all-targets -- -D warnings

.PHONY: test
test: ## Run all tests
	cargo +$(RUST_VERSION) test

.PHONY: test-doc
test-doc: ## Run documentation tests
	cargo +$(RUST_VERSION) test --doc

.PHONY: build
build: ## Build the crate
	cargo +$(RUST_VERSION) build

.PHONY: build-release
build-release: ## Build in release mode
	cargo +$(RUST_VERSION) build --release

.PHONY: doc
doc: ## Generate documentation
	cargo +$(RUST_VERSION) doc --no-deps

.PHONY: doc-open
doc-open: ## Generate and open documentation
	cargo +$(RUST_VERSION) doc --no-deps --open

.PHONY: clean
clean: ## Clean build artifacts
	cargo clean

.PHONY: ci
ci: fmt-check clippy test test-doc ## Run all CI checks

.PHONY: pre-commit
pre-commit: fmt clippy test ## Run pre-commit checks

.PHONY: verify-version
verify-version: ## Verify version in Cargo.toml
	@VERSION=$$(grep '^version = ' Cargo.toml | head -1 | sed 's/.*version = "\(.*\)"/\1/'); \
	echo "  clocksync version: $$VERSION"; \
	EDITION=$$(grep '^edition = ' Cargo.toml | head -1 | sed 's/.*edition = "\(.*\)"/\1/'); \
	echo "  edition: $$EDITION"; \
	RUST_VER=$$(grep '^rust-version = ' Cargo.toml | head -1 | sed 's/.*rust-version = "\(.*\)"/\1/'); \
	echo "  rust-version: $$RUST_VER"

.PHONY: publish-dry-run
publish-dry-run: ## Dry run of publishing to crates.io
	cargo +$(RUST_VERSION) publish --dry-run

.PHONY: publish
publish: ## Publish to crates.io (requires CARGO_REGISTRY_TOKEN)
	cargo +$(RUST_VERSION) publish

.PHONY: update-deps
update-deps: ## Update dependencies
	cargo update

.PHONY: all
all: ci doc ## Run all checks and build documentation
