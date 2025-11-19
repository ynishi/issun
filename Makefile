.PHONY: help preflight publish test check build clean doc release-check release release-patch release-minor
.PHONY: fmt-examples clippy-examples check-examples test-examples build-examples clean-examples
.PHONY: server server-dev certs test-network

# Define examples directories
EXAMPLES := examples/junk-bot-game

help:
	@echo "Available targets:"
	@echo "  make check          - Run cargo check on all crates and examples"
	@echo "  make test           - Run all tests (workspace + examples)"
	@echo "  make test-network   - Run network tests with network feature"
	@echo "  make build          - Build all crates and examples"
	@echo "  make doc            - Generate documentation"
	@echo "  make clean          - Clean build artifacts (workspace + examples)"
	@echo "  make preflight      - Run all checks before publishing"
	@echo ""
	@echo "Server targets:"
	@echo "  make server         - Run relay server (release mode)"
	@echo "  make server-dev     - Run relay server (debug mode)"
	@echo "  make certs          - Generate self-signed TLS certificates"
	@echo ""
	@echo "Examples-specific targets:"
	@echo "  make fmt-examples      - Format all examples"
	@echo "  make clippy-examples   - Run clippy on all examples"
	@echo "  make check-examples    - Check all examples"
	@echo "  make test-examples     - Test all examples"
	@echo "  make build-examples    - Build all examples"
	@echo "  make clean-examples    - Clean all examples"
	@echo ""
	@echo "Release targets:"
	@echo "  make release-check  - Dry-run release with cargo-release"
	@echo "  make release        - Release patch version (0.x.y -> 0.x.y+1)"
	@echo "  make release-patch  - Release patch version (same as release)"
	@echo "  make release-minor  - Release minor version (0.x.y -> 0.x+1.0)"
	@echo "  make publish        - Publish to crates.io manually"

check:
	@echo "🔍 Checking all crates..."
	cargo check --all-targets --all-features
	@$(MAKE) check-examples

test:
	@echo "🧪 Running tests..."
	cargo test --all-targets --all-features
	cargo test --doc --all-features
	@$(MAKE) test-examples

build:
	@echo "🔨 Building all crates..."
	cargo build --all-features
	@$(MAKE) build-examples

doc:
	@echo "📚 Generating documentation..."
	cargo doc --all-features --no-deps --open

clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	@$(MAKE) clean-examples

# Server targets
certs:
	@echo "🔐 Generating self-signed TLS certificates..."
	@mkdir -p certs
	@openssl req -x509 -newkey rsa:4096 -keyout certs/key.pem -out certs/cert.pem \
		-days 365 -nodes -subj "/CN=localhost"
	@echo "✅ Certificates generated in certs/"

server: certs
	@echo "🚀 Starting ISSUN relay server (release mode)..."
	RUST_LOG=issun_server=info cargo run -p issun-server --release

server-dev: certs
	@echo "🚀 Starting ISSUN relay server (debug mode)..."
	RUST_LOG=issun_server=debug cargo run -p issun-server

test-network:
	@echo "🧪 Running network tests..."
	cargo test --features network

# Examples targets
fmt-examples:
	@echo "🎨 Formatting examples..."
	@for example in $(EXAMPLES); do \
		echo "  Formatting $$example..."; \
		cd $$example && cargo fmt && cd - > /dev/null; \
	done

clippy-examples:
	@echo "📎 Running clippy on examples..."
	@for example in $(EXAMPLES); do \
		echo "  Clippy on $$example..."; \
		cd $$example && cargo clippy --all-targets --fix --allow-dirty --allow-staged -- -D warnings && cd - > /dev/null; \
	done

check-examples:
	@echo "🔍 Checking examples..."
	@for example in $(EXAMPLES); do \
		echo "  Checking $$example..."; \
		cd $$example && cargo check --all-targets && cd - > /dev/null; \
	done

test-examples:
	@echo "🧪 Running tests in examples..."
	@for example in $(EXAMPLES); do \
		echo "  Testing $$example..."; \
		cd $$example && cargo test --all-targets && cd - > /dev/null; \
	done

build-examples:
	@echo "🔨 Building examples..."
	@for example in $(EXAMPLES); do \
		echo "  Building $$example..."; \
		cd $$example && cargo build && cd - > /dev/null; \
	done

clean-examples:
	@echo "🧹 Cleaning examples..."
	@for example in $(EXAMPLES); do \
		echo "  Cleaning $$example..."; \
		cd $$example && cargo clean && cd - > /dev/null; \
	done

preflight:
	@echo "🚦 Running preflight checks for the entire workspace..."
	@echo ""
	@echo "1️⃣  Formatting code..."
	cargo fmt --all
	@$(MAKE) fmt-examples
	@echo ""
	@echo "2️⃣  Running clippy (auto-fix)..."
	cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged -- -D warnings
	@$(MAKE) clippy-examples
	@echo ""
	@echo "3️⃣  Running tests..."
	cargo test --all-targets --all-features
	cargo test --doc --all-features
	@$(MAKE) test-examples
	@echo ""
	@echo "✅ All preflight checks passed!"

release-check:
	@echo "🔍 Dry-run release with cargo-release..."
	@echo ""
	@echo "Note: Install cargo-release if not already installed:"
	@echo "  cargo install cargo-release"
	@echo ""
	@echo "Checking patch release (0.x.y -> 0.x.y+1)..."
	cargo release patch

release-patch: preflight
	@echo "🚀 Releasing PATCH version with cargo-release..."
	@echo ""
	@echo "This will:"
	@echo "  - Update version numbers (0.x.y -> 0.x.y+1)"
	@echo "  - Create git commit and tag"
	@echo "  - (Publish step is manual, see make publish)"
	@echo ""
	@read -p "Continue? [y/N] " confirm && [ "$$confirm" = "y" ] || exit 1
	cargo release patch --execute --no-confirm

release-minor: preflight
	@echo "🚀 Releasing MINOR version with cargo-release..."
	@echo ""
	@echo "This will:"
	@echo "  - Update version numbers (0.x.y -> 0.x+1.0)"
	@echo "  - Create git commit and tag"
	@echo "  - (Publish step is manual, see make publish)"
	@echo ""
	@read -p "Continue? [y/N] " confirm && [ "$$confirm" = "y" ] || exit 1
	cargo release minor --execute --no-confirm

release: release-patch

publish: preflight
	@echo ""
	@echo "🚀 Starting sequential publish process..."
	@echo ""

	@echo "--- Step 1: Publishing issun-macros ---"
	@echo "  Running dry-run for issun-macros..."
	cargo publish -p issun-macros --dry-run --allow-dirty

	@echo "  ✓ Dry-run successful for issun-macros"
	@echo "  Publishing issun-macros to crates.io..."
	cargo publish -p issun-macros --allow-dirty

	@echo ""
	@echo "✅ issun-macros published successfully!"
	@echo ""
	@echo "⏳ Waiting 30 seconds for crates.io index to update..."
	sleep 30

	@echo ""
	@echo "--- Step 2: Publishing issun ---"
	@echo "  Running dry-run for issun..."
	cargo publish -p issun --dry-run --allow-dirty

	@echo "  ✓ Dry-run successful for issun"
	@echo "  Publishing issun to crates.io..."
	cargo publish -p issun --allow-dirty

	@echo ""
	@echo "✅ issun published successfully!"
	@echo ""
	@echo "🎉 All crates have been successfully published to crates.io!"
