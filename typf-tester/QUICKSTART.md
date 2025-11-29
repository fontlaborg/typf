# TypF Tester - Quick Start Guide

**Community project by FontLab** - https://www.fontlab.org/

A comprehensive testing and benchmarking tool for TypF text shaping and rendering backends.

## 5-Minute Quick Start

### 1. Setup

```bash
# Navigate to typf-tester directory
cd typf-tester

# Install dependencies
pip install fire pillow

# Verify installation
python typfme.py info
```

### 2. Test Rendering

```bash
# Render samples with all available backends
python typfme.py render

# View outputs
ls output/*.png
open output/render-*.png
```

### 3. Run Benchmarks

```bash
# Quick benchmark (50 iterations)
python typfme.py bench --iterations=50

# View results
cat output/benchmark_summary.md
```

That's it! You're now testing TypF backends.

---

## Common Tasks

### Render Specific Backend

```bash
# Test only HarfBuzz shaper
python typfme.py render --backend=harfbuzz

# Generate only SVG outputs
python typfme.py render --format=svg

# Both formats with specific backend
python typfme.py render --backend=none --format=both
```

### Benchmark Performance

```bash
# Full benchmark (100 iterations, default)
python typfme.py bench

# More iterations for accuracy
python typfme.py bench --iterations=1000

# Detailed per-test results
python typfme.py bench --detailed=True
```

### Specialized Benchmarks

```bash
# Shaping performance only (fast)
python typfme.py bench-shaping --iterations=1000

# Rendering performance only
python typfme.py bench-rendering --iterations=100

# Text length scaling analysis
python typfme.py bench-scaling --iterations=50
```

### Compare Backends

```bash
# Side-by-side comparison
python typfme.py compare

# View comparison outputs
open output/compare-*.png
```

---

## Understanding Output

### Directory Structure

```
typf-tester/
├── typfme.py              # Main test tool
├── fonts/                 # Test fonts
│   ├── Kalnia[wdth,wght].ttf
│   └── NotoNaskhArabic-Regular.ttf
└── output/                # Generated outputs
    ├── render-*.png       # Rendered samples
    ├── render-*.svg       # Vector outputs
    ├── compare-*.png      # Backend comparisons
    ├── benchmark_report.json        # Detailed JSON data
    ├── benchmark_summary.md         # Human-readable summary
    ├── shaping_benchmark.json       # Shaping-only data
    ├── rendering_benchmark.json     # Rendering-only data
    └── scaling_benchmark.json       # Scaling analysis
```

### Reading Benchmark Results

**Markdown Summary** (`output/benchmark_summary.md`):
```markdown
## Backend Performance

| Backend | Avg Time (ms) | Ops/sec | Success |
|---------|---------------|---------|---------|
| HARFBUZZ + OPIXA | 1.096 | 2471 | 100% |
```

**Interpretation:**
- **Avg Time**: Lower is better (milliseconds per operation)
- **Ops/sec**: Higher is better (operations per second)
- **Success**: Should be 100%

---

## Troubleshooting

### "Module 'typf' has no attribute 'Typf'"

The Python bindings need to be rebuilt:

```bash
# Rebuild Python bindings
cd ../bindings/python
maturin develop --release --features shaping-hb,export-png,export-svg

# Verify
python -c "import typf; print(typf.__version__)"
```

### "Font not found" Warnings

Test fonts are missing. Download them or use your own:

```bash
# Place fonts in typf-tester/fonts/
cp /path/to/your/font.ttf fonts/
```

### All Backends Show "✗ Not available"

1. Check Python bindings are built: `python -c "import typf; print(dir(typf))"`
2. Rebuild if needed (see above)
3. Verify with: `python typfme.py info`

### Benchmark Fails with "Invalid dimensions"

Text is too long for bitmap rendering. Solutions:

1. Use smaller font sizes: `python typfme.py bench --detailed=True`
2. Use SVG export: `python typfme.py render --format=svg`
3. See `docs/PERFORMANCE.md` for detailed strategies

---

## Advanced Usage

### Custom Test Texts

Edit `typfme.py` to add your own sample texts:

```python
self.sample_texts = {
    "latn": "AVAST Wallflower Efficiency",
    "mytext": "Your custom text here",  # Add this line
}
```

### Automated Testing

Run benchmarks in CI/CD:

```bash
# Exit with error if benchmarks fail
python typfme.py bench --iterations=10 || exit 1

# Parse JSON for regression detection
cat output/benchmark_report.json | jq '.successful_tests'
```

### Performance Analysis

```bash
# Compare shaping backends
python typfme.py bench-shaping --iterations=1000 > shaping_results.txt

# Analyze rendering scaling
python typfme.py bench-scaling --detailed=True

# Check if scaling is linear
cat output/scaling_benchmark.json | jq '.results[] | {text, char_count, us_per_char}'
```

---

## Interpreting Performance Results

### Shaping Performance (~30-47µs)

**Fast** (shaping is NOT the bottleneck):
- NONE shaper: ~36µs (simple left-to-right)
- HarfBuzz: ~47µs (full OpenType shaping)

**What this means:**
- Shaping takes <0.05ms per operation
- Optimizing shaping won't improve overall performance
- Focus optimization efforts on rendering

### Rendering Performance (~1100µs)

**Slow** (rendering IS the bottleneck):
- Opixa renderer: ~1122µs (37x slower than shaping)

**What this means:**
- Rendering dominates total time (97%)
- Font size scaling is super-linear (O(size²))
- This is expected for bitmap rasterization

### Text Length Scaling

**Linear scaling** (good):
- 17 chars: ~0.7ms
- 27 chars: ~1.5ms
- Roughly proportional increase

**Limit discovered** (~200-300 chars at 48px):
- Bitmap width limit: ~10,000 pixels
- Use SVG export or line wrapping for longer texts

---

## Next Steps

1. **Choose the right backend**: Read [Backend Comparison Guide](../docs/BACKEND_COMPARISON.md)
2. **Learn optimization strategies**: Read [Performance Guide](../docs/PERFORMANCE.md)
3. **Handle long texts**: See [Long Text Example](../examples/long_text_handling.rs)
4. **Explore all examples**: Check [Examples README](../examples/README.md)
5. **Production deployment**: Review [Project Status](../PROJECT_STATUS.md)

---

## Command Reference

### Rendering Commands

| Command | Purpose | Example |
|---------|---------|---------|
| `render` | Test all backends | `python typfme.py render` |
| `render --backend=X` | Test specific shaper | `--backend=harfbuzz` |
| `render --format=X` | Specific format | `--format=svg` |
| `compare` | Side-by-side comparison | `python typfme.py compare` |

### Benchmarking Commands

| Command | Purpose | Iterations | Output |
|---------|---------|------------|--------|
| `bench` | Full benchmark | 100 (default) | JSON + Markdown |
| `bench-shaping` | Shaping only | 1000 (default) | JSON |
| `bench-rendering` | Rendering only | 100 (default) | JSON |
| `bench-scaling` | Text length scaling | 50 (default) | JSON |

### Information Commands

| Command | Purpose |
|---------|---------|
| `info` | Show environment info |

### Common Options

- `--iterations=N` - Number of benchmark iterations
- `--detailed=True` - Show per-test results
- `--backend=X` - Filter by shaper (none, harfbuzz)
- `--format=X` - Output format (png, svg, both)

---

## Performance Targets vs. Actual

From comprehensive benchmarking (see `output/benchmark_summary.md`):

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| Shaping (100 chars) | <50µs | ~47µs | ✅ Achieved |
| Rendering (16px) | <1µs/glyph | ~195µs total | ✅ Good |
| Rendering (128px) | N/A | ~2935µs | ⚠️ Super-linear |
| Throughput | >1000 ops/sec | 2400 ops/sec | ✅ Exceeded |

**Conclusion:** Performance targets met. Rendering optimization opportunities exist for large font sizes.

---

## Getting Help

- **Documentation**: See `README.md` in this directory
- **Performance Guide**: Read `../docs/PERFORMANCE.md`
- **Examples**: Check `../examples/README.md`
- **Project Status**: Review `../PROJECT_STATUS.md`
- **Issues**: Report at https://github.com/fontlab/typf/issues

---

**Community project by FontLab** - Professional font editing software
https://www.fontlab.org/
