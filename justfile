# Periplon SDK - Just Commands
# Run `just --list` to see all available commands

# Default recipe - show available commands
default:
    @just --list

# Run all pre-commit checks (format, clippy, tests)
pre-commit: fmt-check clippy test
    @echo "✅ All pre-commit checks passed!"

# Full CI check (what runs in GitHub Actions)
ci: fmt-check clippy test build-release
    @echo "✅ CI checks passed!"

# Format code
fmt:
    cargo fmt --all

# Check formatting without modifying files
fmt-check:
    cargo fmt --all -- --check

# Run clippy with strict warnings
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Run clippy and automatically fix issues
clippy-fix:
    cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged

# Build library and all binaries (debug)
build:
    cargo build --all-features

# Build library and all binaries (release)
build-release:
    cargo build --release --all-features

# Build DSL executor binary
build-executor:
    cargo build --release --bin periplon-executor

# Build DSL TUI binary
build-tui:
    cargo build --release --bin periplon-tui --features tui

# Run all tests
test:
    cargo test --all-features

# Run tests with output visible
test-verbose:
    cargo test --all-features -- --nocapture

# Run only unit tests (excluding integration tests)
test-unit:
    cargo test --lib --all-features

# Run integration tests (requires CLI installed)
test-integration:
    cargo test --test integration_tests

# Run specific test by name
test-name TEST:
    cargo test {{TEST}} -- --nocapture

# Check code without building
check:
    cargo check --all-targets --all-features

# Run benchmarks
bench:
    cargo bench --bench dsl_benchmarks

# Clean build artifacts
clean:
    cargo clean

# Update dependencies
update:
    cargo update

# Run simple query example
example-simple:
    cargo run --example simple_query

# Run interactive client example
example-interactive:
    cargo run --example interactive_client

# Run DSL executor example
example-dsl:
    cargo run --example dsl_executor_example

# Generate DSL template
dsl-template:
    cargo run --release --bin periplon-executor -- template

# Generate workflow from natural language description
dsl-generate DESC OUTPUT:
    cargo run --release --bin periplon-executor -- generate "{{DESC}}" -o {{OUTPUT}}

# Validate DSL workflow file
dsl-validate FILE:
    cargo run --release --bin periplon-executor -- validate {{FILE}}

# Run DSL workflow
dsl-run FILE:
    cargo run --release --bin periplon-executor -- run {{FILE}}

# Launch TUI
tui:
    cargo run --release --bin periplon-tui --features tui

# Launch TUI with custom workflow directory
tui-dir DIR:
    cargo run --release --bin periplon-tui --features tui -- --workflow-dir {{DIR}}

# Install binaries to cargo bin directory
install:
    cargo install --path . --bin periplon-executor
    cargo install --path . --bin periplon-tui --features tui

# Watch for changes and run tests
watch:
    cargo watch -x test

# Watch for changes and run clippy
watch-clippy:
    cargo watch -x clippy

# Generate documentation and open in browser
doc:
    cargo doc --all-features --open

# Check for outdated dependencies
outdated:
    cargo outdated

# Run cargo audit to check for security vulnerabilities
audit:
    cargo audit

# Full quality check (everything)
quality: fmt-check clippy test bench doc
    @echo "✅ Full quality check passed!"

# Publishing workflow recipes

# Build and test documentation locally
doc-test:
    cargo doc --all-features --no-deps
    @echo "✅ Documentation built successfully!"

# Check if ready to publish (runs all pre-publish checks)
publish-check: ci doc-test
    @echo "🔍 Running publish dry-run..."
    cargo publish --dry-run --allow-dirty
    @echo "✅ Ready to publish!"

# Dry run publish without actually publishing
publish-dry:
    cargo publish --dry-run

# Login to crates.io (run this first with your API token)
publish-login:
    @echo "Please enter your crates.io API token:"
    @cargo login

# Verify package is ready for publishing
publish-verify: fmt-check clippy test-unit build-release doc-test
    @echo "✅ All pre-publish checks passed!"
    @echo ""
    @echo "Next steps:"
    @echo "  1. Review the package: just publish-dry"
    @echo "  2. Login to crates.io: just publish-login"
    @echo "  3. Publish: just publish"

# Publish to crates.io (requires login first)
publish: publish-verify
    @echo "🚀 Publishing to crates.io..."
    cargo publish
    @echo "✅ Published successfully!"
    @echo ""
    @echo "Check your package at: https://crates.io/crates/periplon"
    @echo "Documentation will be available at: https://docs.rs/periplon"
