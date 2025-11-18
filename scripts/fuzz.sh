#!/bin/bash
# Fuzz testing script for TYPF
# Usage: ./scripts/fuzz.sh [target] [duration_seconds]

set -e

TARGET="${1:-fuzz_unicode_process}"
DURATION="${2:-60}"

echo "=== TYPF Fuzz Testing ==="
echo "Target: $TARGET"
echo "Duration: ${DURATION}s"
echo ""

# Check if cargo-fuzz is installed
if ! command -v cargo-fuzz &> /dev/null; then
    echo "Installing cargo-fuzz..."
    cargo install cargo-fuzz
fi

# Available targets
TARGETS=(
    "fuzz_unicode_process"
    "fuzz_harfbuzz_shape"
    "fuzz_pipeline"
)

# Validate target
if [[ ! " ${TARGETS[@]} " =~ " ${TARGET} " ]]; then
    echo "ERROR: Invalid target '$TARGET'"
    echo "Available targets:"
    for t in "${TARGETS[@]}"; do
        echo "  - $t"
    done
    exit 1
fi

# Create corpus directory
mkdir -p fuzz/corpus/$TARGET

# Add seed inputs if corpus is empty
if [ -z "$(ls -A fuzz/corpus/$TARGET 2>/dev/null)" ]; then
    echo "Creating seed corpus..."
    mkdir -p fuzz/corpus/$TARGET

    case $TARGET in
        fuzz_unicode_process)
            echo "Hello, World!" > fuzz/corpus/$TARGET/hello.txt
            echo "مرحبا بالعالم" > fuzz/corpus/$TARGET/arabic.txt
            echo "你好世界" > fuzz/corpus/$TARGET/chinese.txt
            echo "שלום עולם" > fuzz/corpus/$TARGET/hebrew.txt
            echo "🎉🎊🎈" > fuzz/corpus/$TARGET/emoji.txt
            ;;
        fuzz_harfbuzz_shape)
            echo "abcdefg" > fuzz/corpus/$TARGET/latin.txt
            echo "مرحبا" > fuzz/corpus/$TARGET/arabic.txt
            echo "テスト" > fuzz/corpus/$TARGET/japanese.txt
            ;;
        fuzz_pipeline)
            echo "Test" > fuzz/corpus/$TARGET/simple.txt
            echo "Complex text with numbers 123" > fuzz/corpus/$TARGET/complex.txt
            ;;
    esac
fi

echo "Running fuzzer..."
echo "Corpus: fuzz/corpus/$TARGET"
echo "Artifacts will be saved to: fuzz/artifacts/$TARGET"
echo ""

# Run fuzzer with timeout
cd fuzz
cargo fuzz run $TARGET -- -max_total_time=$DURATION

echo ""
echo "=== Fuzzing Complete ==="
echo ""

# Check for crashes
if [ -d "artifacts/$TARGET" ] && [ "$(ls -A artifacts/$TARGET 2>/dev/null)" ]; then
    echo "⚠️  CRASHES FOUND!"
    echo "Crash files in: fuzz/artifacts/$TARGET"
    ls -lh artifacts/$TARGET
    exit 1
else
    echo "✓ No crashes found"
    exit 0
fi
