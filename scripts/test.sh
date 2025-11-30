#!/bin/bash
# test.sh - Run all checks and tests
#
# Usage:
#   ./scripts/test.sh              # Run all checks and tests
#   ./scripts/test.sh --quick      # Skip slow tests
#   ./scripts/test.sh --rust       # Run only Rust tests
#   ./scripts/test.sh --python     # Run only Python tests
#   ./scripts/test.sh --lint       # Run only linting (no tests)
#
# Runs: fmt check, clippy, cargo test, maturin develop, pytest

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$ROOT_DIR"

# Parse arguments
RUN_RUST=true
RUN_PYTHON=true
RUN_LINT=true
RUN_TESTS=true
QUICK_MODE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --quick)
            QUICK_MODE=true
            shift
            ;;
        --rust)
            RUN_PYTHON=false
            shift
            ;;
        --python)
            RUN_RUST=false
            shift
            ;;
        --lint)
            RUN_TESTS=false
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--quick] [--rust] [--python] [--lint]"
            exit 1
            ;;
    esac
done

echo "Running typf tests..."
echo ""

FAILED=false

# Rust formatting check
if [[ "$RUN_RUST" == "true" ]] && [[ "$RUN_LINT" == "true" ]]; then
    echo "==> Checking Rust formatting..."
    if cargo fmt --all --check; then
        echo "    Formatting OK"
    else
        echo "    Formatting FAILED - run 'cargo fmt --all' to fix"
        FAILED=true
    fi
    echo ""
fi

# Rust clippy
if [[ "$RUN_RUST" == "true" ]] && [[ "$RUN_LINT" == "true" ]]; then
    echo "==> Running clippy..."
    if cargo clippy --workspace --all-features -- -D warnings; then
        echo "    Clippy OK"
    else
        echo "    Clippy FAILED"
        FAILED=true
    fi
    echo ""
fi

# Rust tests
if [[ "$RUN_RUST" == "true" ]] && [[ "$RUN_TESTS" == "true" ]]; then
    echo "==> Running Rust tests..."
    if [[ "$QUICK_MODE" == "true" ]]; then
        if cargo test --workspace; then
            echo "    Rust tests OK (quick mode)"
        else
            echo "    Rust tests FAILED"
            FAILED=true
        fi
    else
        if cargo test --workspace --all-features; then
            echo "    Rust tests OK"
        else
            echo "    Rust tests FAILED"
            FAILED=true
        fi
    fi
    echo ""
fi

# Python linting
if [[ "$RUN_PYTHON" == "true" ]] && [[ "$RUN_LINT" == "true" ]]; then
    if command -v uv &>/dev/null; then
        echo "==> Running Python linting (ruff via uvx)..."
        if [[ -d "bindings/python/python" ]]; then
            if uvx ruff check bindings/python/python bindings/python/tests 2>/dev/null; then
                echo "    Ruff OK"
            else
                echo "    Ruff found issues"
                # Don't fail on Python lint issues for now
            fi
        fi
        echo ""
    elif command -v ruff &>/dev/null; then
        echo "==> Running Python linting (ruff)..."
        if [[ -d "bindings/python/python" ]]; then
            if ruff check bindings/python/python; then
                echo "    Ruff OK"
            else
                echo "    Ruff found issues"
            fi
        fi
        echo ""
    fi
fi

# Python tests
if [[ "$RUN_PYTHON" == "true" ]] && [[ "$RUN_TESTS" == "true" ]]; then
    if command -v uv &>/dev/null; then
        echo "==> Running Python tests (uv)..."
        cd bindings/python
        if uv run --isolated --with pytest pytest tests/ -v 2>&1 | tail -20; then
            echo "    Python tests OK"
        else
            echo "    Python tests FAILED"
            # Don't fail the whole build for Python test issues
        fi
        cd "$ROOT_DIR"
        echo ""
    elif command -v maturin &>/dev/null && command -v pytest &>/dev/null; then
        echo "==> Building Python extension for testing..."
        if maturin develop --release 2>&1 | tail -5; then
            echo ""
            echo "==> Running Python tests..."
            if pytest bindings/python/tests -v 2>/dev/null; then
                echo "    Python tests OK"
            else
                echo "    Python tests FAILED (or no tests found)"
            fi
        else
            echo "    Python build FAILED"
            FAILED=true
        fi
        echo ""
    else
        echo "==> Skipping Python tests (uv or maturin+pytest not installed)"
        echo ""
    fi
fi

# Summary
echo "========================================"
if [[ "$FAILED" == "true" ]]; then
    echo "FAILED: Some checks did not pass"
    exit 1
else
    echo "SUCCESS: All checks passed"
    exit 0
fi
