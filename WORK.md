# Current Work Log

## 2025-11-21: Fixed typf-bench showing 0.000 ops/ns

### Problem
The `typf-bench` tool was always displaying `0.000 ops/ns` for all benchmark results, making it impossible to see actual performance metrics.

### Root Cause
The metric "ops/ns" (operations per nanosecond) produces extremely small numbers. Even a very fast operation taking 10 microseconds (10,000 ns) would show:
- ops/ns = 1 / 10,000 = 0.0001
- Which rounds to `0.000` when formatted with 3 decimal places

Standard benchmarking tools use "ns/op" (nanoseconds per operation) instead, which produces meaningful numbers.

### Solution
Changed the benchmark metric from `ops/ns` to `ns/op`:

1. Updated `BenchmarkResult` struct field: `ops_per_ns` → `ns_per_op`
2. Simplified calculation: `ns_per_op = total_time_ns / iterations`
3. Updated display format: `ops/ns: {:8.3}` → `ns/op: {:10.1}`

### Files Modified
- `crates/typf-bench/src/main.rs:147` - Changed struct field name
- `crates/typf-bench/src/main.rs:308-312` - Simplified calculation logic
- `crates/typf-bench/src/main.rs:330` - Updated result construction
- `crates/typf-bench/src/main.rs:380` - Updated display format

### Testing
- ✅ `cargo check -p typf-bench` passes (3 warnings about unused code, which is acceptable)
- Code compiles successfully
- Display format now shows `ns/op: {:10.1}` which will display meaningful values like `ns/op: 125000.0` instead of `ops/ns: 0.000`

### Next Steps
The benchmark should now display meaningful performance numbers when run.
