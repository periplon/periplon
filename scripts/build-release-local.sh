#!/bin/bash
# Local release build script for testing

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_ROOT/dist"

echo "Building release binaries locally..."
echo ""

# Create build directory
mkdir -p "$BUILD_DIR"
rm -rf "$BUILD_DIR"/*

# Get version from Cargo.toml
VERSION=$(grep '^version = ' "$PROJECT_ROOT/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')
echo "Building version: $VERSION"
echo ""

# Detect current platform
OS=$(uname -s)
ARCH=$(uname -m)

case "$OS" in
    Linux)
        case "$ARCH" in
            x86_64)
                TARGET="x86_64-unknown-linux-gnu"
                PLATFORM="linux-x86_64"
                ;;
            aarch64|arm64)
                TARGET="aarch64-unknown-linux-gnu"
                PLATFORM="linux-aarch64"
                ;;
            *)
                echo "Unsupported architecture: $ARCH"
                exit 1
                ;;
        esac
        ;;
    Darwin)
        case "$ARCH" in
            x86_64)
                TARGET="x86_64-apple-darwin"
                PLATFORM="macos-x86_64"
                ;;
            arm64)
                TARGET="aarch64-apple-darwin"
                PLATFORM="macos-aarch64"
                ;;
            *)
                echo "Unsupported architecture: $ARCH"
                exit 1
                ;;
        esac
        ;;
    MINGW*|MSYS*|CYGWIN*)
        TARGET="x86_64-pc-windows-msvc"
        PLATFORM="windows-x86_64"
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

echo "Building for: $PLATFORM (target: $TARGET)"
echo ""

# Check if target is installed
if ! rustup target list --installed | grep -q "$TARGET"; then
    echo "Installing target: $TARGET"
    rustup target add "$TARGET"
    echo ""
fi

# Build TUI binary
echo "Building periplon-tui..."
cargo build --release --bin periplon-tui --features tui --target "$TARGET"
echo "✓ periplon-tui built successfully"
echo ""

# Build executor binary
echo "Building periplon-executor..."
cargo build --release --bin periplon-executor --target "$TARGET"
echo "✓ periplon-executor built successfully"
echo ""

# Copy binaries to dist directory
echo "Copying binaries to dist/..."

if [ "$OS" = "MINGW*" ] || [ "$OS" = "MSYS*" ] || [ "$OS" = "CYGWIN*" ]; then
    # Windows
    cp "$PROJECT_ROOT/target/$TARGET/release/periplon-tui.exe" "$BUILD_DIR/periplon-tui-$PLATFORM.exe"
    cp "$PROJECT_ROOT/target/$TARGET/release/periplon-executor.exe" "$BUILD_DIR/periplon-executor-$PLATFORM.exe"

    # Calculate checksums
    cd "$BUILD_DIR"
    certutil -hashfile "periplon-tui-$PLATFORM.exe" SHA256 > "periplon-tui-$PLATFORM.exe.sha256"
    certutil -hashfile "periplon-executor-$PLATFORM.exe" SHA256 > "periplon-executor-$PLATFORM.exe.sha256"
else
    # Unix-like (Linux, macOS)
    cp "$PROJECT_ROOT/target/$TARGET/release/periplon-tui" "$BUILD_DIR/periplon-tui-$PLATFORM"
    cp "$PROJECT_ROOT/target/$TARGET/release/periplon-executor" "$BUILD_DIR/periplon-executor-$PLATFORM"

    # Strip binaries
    echo "Stripping binaries..."
    strip "$BUILD_DIR/periplon-tui-$PLATFORM" || true
    strip "$BUILD_DIR/periplon-executor-$PLATFORM" || true

    # Calculate checksums
    cd "$BUILD_DIR"
    shasum -a 256 "periplon-tui-$PLATFORM" > "periplon-tui-$PLATFORM.sha256"
    shasum -a 256 "periplon-executor-$PLATFORM" > "periplon-executor-$PLATFORM.sha256"
fi

echo "✓ Binaries copied to dist/"
echo ""

# Generate combined checksums file
cat *.sha256 > SHA256SUMS
echo "✓ Generated SHA256SUMS"
echo ""

# Display binary info
echo "Build complete!"
echo ""
echo "Binaries in $BUILD_DIR:"
ls -lh "$BUILD_DIR"
echo ""

# Test binaries
echo "Testing binaries..."
cd "$PROJECT_ROOT"

if [ "$OS" = "MINGW*" ] || [ "$OS" = "MSYS*" ] || [ "$OS" = "CYGWIN*" ]; then
    "$BUILD_DIR/periplon-tui-$PLATFORM.exe" --version
    "$BUILD_DIR/periplon-executor-$PLATFORM.exe" --version
else
    "$BUILD_DIR/periplon-tui-$PLATFORM" --version
    "$BUILD_DIR/periplon-executor-$PLATFORM" --version
fi

echo ""
echo "✓ All binaries working correctly!"
echo ""
echo "Distribution files ready in: $BUILD_DIR"
