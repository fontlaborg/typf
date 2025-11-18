#!/usr/bin/env bash
# Simple benchmark runner for TYPF

set -e

echo "Running TYPF benchmarks..."
echo

# Run all benchmarks
cargo bench --workspace --all-features -- --output-format bencher | tee bench-results.txt

echo
echo "Results saved to bench-results.txt"
echo
echo "To compare with a previous run:"
echo "  ./scripts/bench-compare.sh <baseline-commit> HEAD"
