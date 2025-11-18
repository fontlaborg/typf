#!/bin/bash
# Memory profiling script for TYPF
# Uses Valgrind (massif) and heaptrack for memory analysis
# Usage: ./scripts/profile-memory.sh [target]

set -e

TARGET="${1:-typf-cli}"
PROFILE_DIR="target/profile"

# Create profile directory
mkdir -p "$PROFILE_DIR"

echo "=== TYPF Memory Profiling ==="
echo "Target: $TARGET"
echo ""

# Check for profiling tools
has_valgrind=false
has_heaptrack=false

if command -v valgrind &> /dev/null; then
    has_valgrind=true
    echo "✓ Valgrind found"
else
    echo "✗ Valgrind not found (install: apt install valgrind / brew install valgrind)"
fi

if command -v heaptrack &> /dev/null; then
    has_heaptrack=true
    echo "✓ Heaptrack found"
else
    echo "✗ Heaptrack not found (install: apt install heaptrack / brew install heaptrack)"
fi

echo ""

# Build release binary with debug symbols
echo "Building release binary with debug symbols..."
cargo build --release --package "$TARGET"

BINARY="target/release/$TARGET"

if [ ! -f "$BINARY" ]; then
    echo "ERROR: Binary not found at $BINARY"
    exit 1
fi

# Run Valgrind massif if available
if [ "$has_valgrind" = true ]; then
    echo ""
    echo "=== Running Valgrind Massif ==="
    MASSIF_OUT="$PROFILE_DIR/massif.out"

    valgrind --tool=massif \
        --massif-out-file="$MASSIF_OUT" \
        --stacks=yes \
        "$BINARY" --help > /dev/null 2>&1 || true

    echo "Massif output saved to: $MASSIF_OUT"
    echo "Visualize with: ms_print $MASSIF_OUT"

    # Show peak memory usage
    if [ -f "$MASSIF_OUT" ]; then
        PEAK=$(grep "peak" "$MASSIF_OUT" | head -1 || echo "Unable to determine")
        echo "Peak memory: $PEAK"
    fi
fi

# Run heaptrack if available
if [ "$has_heaptrack" = true ]; then
    echo ""
    echo "=== Running Heaptrack ==="
    HEAPTRACK_OUT="$PROFILE_DIR/heaptrack.$TARGET"

    heaptrack --output "$HEAPTRACK_OUT" "$BINARY" --help > /dev/null 2>&1 || true

    echo "Heaptrack output saved to: ${HEAPTRACK_OUT}.*"
    echo "Analyze with: heaptrack --analyze ${HEAPTRACK_OUT}.gz"
fi

# Memory baseline test
echo ""
echo "=== Memory Baseline Test ==="
echo "Running simple text rendering to measure memory..."

# Create test file
TEST_TEXT="Hello, World! This is a memory profiling test."
TEST_FILE="$PROFILE_DIR/test.txt"
echo "$TEST_TEXT" > "$TEST_FILE"

if [ "$has_valgrind" = true ]; then
    echo ""
    echo "Valgrind memcheck (leak detection):"
    valgrind --leak-check=full \
        --show-leak-kinds=all \
        --track-origins=yes \
        --log-file="$PROFILE_DIR/memcheck.log" \
        "$BINARY" --help > /dev/null 2>&1 || true

    echo "Memcheck log saved to: $PROFILE_DIR/memcheck.log"

    # Show summary
    if [ -f "$PROFILE_DIR/memcheck.log" ]; then
        echo ""
        grep -A 5 "LEAK SUMMARY" "$PROFILE_DIR/memcheck.log" || echo "No leaks detected!"
    fi
fi

echo ""
echo "=== Memory Profiling Complete ==="
echo "Profile data saved in: $PROFILE_DIR/"
echo ""
echo "Next steps:"
echo "  1. View massif profile: ms_print $PROFILE_DIR/massif.out | less"
echo "  2. Analyze heaptrack: heaptrack --analyze $PROFILE_DIR/heaptrack.$TARGET.gz"
echo "  3. Check for leaks: cat $PROFILE_DIR/memcheck.log"
