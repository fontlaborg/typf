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
echo "📚 Documentation built successfully!"
echo "View documentation with: zensical serve"
