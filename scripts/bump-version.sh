#!/bin/bash
# Version bumping script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

usage() {
    echo "Usage: $0 <major|minor|patch|VERSION>"
    echo ""
    echo "Examples:"
    echo "  $0 patch      # 0.1.0 -> 0.1.1"
    echo "  $0 minor      # 0.1.0 -> 0.2.0"
    echo "  $0 major      # 0.1.0 -> 1.0.0"
    echo "  $0 1.2.3      # Set to 1.2.3"
    exit 1
}

if [ $# -ne 1 ]; then
    usage
fi

BUMP_TYPE=$1

# Get current version from Cargo.toml
CURRENT_VERSION=$(grep '^version = ' "$PROJECT_ROOT/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')
echo "Current version: $CURRENT_VERSION"

# Parse version components
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"

# Calculate new version
case "$BUMP_TYPE" in
    major)
        NEW_VERSION="$((MAJOR + 1)).0.0"
        ;;
    minor)
        NEW_VERSION="$MAJOR.$((MINOR + 1)).0"
        ;;
    patch)
        NEW_VERSION="$MAJOR.$MINOR.$((PATCH + 1))"
        ;;
    *)
        # Assume it's a version string
        if [[ ! "$BUMP_TYPE" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
            echo "Error: Invalid version format. Must be X.Y.Z"
            usage
        fi
        NEW_VERSION="$BUMP_TYPE"
        ;;
esac

echo "New version: $NEW_VERSION"

# Update Cargo.toml
sed -i.bak "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" "$PROJECT_ROOT/Cargo.toml"
rm -f "$PROJECT_ROOT/Cargo.toml.bak"

# Update Cargo.lock
cd "$PROJECT_ROOT"
cargo update -p periplon

echo ""
echo "✓ Version bumped to $NEW_VERSION"
echo ""
echo "Next steps:"
echo "  1. Review changes: git diff"
echo "  2. Commit: git add Cargo.toml Cargo.lock && git commit -m 'chore: bump version to $NEW_VERSION'"
echo "  3. Tag: git tag -a v$NEW_VERSION -m 'Release v$NEW_VERSION'"
echo "  4. Push: git push && git push --tags"
