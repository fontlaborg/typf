#!/usr/bin/env bash
# Simple benchmark runner for Typf

set -e

echo "Running Typf benchmarks..."
echo

# Run all benchmarks
cargo bench --workspace --all-features -- --output-format bencher | tee bench-results.txt

echo
echo "Results saved to bench-results.txt"
echo
echo "To compare with a previous run:"
echo "  ./scripts/bench-compare.sh <baseline-commit> HEAD"
