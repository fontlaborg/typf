#!/bin/bash
# Count tests and update README badge
# Usage: ./scripts/count-tests.sh

set -e

# Run tests with verbose output and count
echo "Running tests to count them..."
TEST_OUTPUT=$(cargo test --workspace --all-features -- --list 2>&1)

# Count total tests (lines ending with ": test")
TEST_COUNT=$(echo "$TEST_OUTPUT" | grep -E ": test$" | wc -l | tr -d ' ')

echo "Total tests found: $TEST_COUNT"

# Update README.md badge
if [ -f "README.md" ]; then
    # Use sed to update the badge (macOS and Linux compatible)
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        sed -i '' "s/tests-[0-9]*%20passing/tests-${TEST_COUNT}%20passing/g" README.md
    else
        # Linux
        sed -i "s/tests-[0-9]*%20passing/tests-${TEST_COUNT}%20passing/g" README.md
    fi
    echo "Updated README.md badge to show $TEST_COUNT tests"
else
    echo "ERROR: README.md not found"
    exit 1
fi
