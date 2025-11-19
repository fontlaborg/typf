# TYPF Tester

Comprehensive testing and benchmarking tool for TYPF v2.0 text rendering pipeline.

Made by FontLab https://www.fontlab.com/

## Overview

`typfme.py` is a comprehensive CLI tool for testing, rendering, and benchmarking all TYPF backend combinations with multiple sample texts, font sizes, and output formats. It helps validate backend implementations, compare performance, and generate visual outputs for quality verification.

## Features

- Test all shaping + rendering backend combinations
- Render samples in PNG and SVG formats
- Benchmark performance across multiple texts and font sizes
- Generate detailed performance reports
- Support for multiple scripts (Latin, Arabic, mixed, etc.)
- Visual comparison of backend outputs

## Installation

### Prerequisites

1. Build TYPF Python bindings with required features:

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
- TYPF version
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
- Detailed JSON report saved to `output/benchmark_report.json`

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
- name: Build TYPF Python bindings
  run: |
    cd bindings/python
    maturin develop --release --features shaping-hb,export-png,export-svg

- name: Run TYPF benchmarks
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

Same as TYPF: MIT OR Apache-2.0

## Support

For issues or questions:
- File an issue on GitHub
- Contact FontLab: https://www.fontlab.com/

---

Made by FontLab https://www.fontlab.com/
