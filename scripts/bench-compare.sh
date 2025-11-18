#!/usr/bin/env bash
# Benchmark comparison script for TYPF
# Compares performance between two git commits

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

usage() {
    cat << USAGE
Usage: $0 [OPTIONS] <baseline-commit> <current-commit>

Compare benchmark results between two commits.

OPTIONS:
    -h, --help          Show this help message
    -o, --output FILE   Save comparison to file
    -v, --verbose       Show detailed benchmark output

EXAMPLES:
    # Compare main branch with current working directory
    $0 main HEAD

    # Compare two specific commits
    $0 abc1234 def5678

    # Save results to file
    $0 -o comparison.txt main HEAD
USAGE
}

# Parse arguments
OUTPUT_FILE=""
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            usage
            exit 0
            ;;
        -o|--output)
            OUTPUT_FILE="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        *)
            break
            ;;
    esac
done

if [ $# -ne 2 ]; then
    echo -e "${RED}Error: Requires two commit references${NC}"
    usage
    exit 1
fi

BASELINE=$1
CURRENT=$2

echo -e "${BLUE}=== TYPF Benchmark Comparison ===${NC}"
echo -e "Baseline: ${YELLOW}$BASELINE${NC}"
echo -e "Current:  ${YELLOW}$CURRENT${NC}"
echo

# Create temp directory for results
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

BASELINE_RESULTS="$TEMP_DIR/baseline.txt"
CURRENT_RESULTS="$TEMP_DIR/current.txt"

# Function to run benchmarks for a commit
run_benchmarks() {
    local commit=$1
    local output_file=$2
    
    echo -e "${BLUE}Running benchmarks for $commit...${NC}"
    
    # Checkout commit (save current state)
    CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
    STASH_RESULT=$(git stash push -m "bench-compare temp stash" 2>&1)
    
    git checkout "$commit" --quiet 2>/dev/null || {
        echo -e "${RED}Error: Could not checkout $commit${NC}"
        exit 1
    }
    
    # Run benchmarks
    if [ "$VERBOSE" = true ]; then
        cargo bench --workspace --all-features 2>&1 | tee "$output_file"
    else
        cargo bench --workspace --all-features > "$output_file" 2>&1
    fi
    
    # Restore previous state
    git checkout "$CURRENT_BRANCH" --quiet
    if [[ "$STASH_RESULT" != "No local changes to save" ]]; then
        git stash pop --quiet
    fi
}

# Run benchmarks for baseline
run_benchmarks "$BASELINE" "$BASELINE_RESULTS"
echo

# Run benchmarks for current
run_benchmarks "$CURRENT" "$CURRENT_RESULTS"
echo

# Compare results
echo -e "${BLUE}=== Comparison ===${NC}"
echo

# Extract benchmark times and compare
compare_benchmarks() {
    local baseline=$1
    local current=$2
    
    # Look for "time:" patterns in criterion output
    grep -E "time:\s+\[" "$baseline" > "$TEMP_DIR/baseline_times.txt" 2>/dev/null || true
    grep -E "time:\s+\[" "$current" > "$TEMP_DIR/current_times.txt" 2>/dev/null || true
    
    if [ ! -s "$TEMP_DIR/baseline_times.txt" ] || [ ! -s "$TEMP_DIR/current_times.txt" ]; then
        echo -e "${YELLOW}No benchmark timing data found${NC}"
        echo "This might happen if:"
        echo "  - No benchmarks are defined"
        echo "  - Benchmark format has changed"
        echo "  - Build failed"
        return
    fi
    
    echo -e "${GREEN}Performance Changes:${NC}"
    echo "------------------------------------------------------------"
    
    # Parse and compare (simplified - real implementation would be more robust)
    python3 << 'PYTHON' 2>/dev/null || echo "Python not available for detailed comparison"
import re
import sys

def parse_time(line):
    """Extract mean time from criterion output"""
    match = re.search(r'time:\s+\[[\d.]+\s+(\w+)\s+([\d.]+)\s+(\w+)\s+[\d.]+\s+(\w+)\]', line)
    if match:
        # Convert to nanoseconds
        value = float(match.group(2))
        unit = match.group(3)
        
        units = {'ns': 1, 'µs': 1000, 'us': 1000, 'ms': 1000000, 's': 1000000000}
        return value * units.get(unit, 1)
    return None

try:
    with open('$TEMP_DIR/baseline_times.txt') as f:
        baseline_lines = f.readlines()
    with open('$TEMP_DIR/current_times.txt') as f:
        current_lines = f.readlines()
    
    for i, (baseline, current) in enumerate(zip(baseline_lines, current_lines)):
        baseline_time = parse_time(baseline)
        current_time = parse_time(current)
        
        if baseline_time and current_time:
            change = ((current_time - baseline_time) / baseline_time) * 100
            
            if abs(change) < 1:
                status = "✓ Same"
                color = ""
            elif change < 0:
                status = f"↑ {abs(change):.1f}% faster"
                color = "\033[0;32m"  # Green
            else:
                status = f"↓ {change:.1f}% slower"
                color = "\033[0;31m"  # Red
            
            print(f"Benchmark {i+1}: {color}{status}\033[0m")
except Exception as e:
    print(f"Error comparing: {e}", file=sys.stderr)
PYTHON
    
    echo "------------------------------------------------------------"
}

compare_benchmarks "$BASELINE_RESULTS" "$CURRENT_RESULTS"

# Save to file if requested
if [ -n "$OUTPUT_FILE" ]; then
    {
        echo "TYPF Benchmark Comparison"
        echo "Baseline: $BASELINE"
        echo "Current: $CURRENT"
        echo "Date: $(date)"
        echo
        echo "=== Baseline Results ==="
        cat "$BASELINE_RESULTS"
        echo
        echo "=== Current Results ==="
        cat "$CURRENT_RESULTS"
    } > "$OUTPUT_FILE"
    
    echo
    echo -e "${GREEN}Results saved to: $OUTPUT_FILE${NC}"
fi

echo
echo -e "${BLUE}Benchmark comparison complete!${NC}"
