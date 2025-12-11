#!/bin/bash
# build.sh - Build all Rust crates and Python wheels
#
# Usage:
#   ./scripts/build.sh              # Build all in release mode
#   ./scripts/build.sh --debug      # Build in debug mode
#   ./scripts/build.sh --rust       # Build only Rust crates
#   ./scripts/build.sh --python     # Build only Python wheel
#
# Output:
#   Rust binaries: target/release/
#   Python wheels: target/wheels/

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$ROOT_DIR"

# Parse arguments
BUILD_RUST=true
BUILD_PYTHON=true
PROFILE="release"

while [[ $# -gt 0 ]]; do
    case $1 in
        --debug)
            PROFILE="debug"
            shift
            ;;
        --rust)
            BUILD_PYTHON=false
            shift
            ;;
        --python)
            BUILD_RUST=false
            shift
            ;;
        --release)
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--debug] [--rust] [--python]"
            exit 1
            ;;
    esac
done

echo "Building typf ($PROFILE profile)..."
echo ""

# Build Rust workspace (excluding Python bindings which need maturin)
if [[ "$BUILD_RUST" == "true" ]]; then
    echo "==> Building Rust workspace..."
    if [[ "$PROFILE" == "release" ]]; then
        cargo build --workspace --exclude typf-py --release
    else
        cargo build --workspace --exclude typf-py
    fi
    echo "    Rust build complete"
    echo ""
fi

# Build Python wheel
if [[ "$BUILD_PYTHON" == "true" ]]; then
    echo "==> Building Python wheel..."
    if command -v maturin &>/dev/null; then
        if [[ "$PROFILE" == "release" ]]; then
            maturin build --release
        else
            maturin build
        fi
        echo "    Python wheel built: target/wheels/"
    else
        echo "    Warning: maturin not found, skipping Python build"
        echo "    Install with: uv tool install maturin (or use: uvx maturin ...)"
    fi
    echo ""
fi

echo "Build complete!"
echo ""

# Show build artifacts
if [[ "$BUILD_RUST" == "true" ]]; then
    BINARY_PATH="target/${PROFILE}/typf"
    if [[ -f "$BINARY_PATH" ]]; then
        echo "Rust binary: $BINARY_PATH ($(du -h "$BINARY_PATH" | cut -f1))"
    fi
fi

if [[ "$BUILD_PYTHON" == "true" ]] && [[ -d "target/wheels" ]]; then
    echo "Python wheels:"
    ls -1 target/wheels/*.whl 2>/dev/null | while read -r wheel; do
        echo "  $wheel ($(du -h "$wheel" | cut -f1))"
    done
fi
