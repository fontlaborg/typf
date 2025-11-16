# Edge List Allocation Strategy

## Decision: Use Vec<Edge> (No Custom Pool)

**Date:** 2025-11-15
**Status:** Final

## Benchmark Results

Tested 4 allocation patterns with 100 edges:

| Operation | Time | Analysis |
|-----------|------|----------|
| `push()` without capacity | 521 ns | Includes reallocation overhead |
| `with_capacity()` + `push()` | 280 ns | **1.86x faster** - eliminates reallocs |
| `insert_sorted()` | 3.6 µs | Binary search + insert (expected O(n log n)) |
| `sort_by_x()` | 92 ns | Very fast for already-sorted data |

## Key Findings

1. **Pre-allocation wins**: `with_capacity()` is 1.86x faster than reallocating
2. **Vec is fast**: 280ns for 100 edges = 2.8ns per edge
3. **Sorting is cheap**: 92ns to sort 100 edges (already sorted case)
4. **insert_sorted is expensive**: 3.6µs = 13x slower than bulk operations

## Implementation Strategy

### For Edge Tables (per-scanline storage)
- Use `EdgeList::with_capacity()` when edge count is known
- Typical glyph has <100 edges, so pre-allocate Vec::with_capacity(128)
- Reuse EdgeList instances via `clear()` to avoid allocations

### For Active Edge List
- Use `EdgeList::new()` initially
- Build via `push()` from edge table (already sorted by Y)
- Use `sort_by_x()` when needed (very fast)
- Use `remove_inactive()` to prune (avoids allocation)

## Why NOT Custom Pool?

### Cons of Custom Pool
- **Complexity**: Need allocator, free list, fragmentation handling
- **Memory overhead**: Pool must hold max edges (wasted for simple glyphs)
- **Cache locality**: Vec is sequential, pool may scatter edges
- **Maintenance burden**: More code to test and optimize

### Pros of Vec
- **Simple**: Standard library, well-tested
- **Fast**: Benchmarks show excellent performance
- **Flexible**: Grows as needed, no pre-tuning required
- **Cache-friendly**: Sequential allocation
- **Reusable**: `clear()` keeps allocation for reuse

## Performance Target Met

- **Target**: <1ms for typical glyph rasterization
- **Edge management**: 280ns for 100 edges = 0.28% of budget
- **Verdict**: Vec overhead is negligible

## Recommendation

**Stick with `Vec<Edge>` unless profiling shows it's a bottleneck.**

If we ever need custom allocation:
1. Profile first with `cargo flamegraph`
2. Identify actual hot path
3. Only optimize if edge allocation >5% of total time

## Code Pattern

```rust
// Good: Pre-allocate when possible
let mut edge_table = vec![EdgeList::with_capacity(128); height];

// Good: Reuse allocations
active_edges.clear();
for y in 0..height {
    active_edges.extend(&edge_table[y]);
}

// Bad: Repeated insert_sorted (13x slower)
for edge in edges {
    active_edges.insert_sorted(edge);  // O(n log n) per edge
}
```

## Future Work

If profiling identifies edge allocation as bottleneck (>5% of render time):
- Consider arena allocator (bumpalo crate)
- Consider small Vec optimization (smallvec crate for <N edges)
- Consider object pool for EdgeList reuse across glyphs

**For now: KISS principle - Vec is perfect.**
