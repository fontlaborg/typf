#!/bin/bash
# set-version.sh - Sync workspace version from git tag
#
# Usage:
#   ./scripts/set-version.sh           # Use latest git tag (e.g., v2.4.0 -> 2.4.0)
#   ./scripts/set-version.sh 2.4.0     # Set specific version
#   ./scripts/set-version.sh --check   # Only print current version, don't modify
#
# Requires: cargo-edit (cargo install cargo-edit)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$ROOT_DIR"

# Check if cargo-edit is installed
if ! cargo set-version --help &>/dev/null; then
    echo "Error: cargo-edit is required. Install with:"
    echo "  cargo install cargo-edit"
    exit 1
fi

# Handle --check flag
if [[ "${1:-}" == "--check" ]]; then
    # Extract version from workspace Cargo.toml
    VERSION=$(grep -E '^version\s*=' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
    echo "$VERSION"
    exit 0
fi

# Get version from argument or git tag
if [[ -n "${1:-}" ]]; then
    VERSION="$1"
    echo "Using provided version: $VERSION"
else
    # Try to get version from git tag
    if git describe --tags --abbrev=0 &>/dev/null; then
        TAG=$(git describe --tags --abbrev=0)
        VERSION="${TAG#v}"  # Remove 'v' prefix
        echo "Extracted version from git tag '$TAG': $VERSION"
    else
        echo "Error: No git tag found and no version provided."
        echo "Usage: $0 [VERSION]"
        echo "Example: $0 2.4.0"
        exit 1
    fi
fi

# Validate version format (semver)
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?(\+[a-zA-Z0-9.]+)?$ ]]; then
    echo "Error: Invalid version format '$VERSION'"
    echo "Expected semver format: MAJOR.MINOR.PATCH[-PRERELEASE][+BUILD]"
    exit 1
fi

# Update workspace version
echo "Setting workspace version to $VERSION..."
cargo set-version --workspace "$VERSION"

# Update pyproject.toml version
PYPROJECT="$ROOT_DIR/pyproject.toml"
if [[ -f "$PYPROJECT" ]]; then
    echo "Updating pyproject.toml version..."
    # Use sed to update the version line in pyproject.toml
    if grep -q '^version = ' "$PYPROJECT"; then
        sed -i.bak "s/^version = .*/version = \"$VERSION\"/" "$PYPROJECT"
        rm -f "$PYPROJECT.bak"
        echo "Updated pyproject.toml to version $VERSION"
    else
        echo "Note: pyproject.toml uses dynamic versioning, skipping"
    fi
fi

echo ""
echo "Version updated to $VERSION"
echo ""
echo "Files modified:"
git diff --name-only 2>/dev/null || true
