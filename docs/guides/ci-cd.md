# CI/CD Pipeline Documentation

This document describes the continuous integration and deployment pipelines for the project.

## Overview

The project uses GitHub Actions for CI/CD with three main workflows:

1. **CI Workflow** (`ci.yml`) - Run on every push and PR
2. **Release Workflow** (`release.yml`) - Run on version tags
3. **Nightly Workflow** (`nightly.yml`) - Run daily for testing bleeding edge

## CI Workflow

**Trigger**: Push to `main`/`develop` branches, pull requests

### Jobs

#### 1. Code Quality (`check`)
- Runs on `ubuntu-latest`
- Checks:
  - Code formatting (`cargo fmt`)
  - Linting (`cargo clippy`)
  - Documentation generation
- Uses caching for faster builds

#### 2. Test Suite (`test`)
- Matrix strategy: Ubuntu, macOS, Windows × Stable/Beta Rust
- Runs:
  - Unit tests
  - Integration tests
  - Doc tests
- Tests with different feature combinations:
  - Default features
  - All features
  - TUI-specific features

#### 3. TUI Binary Build (`build-tui`)
- Cross-compilation for:
  - Linux x86_64
  - Linux ARM64 (aarch64)
  - macOS x86_64
  - macOS ARM64 (Apple Silicon)
  - Windows x86_64
- Generates SHA256 checksums
- Uploads artifacts for 90 days

#### 4. DSL Executor Build (`build-executor`)
- Cross-compilation for major platforms
- Separate from TUI for independent versioning

#### 5. Security Audit (`audit`)
- Runs `cargo audit` to check for vulnerabilities
- Scans dependencies for known security issues

#### 6. Code Coverage (`coverage`)
- Generates coverage reports using `cargo-tarpaulin`
- Uploads to Codecov for tracking

## Release Workflow

**Trigger**: Git tags matching `v*.*.*` (e.g., `v0.1.0`)

### Process

1. **Create Release**
   - Parses version from tag
   - Creates GitHub release
   - Marks as pre-release if contains `alpha`, `beta`, or `rc`

2. **Build & Upload TUI Binaries**
   - Builds for all supported platforms
   - Strips binaries to reduce size
   - Generates SHA256 checksums
   - Uploads as release assets

3. **Build & Upload Executor Binaries**
   - Similar process for `periplon-executor`

4. **Publish to crates.io**
   - Automatically publishes library to Rust package registry
   - Requires `CARGO_REGISTRY_TOKEN` secret

5. **Combined Checksums**
   - Creates `SHA256SUMS` file with all checksums
   - Allows users to verify downloads

### Creating a Release

```bash
# 1. Bump version
./scripts/bump-version.sh patch  # or minor, major, or x.y.z

# 2. Review changes
git diff

# 3. Commit version bump
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to 0.1.1"

# 4. Create and push tag
git tag -a v0.1.1 -m "Release v0.1.1"
git push origin main
git push origin v0.1.1

# 5. GitHub Actions will automatically:
#    - Build binaries for all platforms
#    - Create GitHub release
#    - Upload assets
#    - Publish to crates.io
```

## Nightly Workflow

**Trigger**: Daily at 2 AM UTC, or manual dispatch

### Purpose

- Test bleeding-edge changes
- Catch platform-specific issues early
- Benchmark performance trends
- Provide nightly builds for testing

### Artifacts

- Retained for 7 days
- Named with `-nightly` suffix
- Available via GitHub Actions UI

## Platform Support Matrix

| Platform          | Target Triple              | CI | Release | Notes               |
|-------------------|----------------------------|----|---------|---------------------|
| Linux x86_64      | x86_64-unknown-linux-gnu   | ✓  | ✓       | Primary platform    |
| Linux ARM64       | aarch64-unknown-linux-gnu  | ✓  | ✓       | Server/cloud        |
| macOS x86_64      | x86_64-apple-darwin        | ✓  | ✓       | Intel Macs          |
| macOS ARM64       | aarch64-apple-darwin       | ✓  | ✓       | Apple Silicon       |
| Windows x86_64    | x86_64-pc-windows-msvc     | ✓  | ✓       | Primary Windows     |

## Caching Strategy

All workflows use aggressive caching to speed up builds:

- **Cargo Registry**: Downloaded crates metadata
- **Cargo Index**: Git index for crates.io
- **Build Artifacts**: Compiled dependencies

Cache keys include:
- OS (`runner.os`)
- Rust version (`matrix.rust`)
- `Cargo.lock` hash

## Required Secrets

Configure these in GitHub repository settings:

| Secret Name            | Description                    | Required For |
|------------------------|--------------------------------|--------------|
| `GITHUB_TOKEN`         | Auto-provided by GitHub        | All          |
| `CARGO_REGISTRY_TOKEN` | Token for publishing to crates.io | Releases |
| `CODECOV_TOKEN`        | Token for coverage uploads     | CI           |

### Getting Tokens

#### crates.io Token
1. Login to [crates.io](https://crates.io)
2. Go to Account Settings
3. Generate new API token
4. Add to GitHub Secrets as `CARGO_REGISTRY_TOKEN`

#### Codecov Token
1. Sign up at [codecov.io](https://codecov.io)
2. Add your repository
3. Copy the upload token
4. Add to GitHub Secrets as `CODECOV_TOKEN`

## Build Optimization

### Release Profile

Configured in `.cargo/config.toml`:

```toml
[profile.release]
opt-level = 3        # Maximum optimization
lto = true           # Link-time optimization
codegen-units = 1    # Single codegen unit for better optimization
strip = true         # Strip symbols
```

This produces highly optimized, small binaries at the cost of longer build times.

### CI Profile

```toml
[profile.ci]
inherits = "release"
strip = false        # Keep symbols for debugging
debug = true         # Include debug info
```

## Troubleshooting

### Build Fails on Cross-Compilation

**Linux ARM64**:
- Ensure `gcc-aarch64-linux-gnu` is installed
- Check linker configuration in `.cargo/config.toml`

**macOS ARM64**:
- Requires Xcode 12+ on runner
- May need `xcode-select --install`

### Test Failures

1. Check if platform-specific:
   - Review matrix build results
   - Look for OS-specific code paths

2. Check feature flags:
   - Ensure all required features are enabled
   - Verify conditional compilation

### Release Asset Upload Fails

- Verify `GITHUB_TOKEN` has proper permissions
- Check asset size limits (2GB per asset)
- Ensure unique asset names

### crates.io Publish Fails

- Verify version in `Cargo.toml` is new
- Check that all dependencies are published
- Ensure no git-based dependencies in release

## Local Testing

### Test CI Locally with Act

Install [act](https://github.com/nektos/act):

```bash
# Install act
brew install act  # macOS
# or download from GitHub releases

# Run CI workflow
act -W .github/workflows/ci.yml

# Run specific job
act -j test

# Run with secrets
act -s GITHUB_TOKEN=your_token
```

### Manual Cross-Compilation

```bash
# Install target
rustup target add aarch64-unknown-linux-gnu

# Build using cargo alias
cargo build-linux-arm

# Or manually
cargo build --release --target aarch64-unknown-linux-gnu --features tui
```

## Performance Tracking

### Benchmark Results

Nightly builds include benchmark runs:

1. Results uploaded as artifacts
2. Retained for 30 days
3. Compare trends over time

### Build Times

Monitor workflow execution times:

1. GitHub Actions provides timing data
2. Look for regressions in build duration
3. Investigate if builds exceed thresholds

## Best Practices

### For Contributors

1. **Run tests locally** before pushing:
   ```bash
   cargo test --all-features
   cargo clippy --all-targets --all-features
   cargo fmt --check
   ```

2. **Test on your platform**:
   ```bash
   cargo build --release --features tui
   ./target/release/periplon-tui --version
   ```

3. **Check CI status** before merging PRs

### For Maintainers

1. **Review dependency updates** regularly
2. **Monitor security advisories** via `cargo audit`
3. **Keep workflows updated** with latest Actions versions
4. **Test release process** before major versions
5. **Document breaking changes** in release notes

## Future Improvements

- [ ] Docker image builds
- [ ] Homebrew formula updates
- [ ] Windows installer (MSI)
- [ ] Linux packages (deb, rpm)
- [ ] Performance regression testing
- [ ] Automated changelog generation
- [ ] Signed releases (GPG)
- [ ] Universal macOS binaries (x86_64 + ARM64)

## References

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [cargo Documentation](https://doc.rust-lang.org/cargo/)
- [Cross-compilation Guide](https://rust-lang.github.io/rustup/cross-compilation.html)
- [cargo-release](https://github.com/crate-ci/cargo-release)
