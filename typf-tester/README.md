# TypF Tester

Comprehensive testing and benchmarking tool for TypF text rendering pipeline.

Made by FontLab https://www.fontlab.com/

## Overview

`typfme.py` is a comprehensive CLI tool for testing, rendering, and benchmarking all TypF backend combinations with multiple sample texts, font sizes, and output formats. It helps validate backend implementations, compare performance, and generate visual outputs for quality verification.

## Features

- Test all shaping + rendering backend combinations
- Render samples in PNG and SVG formats
- Benchmark performance across multiple texts and font sizes
- Generate detailed performance reports
- Support for multiple scripts (Latin, Arabic, mixed, etc.)
- Visual comparison of backend outputs

## Installation

### Prerequisites

1. Build TypF Python bindings with required features:

```bash
# From repository root
cd bindings/python
maturin develop --release --features shaping-hb,export-png,export-svg
```

2. Install Python dependencies:

```bash
cd typf-tester
pip install -r requirements.txt
```

Or using uv:

```bash
uv pip install -r requirements.txt
```

## Usage

### Info Command

Display information about available backends, fonts, and sample texts:

```bash
python typfme.py info
```

Output:
- TypF version
- Available shaping/rendering backends
- Test fonts with sizes
- Sample text catalog

### Render Command

Render sample texts with all or specific backends:

```bash
# Render with all backends, both PNG and SVG
python typfme.py render

# Render only PNG outputs
python typfme.py render --format=png

# Render only SVG outputs
python typfme.py render --format=svg

# Render with specific backend
python typfme.py render --backend=harfbuzz
```

Output files are saved to `typf-tester/output/` with naming pattern:
- `render-{shaper}-{renderer}-{text_name}.{format}`

Example:
- `render-harfbuzz-orge-simple_latin.png`
- `render-none-orge-arabic.svg`

### Compare Command

Generate side-by-side comparison of all backends using the same text:

```bash
python typfme.py compare
```

Outputs both PNG and SVG for visual comparison:
- `compare-{shaper}-{renderer}.png`
- `compare-{shaper}-{renderer}.svg`

### Bench Command

Comprehensive benchmarking across all backend combinations:

```bash
# Standard benchmark (100 iterations)
python typfme.py bench

# Extended benchmark (1000 iterations for more precision)
python typfme.py bench --iterations=1000

# Detailed output (per-text, per-size results)
python typfme.py bench --detailed=True
```

Benchmark Results:
- Average time per operation (ms)
- Operations per second
- Success rate
- Performance by text complexity
- **Automatic regression detection** - flags backends >10% slower than previous run
- Detailed JSON report saved to `output/benchmark_report.json`
- Markdown summary with regression warnings in `output/benchmark_summary.md`

### Performance Regression Detection

The benchmark tool automatically detects performance regressions:

- Compares each run against the previous baseline
- Flags any backend combination that's >10% slower
- Displays top 10 regressions in console output
- Includes full regression table in Markdown summary
- JSON report contains `regressions` array with detailed metrics

Example regression warning:
```
⚠️  PERFORMANCE REGRESSIONS DETECTED (>10% slowdown)
  coretext + skia    mixd    32px
    Baseline: 0.571ms → Current: 1.674ms
    Slowdown: +193.3%
```

This helps catch accidental performance degradations during development.

## Sample Texts

The tool includes diverse sample texts for comprehensive testing:

| Text Name       | Content                                  | Purpose                      |
|-----------------|------------------------------------------|------------------------------|
| simple_latin    | The quick brown fox jumps over the lazy dog. | Basic Latin text    |
| complex_latin   | AVAST Wallflower Efficiency              | Kerning & ligatures          |
| arabic          | مرحبا بك في العالم                      | RTL, complex shaping         |
| mixed           | Hello, مرحبا, 你好!                      | Mixed scripts                |
| numbers         | 0123456789                               | Numeric glyphs               |
| punctuation     | !@#$%^&*()_+-=[]{}...                    | Special characters           |

## Test Fonts

Three fonts are included for testing:

1. **Kalnia** (`Kalnia[wdth,wght].ttf`) - Variable font with width and weight axes
2. **Noto Sans** (`NotoSans-Regular.ttf`) - Standard Latin font
3. **Noto Naskh Arabic** (`NotoNaskhArabic-Regular.ttf`) - Arabic script support

## Available Backends

### Shaping Backends

- **none** - Simple left-to-right advancement (no complex shaping)
- **harfbuzz** - Full OpenType shaping with HarfBuzz

### Rendering Backends

- **orge** - Pure Rust monochrome/grayscale rasterizer

### Export Formats

- **PNG** - Raster image (requires `export-png` feature)
- **SVG** - Vector image (requires `export-svg` feature)
- **PNM** - Portable pixmap formats (PBM, PGM, PPM)

## Output Structure

```
typf-tester/
├── typfme.py           # Main CLI tool
├── requirements.txt    # Python dependencies
├── README.md          # This file
├── fonts/             # Test fonts
│   ├── Kalnia[wdth,wght].ttf
│   ├── NotoSans-Regular.ttf
│   └── NotoNaskhArabic-Regular.ttf
└── output/            # Generated outputs
    ├── render-*.png
    ├── render-*.svg
    ├── compare-*.png
    ├── compare-*.svg
    └── benchmark_report.json
```

## Examples

### Quick Start

```bash
# 1. View available backends
python typfme.py info

# 2. Render samples with all backends
python typfme.py render

# 3. View the outputs
open output/*.png

# 4. Run benchmarks
python typfme.py bench

# 5. Compare backends visually
python typfme.py compare
open output/compare-*.png
```

### Advanced Usage

```bash
# Render only Arabic text samples
python typfme.py render --backend=harfbuzz --format=png

# High-precision benchmark
python typfme.py bench --iterations=5000 --detailed=True

# Generate report summary
cat output/benchmark_report.json | jq '.backends[] | select(.success==true) | {backend: .description, avg_ms: .avg_time_ms}' | head -20
```

## Benchmark Interpretation

### Metrics

- **Avg Time (ms)**: Average time per render operation in milliseconds
- **Ops/sec**: Number of render operations per second (higher is better)
- **Success Rate**: Percentage of successful renders vs. total attempts

### Expected Performance

Typical performance on modern hardware:

| Operation              | Target Time    | Notes                          |
|------------------------|----------------|--------------------------------|
| Simple Latin (48px)    | < 10ms         | Basic shaping, minimal glyphs  |
| Complex Arabic (48px)  | < 50ms         | Complex shaping, RTL           |
| Large text (128px)     | < 100ms        | High resolution rendering      |

### Real Benchmark Results

From comprehensive testing on macOS (M-series):

**Backend Performance (48px, 100 iterations):**
```
Backend           Avg Time (ms)   Ops/sec   Success
-------------------------------------------------
HARFBUZZ + ORGE      1.096         2,471     100%
NONE + ORGE          1.116         2,413     100%
```

**Performance by Text Type:**
```
Text Type        Avg Time (ms)   Ops/sec   Notes
-------------------------------------------------
mixed (17 chars)     0.70         3,237     Fastest
simple (27 chars)    1.51         1,647     Typical
```

**Shaping vs Rendering Breakdown:**
- Shaping: ~30-47µs (3% of total time)
- Rendering: ~1122µs (97% of total time)
- **Bottleneck:** Rendering is 37x slower than shaping

**Font Size Scaling:**
```
Size    Render Time   Scaling Factor
-------------------------------------
16px       195µs         1.0x
32px       384µs         1.9x
64px       975µs         5.0x
128px     2935µs        15.0x (super-linear)
```

For detailed performance analysis and optimization strategies, see:
- `../docs/PERFORMANCE.md` - Comprehensive optimization guide
- `output/benchmark_summary.md` - Latest benchmark results
- `output/benchmark_report.json` - Detailed performance data

## Analysis Tools

Three additional analysis scripts provide automated quality and performance analysis:

### `compare_performance.py` - Performance Comparison

Analyzes benchmark JSON data and creates performance comparison reports.

```bash
python typf-tester/compare_performance.py
```

**Features:**
- Groups benchmark data by renderer
- Generates ASCII comparison table showing avg time and ops/sec
- Creates visual bar chart for relative performance
- Identifies fastest/slowest renderers automatically

**Example Output:**
```
RENDERER PERFORMANCE COMPARISON
========================================
Renderer        Avg Time    Ops/sec
----------------------------------------
json            0.051ms     19603/s     ████████████████████
coregraphics    0.712ms      1404/s     ███
zeno            1.139ms      1318/s     ██
orge            1.738ms      1432/s     █
skia            1.615ms      1251/s     █

🏆 Fastest: JSON renderer (19,603 ops/sec)
📊 Best bitmap: CoreGraphics (1,404 ops/sec)
```

### `compare_quality.py` - Visual Quality Analysis

Analyzes PNG outputs to compare rendering quality across backends.

```bash
python typf-tester/compare_quality.py
```

**Metrics Analyzed:**
- **Coverage**: Percentage of non-white pixels (ink density)
- **Anti-aliasing**: Number of unique gray levels (0-255)
- **Smoothness**: Ratio of gray pixels to black pixels
- **File Size**: PNG compression efficiency

**Example Output:**
```
QUALITY INSIGHTS
========================================
🏆 Best Anti-Aliasing: CoreGraphics
   → 254.0 unique gray levels
   → 81.80% smoothness score

🎨 Smoothest Rendering: Orge
   → 98.21% smoothness score

💾 Most Efficient Compression: Orge
   → 4.27 KB average file size

📊 Cross-Shaper Consistency:
   CoreGraphics    ✓ Consistent (Δ 0.094% coverage)
   Orge            ⚠ Variance detected (Δ 0.371%)
```

**Requirements:** `pip install pillow`

### `bench_svg.py` - SVG vs PNG Performance

Compares SVG vector export vs PNG bitmap rendering performance.

```bash
python typf-tester/bench_svg.py
```

**Features:**
- 500 iterations per test for statistical significance
- Tests all renderers (CoreGraphics, Orge, Skia, Zeno)
- Compares speed and file size trade-offs
- Automated performance insights

**Key Finding**: **SVG is 23.3x faster than PNG on average!**

**Example Output:**
```
PERFORMANCE COMPARISON: SVG vs PNG
========================================================================
Renderer      PNG (ms)  SVG (ms)  Speedup     PNG Size  SVG Size  Ratio
------------------------------------------------------------------------
coregraphics  4.444     0.198     SVG 22.4x   9.76 KB   16.49 KB  1.69x
orge          4.680     0.199     SVG 23.5x   4.29 KB   16.49 KB  3.85x
skia          4.987     0.202     SVG 24.7x   5.12 KB   16.49 KB  3.22x
zeno          4.692     0.208     SVG 22.6x   8.95 KB   16.49 KB  1.84x

SUMMARY
========================================
📈 Average Performance:
   PNG: 4.701ms/op
   SVG: 0.202ms/op
   → SVG is 23.30x faster

💾 Average File Size:
   PNG: 7.03 KB
   SVG: 16.49 KB
   → SVG is 2.35x larger
```

**Use Cases:**
- **For Speed**: Use SVG for previews, interactive rendering
- **For Quality**: Use PNG for final output, print
- **For Scalability**: Use SVG for responsive designs

### `visual_diff.py` - Renderer Visual Comparison & Pixel Analysis

Creates side-by-side comparisons of PNG outputs from different renderers and performs quantitative pixel-level analysis.

#### Visual Comparison Mode

```bash
# Compare all renderers for a specific combination
python typf-tester/visual_diff.py --shaper harfbuzz --text latn

# Create comparisons for all shaper/text combinations
python typf-tester/visual_diff.py --all
```

**Features:**
- 2-column grid layout showing all renderers side-by-side
- Labels with renderer name and image dimensions
- Automatic border and spacing
- Output: `diff-{shaper}-{text}.png` files

**Example Output:**
```
diff-harfbuzz-latn.png:
┌─────────────────────────────────────────┐
│ Comparison: harfbuzz shaper, latn text  │
├──────────────────┬──────────────────────┤
│ coregraphics     │ orge                 │
│ (710×88)         │ (710×88)             │
├──────────────────┼──────────────────────┤
│ skia             │ zeno                 │
│ (710×88)         │ (710×88)             │
└──────────────────┴──────────────────────┘
```

#### Pixel-Level Analysis Mode

```bash
# Analyze specific combination with metrics and heatmaps
python typf-tester/visual_diff.py --analyze --shaper harfbuzz --text latn

# Analyze all combinations and generate full report
python typf-tester/visual_diff.py --analyze
```

**Features:**
- **MSE (Mean Squared Error)** - Average squared pixel difference (lower = more similar)
- **PSNR (Peak Signal-to-Noise Ratio)** - Quality metric in dB (higher = better, ∞ = identical)
- **Max Diff** - Maximum pixel difference (0-255 range)
- **Difference Heatmaps** - Visual representation of pixel differences (red = high difference)
- **JSON Report** - Machine-readable analysis at `output/pixel_diff_analysis.json`

**Example Output:**
```
🔬 Analyzing pixel differences for harfbuzz + latn
   coregraphics vs orge: MSE=3324.73, PSNR=12.91 dB, MaxDiff=255.0
   coregraphics vs skia: MSE=3148.94, PSNR=13.15 dB, MaxDiff=255.0
   orge vs skia: MSE=2062.38, PSNR=14.99 dB, MaxDiff=255.0
   ✅ Saved heatmap: heatmap-harfbuzz-coregraphics-vs-orge-latn.png
   ✅ Saved heatmap: heatmap-harfbuzz-orge-vs-skia-latn.png
```

**Interpreting Metrics:**
- **PSNR > 30 dB**: Excellent similarity (minor differences)
- **PSNR 20-30 dB**: Good similarity (visible differences)
- **PSNR 10-20 dB**: Moderate similarity (significant differences)
- **PSNR < 10 dB**: Poor similarity (major differences)

**Use Cases:**
- Quantify rendering differences between backends
- Track quality regressions across versions
- Identify which renderer pairs are most/least similar
- Debug antialiasing and rasterization issues
- Verify backend implementation consistency
- Documentation and presentations

**Requirements:** `pip install pillow numpy`

### `unified_report.py` - Comprehensive Analysis Report

Combines performance benchmarks, pixel-level quality analysis, and visual comparisons into a single unified report.

```bash
# Generate combined markdown + JSON reports
python typf-tester/unified_report.py
```

**Outputs:**
- `output/unified_analysis.md` - Human-readable markdown report
- `output/unified_analysis.json` - Machine-readable data for further processing

**Report Sections:**
1. **Performance Benchmarks** - Fastest configurations, ops/sec, timing data
2. **Visual Quality Analysis** - PSNR similarity matrix, most/least similar pairs
3. **Image Quality Metrics** - Coverage, anti-aliasing levels, file sizes by renderer
4. **Recommendations** - Best choices for performance, consistency, and quality

**Use Cases:**
- Get overview of all backend performance and quality trade-offs
- Compare multiple metrics in single document
- Track changes across development iterations
- Generate documentation for users
- Make informed backend selection decisions

**Requirements:** `pip install pillow numpy`

---

## Troubleshooting

### Import Error: typf not found

Build the Python bindings:

```bash
cd bindings/python
maturin develop --release --features shaping-hb,export-png,export-svg
```

### Font Not Found

Ensure fonts are copied to `typf-tester/fonts/`:

```bash
cp old-typf/testdata/fonts/*.ttf typf-tester/fonts/
```

### Backend Not Available

Check available backends with:

```bash
python typfme.py info
```

Some backends require specific feature flags during build.

### Permission Denied

Make the script executable:

```bash
chmod +x typfme.py
```

## Development

### Adding New Sample Texts

Edit `typfme.py` and add to `self.sample_texts` dictionary:

```python
self.sample_texts = {
    # ... existing texts
    "my_test": "Custom test text here",
}
```

### Adding New Fonts

1. Copy font to `typf-tester/fonts/`
2. Add to `self.fonts` dictionary in `typfme.py`:

```python
self.fonts = {
    # ... existing fonts
    "myfont": self.fonts_dir / "MyFont-Regular.ttf",
}
```

### Customizing Benchmarks

Modify `self.bench_sizes` for different font sizes:

```python
self.bench_sizes = [12.0, 16.0, 24.0, 32.0, 48.0, 64.0]
```

## Integration with CI/CD

Example GitHub Actions workflow:

```yaml
- name: Build TypF Python bindings
  run: |
    cd bindings/python
    maturin develop --release --features shaping-hb,export-png,export-svg

- name: Run TypF benchmarks
  run: |
    cd typf-tester
    python typfme.py bench --iterations=100

- name: Archive benchmark results
  uses: actions/upload-artifact@v3
  with:
    name: benchmark-results
    path: typf-tester/output/benchmark_report.json
```

## License

Apache-2.0

## Support

For issues or questions:
- File an issue on GitHub
- Contact FontLab: https://www.fontlab.com/

---

Made by FontLab https://www.fontlab.com/
