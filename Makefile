# =============================================================================
# Makefile for financial_types
# Core financial type definitions for trading systems
# =============================================================================

# Detect current branch
CURRENT_BRANCH := $(shell git rev-parse --abbrev-ref HEAD)

# Project name for packaging
PROJECT_NAME := financial_types

# =============================================================================
# Default target
# =============================================================================
.PHONY: all
all: fmt lint test build

# =============================================================================
# 🔧 Build & Run
# =============================================================================

.PHONY: build
build:
	@echo "🔨 Building debug version..."
	cargo build

.PHONY: release
release:
	@echo "🚀 Building release version..."
	cargo build --release

.PHONY: clean
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean

# =============================================================================
# 🧪 Test & Quality
# =============================================================================

.PHONY: test
test:
	@echo "🧪 Running all tests..."
	RUST_LOG=warn cargo test --all-features

.PHONY: test-lib
test-lib:
	@echo "🧪 Running library tests..."
	RUST_LOG=warn cargo test --lib

.PHONY: test-doc
test-doc:
	@echo "🧪 Running documentation tests..."
	cargo test --doc

.PHONY: fmt
fmt:
	@echo "✨ Formatting code..."
	cargo +stable fmt --all

.PHONY: fmt-check
fmt-check:
	@echo "🔍 Checking code formatting..."
	cargo +stable fmt --all --check

.PHONY: lint
lint:
	@echo "🔍 Running clippy lints..."
	cargo clippy --all-targets --all-features -- -D warnings

.PHONY: lint-fix
lint-fix:
	@echo "🔧 Auto-fixing lint issues..."
	cargo clippy --fix --all-targets --all-features --allow-dirty --allow-staged -- -D warnings

.PHONY: fix
fix:
	@echo "🔧 Applying cargo fix suggestions..."
	cargo fix --allow-staged --allow-dirty

.PHONY: check
check: fmt-check lint test
	@echo "✅ All checks passed!"

.PHONY: pre-push
pre-push: fix fmt lint-fix test doc
	@echo "✅ All pre-push checks passed!"

# =============================================================================
# 📦 Packaging & Docs
# =============================================================================

.PHONY: doc
doc:
	@echo "📚 Generating documentation..."
	cargo doc --no-deps --document-private-items

.PHONY: doc-open
doc-open:
	@echo "📚 Opening documentation in browser..."
	cargo doc --no-deps --open

.PHONY: publish
publish:
	@echo "📦 Publishing to crates.io..."
	cargo publish --dry-run
	@echo "Dry run complete. Run 'cargo publish' to actually publish."

# =============================================================================
# 📈 Coverage
# =============================================================================

.PHONY: coverage
coverage: check-cargo-tarpaulin
	@echo "📊 Generating code coverage report (XML)..."
	@mkdir -p coverage
	RUST_LOG=warn cargo tarpaulin --verbose --all-features --timeout 120 --out Xml --output-dir coverage

# cargo-tarpaulin < 0.37.5 cannot read coverage data from Rust 1.99+, so a
# stale local install is upgraded rather than reused.
.PHONY: check-cargo-tarpaulin
check-cargo-tarpaulin:
	@v=$$(cargo tarpaulin --version 2>/dev/null | awk '{print $$NF}'); \
	if [ -z "$$v" ] || [ "$$(printf '%s\n' 0.37.5 "$$v" | sort -V | head -n1)" != "0.37.5" ]; then \
		echo "Installing cargo-tarpaulin >= 0.37.5..."; \
		cargo install cargo-tarpaulin --locked --version '>=0.37.5'; \
	fi

# =============================================================================
# 🚀 Release
# =============================================================================

.PHONY: version
version:
	@echo "📋 Current version:"
	@grep '^version' Cargo.toml | head -1

.PHONY: tag
tag:
	@echo "🏷️  Creating git tag..."
	@version=$$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/'); \
	git tag -a "v$$version" -m "Release v$$version"; \
	echo "Created tag v$$version"

# =============================================================================
# ❓ Help
# =============================================================================

.PHONY: help
help:
	@echo ""
	@echo "╔══════════════════════════════════════════════════════════════════════╗"
	@echo "║              financial_types - Development Commands                  ║"
	@echo "╚══════════════════════════════════════════════════════════════════════╝"
	@echo ""
	@echo "🔧 Build:"
	@echo "  make build           Compile the project (debug)"
	@echo "  make release         Build in release mode"
	@echo "  make clean           Clean build artifacts"
	@echo ""
	@echo "🧪 Test & Quality:"
	@echo "  make test            Run all tests"
	@echo "  make fmt             Format code"
	@echo "  make lint            Run clippy"
	@echo "  make lint-fix        Auto-fix lint issues"
	@echo "  make pre-push        Run all pre-push checks"
	@echo ""
	@echo "📦 Packaging & Docs:"
	@echo "  make doc             Generate documentation"
	@echo "  make publish         Dry-run publish to crates.io"
	@echo ""
