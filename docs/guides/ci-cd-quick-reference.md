# CI/CD Quick Reference

Quick commands and workflows for development and release.

## Daily Development

### Before Committing

```bash
# Format code
cargo fmt

# Run linter
cargo clippy --all-targets --all-features

# Run tests
cargo test --all-features

# Validate CI configuration
./scripts/validate-ci.sh
```

### Create a Pull Request

```bash
# Create feature branch
git checkout -b feature/my-feature

# Make changes, commit, push
git add .
git commit -m "feat: add new feature"
git push origin feature/my-feature

# CI will automatically run on PR
```

## Building Locally

### Quick Builds

```bash
# Build TUI
cargo build --release --bin periplon-tui --features tui

# Build executor
cargo build --release --bin periplon-executor

# Build everything
cargo build --release --all-features
```

### Using Cargo Aliases

```bash
# Defined in .cargo/config.toml
cargo build-tui          # Build TUI binary
cargo build-executor     # Build executor binary
cargo test-all           # Run all tests
```

### Cross-Platform Build

```bash
# Build for specific platform
cargo build-linux-x86     # Linux x86_64
cargo build-linux-arm     # Linux ARM64
cargo build-macos-x86     # macOS Intel
cargo build-macos-arm     # macOS Apple Silicon
cargo build-windows       # Windows x86_64
```

### Local Release Build

```bash
# Build release binaries for current platform
./scripts/build-release-local.sh

# Outputs to ./dist/
# Includes checksums and tests
```

## Creating Releases

### Semantic Versioning

- **Patch** (0.1.0 → 0.1.1): Bug fixes, minor changes
- **Minor** (0.1.0 → 0.2.0): New features, backward compatible
- **Major** (0.1.0 → 1.0.0): Breaking changes

### Release Process

```bash
# 1. Bump version
./scripts/bump-version.sh patch  # or minor, major, 1.2.3

# 2. Review and commit
git diff
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to X.Y.Z"

# 3. Create tag
git tag -a vX.Y.Z -m "Release vX.Y.Z"

# 4. Push
git push origin main
git push origin vX.Y.Z

# 5. GitHub Actions will:
#    - Build binaries for all platforms
#    - Create GitHub release
#    - Upload assets
#    - Publish to crates.io
```

### Hotfix Release

```bash
# Create hotfix branch from tag
git checkout -b hotfix/X.Y.Z vX.Y.Z

# Make fix, test
# ... edit files ...
cargo test

# Bump patch version
./scripts/bump-version.sh patch

# Commit and tag
git commit -am "fix: critical bug"
git tag -a vX.Y.(Z+1) -m "Hotfix vX.Y.(Z+1)"

# Push
git push origin hotfix/X.Y.Z
git push origin vX.Y.(Z+1)

# Merge back to main
git checkout main
git merge hotfix/X.Y.Z
git push origin main
```

## Monitoring CI

### Check Workflow Status

```bash
# Using GitHub CLI
gh run list --workflow=ci.yml
gh run view --log         # View latest run

# View specific run
gh run view <run-id> --log
```

### Download Build Artifacts

```bash
# List artifacts
gh run list --workflow=ci.yml
gh run view <run-id>

# Download artifacts
gh run download <run-id>
```

## Troubleshooting

### CI Build Fails

1. **Check logs**:
   ```bash
   gh run view <run-id> --log
   ```

2. **Reproduce locally**:
   ```bash
   # Run the same commands as CI
   cargo fmt --all -- --check
   cargo clippy --all-targets --all-features
   cargo test --all-features
   ```

3. **Test specific platform**:
   ```bash
   # Install target
   rustup target add x86_64-unknown-linux-gnu

   # Build
   cargo build --target x86_64-unknown-linux-gnu --features tui
   ```

### Release Fails

1. **Check tag format**: Must be `vX.Y.Z` (e.g., `v0.1.0`)

2. **Verify secrets**:
   - `CARGO_REGISTRY_TOKEN` set in GitHub secrets
   - `GITHUB_TOKEN` has proper permissions

3. **Check version uniqueness**:
   ```bash
   # Version must be new on crates.io
   cargo search periplon
   ```

### Cross-Compilation Issues

**Linux ARM64**:
```bash
# Install cross-compilation tools
sudo apt-get install gcc-aarch64-linux-gnu

# Set linker
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc

# Build
cargo build --target aarch64-unknown-linux-gnu
```

**macOS Universal Binary**:
```bash
# Build both architectures
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Combine with lipo
lipo -create \
  target/x86_64-apple-darwin/release/periplon-tui \
  target/aarch64-apple-darwin/release/periplon-tui \
  -output target/release/periplon-tui-universal
```

## Workflow Files

| File | Purpose | Trigger |
|------|---------|---------|
| `.github/workflows/ci.yml` | Continuous integration | Push, PR |
| `.github/workflows/release.yml` | Release builds | Tags `v*.*.*` |
| `.github/workflows/nightly.yml` | Nightly builds | Daily 2 AM UTC |

## Scripts

| Script | Purpose |
|--------|---------|
| `scripts/bump-version.sh` | Bump semantic version |
| `scripts/validate-ci.sh` | Validate CI configuration |
| `scripts/build-release-local.sh` | Build release locally |

## Environment Variables

### Required for Release

```bash
# GitHub (set in repository secrets)
GITHUB_TOKEN          # Auto-provided
CARGO_REGISTRY_TOKEN  # From crates.io

# Optional
CODECOV_TOKEN         # For code coverage
```

### Optional Build Variables

```bash
# Skip CLI version check
export PERIPLON_SKIP_VERSION_CHECK=1

# Custom Rust flags
export RUSTFLAGS="-C target-cpu=native"

# Parallel compilation
export CARGO_BUILD_JOBS=8
```

## Common Tasks Cheatsheet

```bash
# Full local validation (before commit)
cargo fmt && cargo clippy --all-targets --all-features && cargo test --all-features

# Quick build and test TUI
cargo build --features tui --bin periplon-tui && ./target/debug/periplon-tui --version

# Build optimized release
cargo build --release --features tui

# Run benchmarks
cargo bench --bench dsl_benchmarks

# Check for outdated dependencies
cargo outdated

# Update dependencies
cargo update

# Audit security issues
cargo audit

# Generate documentation
cargo doc --no-deps --all-features --open

# Clean build artifacts
cargo clean

# Full release test (local)
./scripts/validate-ci.sh && ./scripts/build-release-local.sh
```

## GitHub Actions Badges

Add to README:

```markdown
[![CI](https://github.com/USERNAME/periplon/workflows/CI/badge.svg)](https://github.com/USERNAME/periplon/actions)
[![Release](https://github.com/USERNAME/periplon/workflows/Release/badge.svg)](https://github.com/USERNAME/periplon/releases)
[![Codecov](https://codecov.io/gh/USERNAME/periplon/branch/main/graph/badge.svg)](https://codecov.io/gh/USERNAME/periplon)
```

## Resources

- [CI/CD Documentation](./ci-cd.md) - Full documentation
- [GitHub Actions Docs](https://docs.github.com/en/actions)
- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [Rust Cross-Compilation](https://rust-lang.github.io/rustup/cross-compilation.html)
