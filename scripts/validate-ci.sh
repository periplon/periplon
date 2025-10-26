#!/bin/bash
# Validate CI/CD workflow configurations

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "Validating CI/CD Workflows..."
echo ""

# Check if workflow files exist
echo "Checking workflow files..."
WORKFLOWS=(
    ".github/workflows/ci.yml"
    ".github/workflows/release.yml"
    ".github/workflows/nightly.yml"
)

for workflow in "${WORKFLOWS[@]}"; do
    if [ -f "$PROJECT_ROOT/$workflow" ]; then
        echo -e "${GREEN}✓${NC} Found: $workflow"
    else
        echo -e "${RED}✗${NC} Missing: $workflow"
        exit 1
    fi
done
echo ""

# Check for cargo-release
echo "Checking release tools..."
if command -v cargo-release &> /dev/null; then
    echo -e "${GREEN}✓${NC} cargo-release installed"
else
    echo -e "${YELLOW}!${NC} cargo-release not installed"
    echo "  Install with: cargo install cargo-release"
fi
echo ""

# Check Cargo.toml configuration
echo "Checking Cargo.toml configuration..."
if [ -f "$PROJECT_ROOT/Cargo.toml" ]; then
    # Check for required metadata
    if grep -q '^name = "periplon"' "$PROJECT_ROOT/Cargo.toml"; then
        echo -e "${GREEN}✓${NC} Package name configured"
    else
        echo -e "${RED}✗${NC} Package name not found"
        exit 1
    fi

    if grep -q '^version = ' "$PROJECT_ROOT/Cargo.toml"; then
        VERSION=$(grep '^version = ' "$PROJECT_ROOT/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')
        echo -e "${GREEN}✓${NC} Version: $VERSION"
    else
        echo -e "${RED}✗${NC} Version not found"
        exit 1
    fi

    if grep -q '^\[features\]' "$PROJECT_ROOT/Cargo.toml"; then
        echo -e "${GREEN}✓${NC} Features section exists"

        # Check for TUI feature
        if grep -q '^tui = ' "$PROJECT_ROOT/Cargo.toml"; then
            echo -e "${GREEN}✓${NC} TUI feature configured"
        else
            echo -e "${YELLOW}!${NC} TUI feature not found"
        fi
    fi

    # Check for binary targets
    if grep -q '\[\[bin\]\]' "$PROJECT_ROOT/Cargo.toml"; then
        echo -e "${GREEN}✓${NC} Binary targets configured"

        # Count binary targets
        BIN_COUNT=$(grep -c '\[\[bin\]\]' "$PROJECT_ROOT/Cargo.toml" || true)
        echo "  Found $BIN_COUNT binary target(s)"
    fi
else
    echo -e "${RED}✗${NC} Cargo.toml not found"
    exit 1
fi
echo ""

# Check .cargo/config.toml
echo "Checking cargo configuration..."
if [ -f "$PROJECT_ROOT/.cargo/config.toml" ]; then
    echo -e "${GREEN}✓${NC} Found .cargo/config.toml"

    # Check for cross-compilation targets
    if grep -q '\[target\.' "$PROJECT_ROOT/.cargo/config.toml"; then
        echo -e "${GREEN}✓${NC} Cross-compilation targets configured"
    fi

    # Check for release profile
    if grep -q '\[profile.release\]' "$PROJECT_ROOT/.cargo/config.toml"; then
        echo -e "${GREEN}✓${NC} Release profile configured"
    fi
else
    echo -e "${YELLOW}!${NC} .cargo/config.toml not found (optional)"
fi
echo ""

# Validate workflow YAML syntax
echo "Validating YAML syntax..."
if command -v yamllint &> /dev/null; then
    for workflow in "${WORKFLOWS[@]}"; do
        if yamllint -d relaxed "$PROJECT_ROOT/$workflow" &> /dev/null; then
            echo -e "${GREEN}✓${NC} Valid YAML: $workflow"
        else
            echo -e "${RED}✗${NC} Invalid YAML: $workflow"
            yamllint -d relaxed "$PROJECT_ROOT/$workflow"
            exit 1
        fi
    done
else
    echo -e "${YELLOW}!${NC} yamllint not installed - skipping YAML validation"
    echo "  Install with: pip install yamllint"
fi
echo ""

# Check for required Rust targets
echo "Checking installed Rust targets..."
REQUIRED_TARGETS=(
    "x86_64-unknown-linux-gnu"
    "x86_64-apple-darwin"
    "x86_64-pc-windows-msvc"
)

INSTALLED_TARGETS=$(rustup target list --installed 2>/dev/null || echo "")

for target in "${REQUIRED_TARGETS[@]}"; do
    if echo "$INSTALLED_TARGETS" | grep -q "$target"; then
        echo -e "${GREEN}✓${NC} Installed: $target"
    else
        echo -e "${YELLOW}!${NC} Not installed: $target"
        echo "  Run: rustup target add $target"
    fi
done
echo ""

# Test local builds
echo "Testing local builds..."

# Test default build
if cargo build --quiet 2>/dev/null; then
    echo -e "${GREEN}✓${NC} Default build successful"
else
    echo -e "${RED}✗${NC} Default build failed"
    exit 1
fi

# Test TUI build
if cargo build --quiet --features tui --bin periplon-tui 2>/dev/null; then
    echo -e "${GREEN}✓${NC} TUI build successful"
else
    echo -e "${RED}✗${NC} TUI build failed"
    exit 1
fi

# Test executor build
if cargo build --quiet --bin periplon-executor 2>/dev/null; then
    echo -e "${GREEN}✓${NC} Executor build successful"
else
    echo -e "${RED}✗${NC} Executor build failed"
    exit 1
fi
echo ""

# Check documentation
echo "Checking documentation..."
if [ -f "$PROJECT_ROOT/docs/ci-cd.md" ]; then
    echo -e "${GREEN}✓${NC} CI/CD documentation exists"
else
    echo -e "${YELLOW}!${NC} CI/CD documentation not found"
fi
echo ""

# Summary
echo "=================================================="
echo -e "${GREEN}✓ CI/CD validation complete!${NC}"
echo "=================================================="
echo ""
echo "Next steps:"
echo "  1. Review workflow configurations"
echo "  2. Set up required GitHub secrets:"
echo "     - CARGO_REGISTRY_TOKEN (for crates.io publishing)"
echo "     - CODECOV_TOKEN (for code coverage)"
echo "  3. Push changes to trigger CI"
echo "  4. Create a version tag to test release workflow"
echo ""
