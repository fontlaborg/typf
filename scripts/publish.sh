#!/bin/bash
# publish.sh - Publish to crates.io and PyPI
#
# Usage:
#   ./scripts/publish.sh                    # Publish all (requires tokens)
#   ./scripts/publish.sh --dry-run          # Simulate without publishing
#   ./scripts/publish.sh --crates           # Publish only to crates.io
#   ./scripts/publish.sh --pypi             # Publish only to PyPI
#
# Environment variables:
#   CRATES_IO_TOKEN - Token for crates.io (or use `cargo login` first)
#   PYPI_API_TOKEN  - Token for PyPI
#
# Publishing order respects dependency DAG with 30s delays between layers.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$ROOT_DIR"

# Parse arguments
DRY_RUN=false
PUBLISH_CRATES=true
PUBLISH_PYPI=true

while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --crates)
            PUBLISH_PYPI=false
            shift
            ;;
        --pypi)
            PUBLISH_CRATES=false
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--dry-run] [--crates] [--pypi]"
            exit 1
            ;;
    esac
done

# Get current version
VERSION=$("$SCRIPT_DIR/set-version.sh" --check)
echo "Publishing typf version $VERSION"
echo ""

if [[ "$DRY_RUN" == "true" ]]; then
    echo "[DRY RUN MODE - no actual publishing]"
    echo ""
fi

# Verify version matches git tag
if git describe --tags --abbrev=0 &>/dev/null; then
    TAG=$(git describe --tags --abbrev=0)
    TAG_VERSION="${TAG#v}"
    if [[ "$VERSION" != "$TAG_VERSION" ]]; then
        echo "WARNING: Cargo version ($VERSION) differs from git tag ($TAG_VERSION)"
        echo "Run './scripts/set-version.sh' to sync versions first"
        if [[ "$DRY_RUN" != "true" ]]; then
            exit 1
        fi
    fi
fi

# Define publishing layers (dependency order)
# Layer 0: No internal dependencies
LAYER_0=(
    "crates/typf-core"
    "crates/typf-unicode"
)

# Layer 1: Depends on Layer 0
LAYER_1=(
    "crates/typf-fontdb"
    "crates/typf-input"
    "crates/typf-export"
    "backends/typf-shape-none"
    "backends/typf-render-opixa"
    "backends/typf-render-json"
    "backends/typf-render-svg"
)

# Layer 2: Depends on Layer 1
LAYER_2=(
    "crates/typf-export-svg"
    "backends/typf-shape-hb"
    "backends/typf-render-color"
    "backends/typf-render-zeno"
    "backends/typf-render-skia"
    "backends/typf-os"
)

# Layer 3: Depends on Layer 2
LAYER_3=(
    "backends/typf-shape-icu-hb"
    "backends/typf-shape-ct"
    "backends/typf-render-cg"
    "backends/typf-os-mac"
    "backends/typf-os-win"
)

# Layer 4: Main crate
LAYER_4=(
    "crates/typf"
)

# Layer 5: CLI and bench (depend on main typf)
LAYER_5=(
    "crates/typf-cli"
    "crates/typf-bench"
)

publish_layer() {
    local layer_name="$1"
    shift
    local crates=("$@")

    echo "==> Publishing $layer_name..."
    for crate_path in "${crates[@]}"; do
        crate_name=$(basename "$crate_path")
        echo "    Publishing $crate_name..."

        if [[ "$DRY_RUN" == "true" ]]; then
            echo "    [DRY RUN] cargo publish -p $crate_name --no-verify"
        else
            if [[ -n "${CRATES_IO_TOKEN:-}" ]]; then
                cargo publish -p "$crate_name" --no-verify --token "$CRATES_IO_TOKEN" || {
                    echo "    Warning: Failed to publish $crate_name (may already exist)"
                }
            else
                cargo publish -p "$crate_name" --no-verify || {
                    echo "    Warning: Failed to publish $crate_name (may already exist)"
                }
            fi
        fi
    done
    echo ""
}

wait_for_index() {
    local seconds="${1:-30}"
    if [[ "$DRY_RUN" == "true" ]]; then
        echo "    [DRY RUN] Would wait ${seconds}s for crates.io index update"
    else
        echo "    Waiting ${seconds}s for crates.io index update..."
        sleep "$seconds"
    fi
    echo ""
}

# Publish to crates.io
if [[ "$PUBLISH_CRATES" == "true" ]]; then
    echo "========================================"
    echo "Publishing to crates.io"
    echo "========================================"
    echo ""

    publish_layer "Layer 0 (core)" "${LAYER_0[@]}"
    wait_for_index 30

    publish_layer "Layer 1 (basic)" "${LAYER_1[@]}"
    wait_for_index 30

    publish_layer "Layer 2 (advanced)" "${LAYER_2[@]}"
    wait_for_index 30

    publish_layer "Layer 3 (platform)" "${LAYER_3[@]}"
    wait_for_index 30

    publish_layer "Layer 4 (typf)" "${LAYER_4[@]}"
    wait_for_index 30

    publish_layer "Layer 5 (binaries)" "${LAYER_5[@]}"

    echo "Crates.io publishing complete"
    echo ""
fi

# Publish to PyPI
if [[ "$PUBLISH_PYPI" == "true" ]]; then
    echo "========================================"
    echo "Publishing to PyPI"
    echo "========================================"
    echo ""

    if ! command -v maturin &>/dev/null; then
        echo "Error: maturin not found. Install with: uv tool install maturin"
        exit 1
    fi

    echo "==> Building Python wheel..."
    maturin build --release
    echo ""

    echo "==> Publishing to PyPI..."
    if [[ "$DRY_RUN" == "true" ]]; then
        echo "    [DRY RUN] maturin publish"
    else
        if [[ -n "${PYPI_API_TOKEN:-}" ]]; then
            # Use token from environment
            maturin publish --username __token__ --password "$PYPI_API_TOKEN"
        else
            # Rely on ~/.pypirc or interactive auth
            maturin publish
        fi
    fi

    echo ""
    echo "PyPI publishing complete"
fi

echo ""
echo "========================================"
echo "Publishing complete: typf v$VERSION"
echo "========================================"
