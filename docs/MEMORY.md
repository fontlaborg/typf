# Memory Profiling Guide

This guide covers memory profiling, leak detection, and optimization strategies for TYPF.

## Quick Start

```bash
# Run automated memory profiling
./scripts/profile-memory.sh typf-cli

# Profile specific component
cargo build --release --package typf-core
valgrind --tool=massif target/release/examples/basic
```

## Tools

### 1. Valgrind (Linux, macOS)

**Installation:**
```bash
# Ubuntu/Debian
sudo apt install valgrind

# macOS (may have compatibility issues on Apple Silicon)
brew install valgrind
```

**Leak Detection:**
```bash
valgrind --leak-check=full \
    --show-leak-kinds=all \
    --track-origins=yes \
    ./target/release/typf-cli --help
```

**Heap Profiling (Massif):**
```bash
valgrind --tool=massif \
    --massif-out-file=massif.out \
    ./target/release/typf-cli render "Test" output.png

# Visualize
ms_print massif.out | less
```

### 2. Heaptrack (Linux, macOS)

**Installation:**
```bash
# Ubuntu/Debian
sudo apt install heaptrack

# macOS
brew install heaptrack
```

**Usage:**
```bash
heaptrack ./target/release/typf-cli render "Test" output.png
heaptrack --analyze heaptrack.typf-cli.*.gz
```

### 3. cargo-profdata (Cross-platform)

**Installation:**
```bash
cargo install cargo-profdata
```

**Usage:**
```bash
cargo profdata -- ./target/release/typf-cli --help
```

### 4. Instruments (macOS only)

Use Xcode Instruments for detailed profiling:
1. Build with debug symbols: `cargo build --release`
2. Open Instruments.app
3. Select "Allocations" or "Leaks" template
4. Attach to running process or launch binary

## Memory Architecture

### Zero-Copy Font Loading

TYPF uses memory-mapped fonts with `Arc<Font>` for zero-copy sharing:

```rust
// Memory-mapped font data (no allocation)
let font_data = std::fs::read("font.ttf")?;
let font = Arc::new(Font::from_data(font_data)?);

// Share across threads without copying
let font_clone = Arc::clone(&font);
```

**Memory Impact:**
- Font file: 1-10 MB (mmap, not resident until accessed)
- Arc overhead: 8 bytes per clone
- No data duplication

### Multi-Level Cache

TYPF uses a three-tier cache system:

```
L1: DashMap (concurrent, ~1000 entries)  → ~100 KB
L2: LRU cache (~10,000 entries)          → ~1 MB
L3: Persistent (optional, disk-backed)   → Unlimited
```

**Memory Budget per 1M characters:**
- Shaping cache: ~50 MB (10,000 runs × 5 KB avg)
- Glyph cache: ~30 MB (5,000 glyphs × 6 KB avg bitmap)
- Font data: ~10 MB (mmap, shared)
- **Total: ~90 MB**

### Glyph Bitmap Memory

Bitmap sizes at different resolutions:

| Size (px) | Gray (1 bpp) | RGBA (4 bpp) |
|-----------|--------------|--------------|
| 16×16     | 256 B        | 1 KB         |
| 48×48     | 2.3 KB       | 9.2 KB       |
| 144×144   | 20 KB        | 81 KB        |

Cache limits prevent unbounded growth:
- L1: 1,000 glyphs × 6 KB avg = ~6 MB
- L2: 10,000 glyphs × 6 KB avg = ~60 MB

## Memory Targets

### Per-Operation Budgets

| Operation | Memory Budget | Notes |
|-----------|---------------|-------|
| Simple Latin (100 chars) | <10 KB | Shaping + small glyphs |
| Complex Arabic (100 chars) | <50 KB | RTL + ligatures |
| Render 1000 glyphs | <10 MB | With L1 cache hits |
| Full pipeline (1M chars) | <100 MB | With all caches |

### Leak Detection

TYPF must have **zero memory leaks**. All tests pass Valgrind memcheck.

**Known intentional "leaks":**
- `Box::leak()` in `typf-fontdb` for font data caching (documented)
- Static initialization in HarfBuzz bindings

## Common Issues

### Issue 1: Font Data Not Released

**Symptom:** Memory grows with each new font load

**Cause:** Not using `Arc<Font>` correctly

**Fix:**
```rust
// Bad: Creates new allocation
let font1 = Font::from_data(data.clone())?;
let font2 = Font::from_data(data.clone())?;

// Good: Share via Arc
let font = Arc::new(Font::from_data(data)?);
let font_ref = Arc::clone(&font);
```

### Issue 2: Cache Grows Unbounded

**Symptom:** Memory increases over time

**Cause:** Missing LRU eviction

**Fix:**
```rust
// Ensure cache has size limit
let cache = LruCache::new(NonZeroUsize::new(10_000).unwrap());
```

### Issue 3: Bitmap Not Released

**Symptom:** Peak memory after rendering

**Cause:** Bitmaps not dropped

**Fix:**
```rust
// Ensure RenderOutput is dropped after export
{
    let output = renderer.render(&shaped, font, &params)?;
    exporter.export(&output)?;
} // output dropped here
```

## Profiling Workflow

### 1. Establish Baseline

```bash
# Build release binary
cargo build --release --package typf-cli

# Profile baseline memory
./scripts/profile-memory.sh typf-cli
```

**Expected baseline:**
- Minimal build: <5 MB resident
- Full build: <20 MB resident
- After rendering 1000 chars: <30 MB

### 2. Profile Specific Workload

```rust
// Create test case
use typf::prelude::*;

fn profile_workload() {
    let typf = Typf::builder().build().unwrap();

    // Simulate real workload
    for i in 0..1000 {
        typf.render_text(
            &format!("Test string {}", i),
            "NotoSans",
            48.0,
            OutputFormat::Png
        ).unwrap();
    }
}
```

Run with profiler:
```bash
valgrind --tool=massif ./target/release/examples/profile_workload
```

### 3. Analyze Results

**Massif output:**
```
Peak memory: 45.2 MB at snapshot 127
```

**Key metrics:**
- Peak resident set size (RSS)
- Heap allocations over time
- Stack depth at peak
- Allocation sites (with debug symbols)

### 4. Optimize

**Strategies:**
1. **Reduce allocations:** Use `&str` instead of `String`
2. **Pool allocations:** Reuse buffers with `Vec::clear()`
3. **Lazy initialization:** Defer expensive allocations
4. **Arc sharing:** Share immutable data
5. **Cache tuning:** Adjust LRU sizes

## Benchmarking Memory

Use Criterion with custom memory measurement:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_memory(c: &mut Criterion) {
    c.bench_function("render_1000_chars", |b| {
        b.iter(|| {
            // Measure peak RSS during iteration
            render_text(black_box("a".repeat(1000)))
        });
    });
}
```

## CI Integration

Memory checks run automatically in CI:

```yaml
# .github/workflows/ci.yml
- name: Memory leak check
  run: |
    cargo build --release
    valgrind --leak-check=full --error-exitcode=1 \
      ./target/release/typf-cli --help
```

Fails build if:
- Memory leaks detected
- Peak RSS > threshold (100 MB for 1M chars)

## References

- [Valgrind Manual](https://valgrind.org/docs/manual/manual.html)
- [Heaptrack Documentation](https://github.com/KDE/heaptrack)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [TYPF BENCHMARKS.md](../BENCHMARKS.md) - Performance targets
