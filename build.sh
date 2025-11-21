#!/bin/bash
# Build script for TYPF v2.0
# Community project by FontLab https://www.fontlab.org/

cd "$(dirname "$0")"
set -e

echo "Building TYPF v2.0 workspace (excluding Python bindings)..."
cargo build --release --workspace --exclude typf-py

echo ""
echo "Installing typf-cli with all available features..."
# On macOS, build with CoreText and CoreGraphics support
if [[ "$OSTYPE" == "darwin"* ]]; then
  cargo install --path crates/typf-cli --features "shaping-hb,shaping-mac,shaping-icu-hb,render-mac,render-skia,render-zeno"
else
  cargo install --path crates/typf-cli --features "shaping-hb,shaping-icu-hb,render-skia,render-zeno"
fi

echo ""
echo "Installing typf-bench with all available features..."
# On macOS, build with CoreText and CoreGraphics support
if [[ "$OSTYPE" == "darwin"* ]]; then
  cargo install --path crates/typf-bench --features "shaping-hb,shaping-mac,shaping-icu-hb,render-mac,render-skia,render-zeno"
else
  cargo install --path crates/typf-bench --features "shaping-hb,shaping-icu-hb,render-skia,render-zeno"
fi

echo ""
echo "Setting up Python environment..."
# Create or update virtual environment
if [ ! -d ".venv" ]; then
  uv venv --python 3.12
fi
source .venv/bin/activate

# Install Python dependencies and the package itself
echo "Installing Python dependencies and typfpy..."
uv pip install --upgrade .[dev]
uv pip install --system --upgrade .[dev]

echo ""
echo "Installing zensical CLI for documentation building..."
uv pip install zensical

echo ""
echo "✅ Build and installation complete!"
echo ""
echo "Installed components:"
echo "  - typf-cli (Rust CLI tool)"
echo "  - typf-bench (Rust benchmark tool)"
echo "  - typfpy (Python package with native bindings)"
echo "  - zensical (documentation builder)"
echo ""

echo "Building comprehensive documentation..."
echo ""
zensical build

echo "Running TYPF tester..."
echo ""
python typf-tester/typfme.py render
echo ""
python typf-tester/typfme.py bench

echo ""
echo "Running TYPF benchmark tool..."
echo ""
# Create a simple font directory for testing if none exists
if [ ! -d "benchmark-fonts" ]; then
  echo "Creating benchmark-fonts directory..."
  mkdir -p benchmark-fonts
  # Copy some system fonts if available (macOS)
  if [[ "$OSTYPE" == "darwin"* ]]; then
    cp "/System/Library/Fonts/Helvetica.ttc" benchmark-fonts/ 2>/dev/null || echo "Helvetica not found"
    cp "/System/Library/Fonts/Times.ttc" benchmark-fonts/ 2>/dev/null || echo "Times not found"
    cp "/System/Library/Fonts/Arial.ttf" benchmark-fonts/ 2>/dev/null || echo "Arial not found"
  fi
fi

# Run benchmark if fonts are available
if [ -n "$(ls -A test-fonts/ 2>/dev/null)" ]; then
  echo "Running comprehensive benchmarks (Level 1)..."
  typf-bench -i test-fonts -l 1 >typf-bench-level1.log 2>&1
  echo "Benchmark results saved to typf-bench-level1.log"
  echo ""
  echo "Sample benchmark results:"
  head -20 typf-bench-level1.log 2>/dev/null || echo "No benchmark output available"
else
  echo "No fonts found in test-fonts/ directory. Skipping benchmarks."
  echo "Add .ttf/.otf fonts to test-fonts/ and run: typf-bench -i test-fonts -l 1"
fi

echo ""
echo "📚 Documentation built successfully!"
echo "View documentation with: zensical serve"
